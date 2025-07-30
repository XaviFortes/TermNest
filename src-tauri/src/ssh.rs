use anyhow::{anyhow, Result};
use bytes::Bytes;
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use ssh2::{Channel, Session, Sftp};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

use crate::{AuthMethod, ConnectionStatus, FileItem, Protocol, Session as AppSession};

#[derive(Debug, Clone)]
pub struct SshConnection {
    pub session_id: String,
    pub session: Arc<Mutex<Session>>,
    pub channel: Arc<Mutex<Option<Channel>>>,
    pub sftp: Arc<Mutex<Option<Sftp>>>,
    pub app_session: AppSession,
}

#[derive(Debug, Clone)]
pub struct SshManager {
    connections: Arc<Mutex<HashMap<String, SshConnection>>>,
    app_handle: AppHandle,
}

impl SshManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            app_handle,
        }
    }

    pub async fn connect(&self, app_session: AppSession) -> Result<String> {
        let session_id = app_session.id.clone();
        
        // Emit connecting status
        self.emit_status(&session_id, "connecting", Some(&format!("Connecting to {}@{}", app_session.username, app_session.host)))?;

        // Create TCP connection
        let tcp = TcpStream::connect(format!("{}:{}", app_session.host, app_session.port))
            .map_err(|e| anyhow!("Failed to connect to {}:{}: {}", app_session.host, app_session.port, e))?;

        // Create SSH session
        let mut session = Session::new().map_err(|e| anyhow!("Failed to create SSH session: {}", e))?;
        session.set_tcp_stream(tcp);
        session.handshake().map_err(|e| anyhow!("SSH handshake failed: {}", e))?;

        // Authenticate
        match &app_session.auth_method {
            AuthMethod::Password => {
                return Err(anyhow!("Password authentication requires runtime password input"));
            }
            AuthMethod::PublicKey { key_path } => {
                let expanded_path = if key_path.starts_with("~/") {
                    let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
                    key_path.replacen("~/", &format!("{}/", home), 1)
                } else {
                    key_path.clone()
                };

                session
                    .userauth_pubkey_file(&app_session.username, None, Path::new(&expanded_path), None)
                    .map_err(|e| anyhow!("Public key authentication failed: {}", e))?;
            }
            AuthMethod::Agent => {
                session
                    .userauth_agent(&app_session.username)
                    .map_err(|e| anyhow!("SSH agent authentication failed: {}", e))?;
            }
        }

        if !session.authenticated() {
            return Err(anyhow!("Authentication failed"));
        }

        // Create PTY channel
        let mut channel = session
            .channel_session()
            .map_err(|e| anyhow!("Failed to create SSH channel: {}", e))?;

        // Request PTY
        channel
            .request_pty("xterm-256color", None, Some((80, 24, 0, 0)))
            .map_err(|e| anyhow!("Failed to request PTY: {}", e))?;

        // Start shell
        channel.shell().map_err(|e| anyhow!("Failed to start shell: {}", e))?;

        // Create SFTP session
        let sftp = session.sftp().map_err(|e| anyhow!("Failed to create SFTP session: {}", e))?;

        // Store connection
        let ssh_connection = SshConnection {
            session_id: session_id.clone(),
            session: Arc::new(Mutex::new(session)),
            channel: Arc::new(Mutex::new(Some(channel))),
            sftp: Arc::new(Mutex::new(Some(sftp))),
            app_session: app_session.clone(),
        };

        {
            let mut connections = self.connections.lock().unwrap();
            connections.insert(session_id.clone(), ssh_connection.clone());
        }

        // Start terminal output reader
        self.start_terminal_reader(ssh_connection.clone()).await?;

        // Emit connected status
        self.emit_status(&session_id, "connected", Some("Connection established"))?;

        Ok(format!("Connected to {}@{}", app_session.username, app_session.host))
    }

    pub async fn disconnect(&self, session_id: &str) -> Result<()> {
        let mut connections = self.connections.lock().unwrap();
        if let Some(connection) = connections.remove(session_id) {
            // Close channel
            if let Ok(mut channel_guard) = connection.channel.lock() {
                if let Some(mut channel) = channel_guard.take() {
                    let _ = channel.close();
                    let _ = channel.wait_close();
                }
            }

            // Close SFTP
            if let Ok(mut sftp_guard) = connection.sftp.lock() {
                if let Some(_sftp) = sftp_guard.take() {
                    // SFTP will be dropped automatically
                }
            }

            self.emit_status(session_id, "disconnected", Some("Disconnected"))?;
        }
        Ok(())
    }

    pub async fn send_input(&self, session_id: &str, input: &str) -> Result<()> {
        let connections = self.connections.lock().unwrap();
        if let Some(connection) = connections.get(session_id) {
            if let Ok(channel_guard) = connection.channel.lock() {
                if let Some(ref mut channel) = *channel_guard {
                    channel.write_all(input.as_bytes())
                        .map_err(|e| anyhow!("Failed to send input: {}", e))?;
                    channel.flush()
                        .map_err(|e| anyhow!("Failed to flush input: {}", e))?;
                }
            }
        }
        Ok(())
    }

    pub async fn list_directory(&self, session_id: &str, path: &str) -> Result<Vec<FileItem>> {
        let connections = self.connections.lock().unwrap();
        if let Some(connection) = connections.get(session_id) {
            if let Ok(sftp_guard) = connection.sftp.lock() {
                if let Some(ref sftp) = *sftp_guard {
                    let mut files = Vec::new();
                    
                    // Add parent directory if not root
                    if path != "/" {
                        files.push(FileItem {
                            name: "..".to_string(),
                            path: std::path::Path::new(path)
                                .parent()
                                .map(|p| p.to_string_lossy().to_string())
                                .unwrap_or_else(|| "/".to_string()),
                            size: 0,
                            is_directory: true,
                            modified: "".to_string(),
                        });
                    }

                    // List directory contents
                    let entries = sftp.readdir(Path::new(path))
                        .map_err(|e| anyhow!("Failed to read directory {}: {}", path, e))?;

                    for (file_path, stat) in entries {
                        let name = file_path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "unknown".to_string());

                        let is_directory = stat.file_type().is_dir();
                        let size = if is_directory { 0 } else { stat.size.unwrap_or(0) };
                        
                        // Format modification time
                        let modified = if let Some(mtime) = stat.mtime {
                            chrono::DateTime::from_timestamp(mtime as i64, 0)
                                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                .unwrap_or_else(|| "unknown".to_string())
                        } else {
                            "unknown".to_string()
                        };

                        files.push(FileItem {
                            name,
                            path: file_path.to_string_lossy().to_string(),
                            size,
                            is_directory,
                            modified,
                        });
                    }

                    return Ok(files);
                }
            }
        }
        Err(anyhow!("No SFTP connection found for session {}", session_id))
    }

    pub fn get_connection_status(&self, session_id: &str) -> Option<ConnectionStatus> {
        let connections = self.connections.lock().unwrap();
        if connections.contains_key(session_id) {
            Some(ConnectionStatus {
                session_id: session_id.to_string(),
                status: "connected".to_string(),
                message: Some("Connected".to_string()),
            })
        } else {
            None
        }
    }

    async fn start_terminal_reader(&self, connection: SshConnection) -> Result<()> {
        let app_handle = self.app_handle.clone();
        let session_id = connection.session_id.clone();

        tokio::spawn(async move {
            let mut buffer = [0u8; 4096];
            loop {
                if let Ok(channel_guard) = connection.channel.lock() {
                    if let Some(ref mut channel) = *channel_guard {
                        match channel.read(&mut buffer) {
                            Ok(0) => break, // EOF
                            Ok(n) => {
                                let output = String::from_utf8_lossy(&buffer[..n]).to_string();
                                let _ = app_handle.emit("terminal_output", serde_json::json!({
                                    "session_id": session_id,
                                    "data": output
                                }));
                            }
                            Err(e) => {
                                eprintln!("Error reading from channel: {}", e);
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
                
                // Small delay to prevent busy loop
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });

        Ok(())
    }

    fn emit_status(&self, session_id: &str, status: &str, message: Option<&str>) -> Result<()> {
        let status = ConnectionStatus {
            session_id: session_id.to_string(),
            status: status.to_string(),
            message: message.map(|s| s.to_string()),
        };
        
        self.app_handle
            .emit("connection_status", &status)
            .map_err(|e| anyhow!("Failed to emit status: {}", e))?;
        
        Ok(())
    }
}
