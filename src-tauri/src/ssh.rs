use anyhow::{anyhow, Result};
use ssh2::{Channel, Session, Sftp};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

use crate::{AuthMethod, ConnectionStatus, FileItem, Session as AppSession};

#[derive(Clone)]
#[allow(dead_code)]
pub struct SshConnection {
    pub session_id: String,
    pub session: Arc<Mutex<Session>>,
    pub channel: Arc<Mutex<Option<Channel>>>,
    pub sftp: Arc<Mutex<Option<Sftp>>>,
    pub app_session: AppSession,
}

#[derive(Clone)]
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
        self.emit_status(&session_id, "connecting", Some(&format!("🔌 Connecting to {}@{}:{}", app_session.username, app_session.host, app_session.port)))?;

        // Create TCP connection with timeout
        self.emit_status(&session_id, "connecting", Some(&format!("🌐 Resolving hostname: {}", app_session.host)))?;
        let addr = format!("{}:{}", app_session.host, app_session.port);
        let socket_addrs: Vec<std::net::SocketAddr> = addr.to_socket_addrs()
            .map_err(|e| anyhow!("Failed to resolve address {}: {}", addr, e))?
            .collect();
        
        self.emit_status(&session_id, "connecting", Some(&format!("🔗 Establishing TCP connection to {}", addr)))?;
        let tcp = if let Some(socket_addr) = socket_addrs.first() {
            TcpStream::connect_timeout(socket_addr, Duration::from_secs(10))
                .map_err(|e| anyhow!("Failed to connect to {}: {}", addr, e))?
        } else {
            return Err(anyhow!("No valid socket address found for {}", addr));
        };
        self.emit_status(&session_id, "connecting", Some("✅ TCP connection established"))?;

        // Create SSH session
        self.emit_status(&session_id, "connecting", Some("🔐 Initializing SSH session"))?;
        let mut session = Session::new().map_err(|e| anyhow!("Failed to create SSH session: {}", e))?;
        session.set_tcp_stream(tcp);
        
        self.emit_status(&session_id, "connecting", Some("🤝 Performing SSH protocol handshake"))?;
        session.handshake().map_err(|e| anyhow!("SSH handshake failed: {}", e))?;
        self.emit_status(&session_id, "connecting", Some("✅ SSH handshake completed"))?;

        // Authenticate
        self.emit_status(&session_id, "connecting", Some("🔑 Starting user authentication"))?;
        match &app_session.auth_method {
            AuthMethod::Password => {
                self.emit_status(&session_id, "connecting", Some("❌ Password authentication not supported yet"))?;
                return Err(anyhow!("Password authentication requires runtime password input"));
            }
            AuthMethod::PublicKey { key_path } => {
                self.emit_status(&session_id, "connecting", Some(&format!("🔐 Trying primary SSH key: {}", key_path)))?;
                let expanded_path = if key_path.starts_with("~/") {
                    let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
                    key_path.replacen("~/", &format!("{}/", home), 1)
                } else {
                    key_path.clone()
                };

                // Try the specified key first
                let mut auth_success = false;
                if std::path::Path::new(&expanded_path).exists() {
                    self.emit_status(&session_id, "connecting", Some(&format!("🔍 Key file found: {}", expanded_path)))?;
                    if let Ok(_) = session.userauth_pubkey_file(&app_session.username, None, Path::new(&expanded_path), None) {
                        self.emit_status(&session_id, "connecting", Some("✅ Primary key authentication successful"))?;
                        auth_success = true;
                    } else {
                        self.emit_status(&session_id, "connecting", Some("❌ Primary key authentication failed"))?;
                    }
                } else {
                    self.emit_status(&session_id, "connecting", Some(&format!("❌ Primary key file not found: {}", expanded_path)))?;
                }
                
                // If specified key fails, try common key locations as fallback
                if !auth_success {
                    self.emit_status(&session_id, "connecting", Some("🔄 Trying fallback SSH keys"))?;
                    let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
                    let common_keys = vec![
                        format!("{}/.ssh/id_ed25519", home),
                        format!("{}/.ssh/id_rsa", home),
                        format!("{}/.ssh/id_ecdsa", home),
                    ];
                    
                    for fallback_key in common_keys {
                        if std::path::Path::new(&fallback_key).exists() {
                            self.emit_status(&session_id, "connecting", Some(&format!("🔑 Trying fallback key: {}", fallback_key.split('/').last().unwrap_or("unknown"))))?;
                            if let Ok(_) = session.userauth_pubkey_file(&app_session.username, None, Path::new(&fallback_key), None) {
                                self.emit_status(&session_id, "connecting", Some(&format!("✅ Fallback key authentication successful: {}", fallback_key.split('/').last().unwrap_or("unknown"))))?;
                                auth_success = true;
                                break;
                            } else {
                                self.emit_status(&session_id, "connecting", Some(&format!("❌ Fallback key failed: {}", fallback_key.split('/').last().unwrap_or("unknown"))))?;
                            }
                        } else {
                            self.emit_status(&session_id, "connecting", Some(&format!("⚠️ Fallback key not found: {}", fallback_key.split('/').last().unwrap_or("unknown"))))?;
                        }
                    }
                }
                
                if !auth_success {
                    return Err(anyhow!("Public key authentication failed. Tried:\n1. Specified key: {}\n2. Fallback keys: ~/.ssh/id_ed25519, ~/.ssh/id_rsa, ~/.ssh/id_ecdsa\n\nPlease check:\n1. Key files exist and have correct permissions (600)\n2. Key is in correct format\n3. Key matches server's authorized_keys", expanded_path));
                }
            }
            AuthMethod::Agent => {
                self.emit_status(&session_id, "connecting", Some("🔐 Trying SSH agent authentication"))?;
                // First try to connect to SSH agent
                match session.userauth_agent(&app_session.username) {
                    Ok(_) => {
                        self.emit_status(&session_id, "connecting", Some("✅ SSH agent authentication successful"))?;
                    },
                    Err(e) => {
                        self.emit_status(&session_id, "connecting", Some(&format!("❌ SSH agent failed: {}", e)))?;
                        self.emit_status(&session_id, "connecting", Some("🔄 Falling back to key files"))?;
                        // If agent fails, try common key locations as fallback
                        let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
                        let common_keys = vec![
                            format!("{}/.ssh/id_rsa", home),
                            format!("{}/.ssh/id_ed25519", home),
                            format!("{}/.ssh/id_ecdsa", home),
                        ];
                        
                        let mut key_found = false;
                        for key_path in common_keys {
                            if std::path::Path::new(&key_path).exists() {
                                self.emit_status(&session_id, "connecting", Some(&format!("🔑 Trying fallback key: {}", key_path.split('/').last().unwrap_or("unknown"))))?;
                                if let Ok(_) = session.userauth_pubkey_file(&app_session.username, None, Path::new(&key_path), None) {
                                    self.emit_status(&session_id, "connecting", Some(&format!("✅ Fallback key successful: {}", key_path.split('/').last().unwrap_or("unknown"))))?;
                                    key_found = true;
                                    break;
                                } else {
                                    self.emit_status(&session_id, "connecting", Some(&format!("❌ Fallback key failed: {}", key_path.split('/').last().unwrap_or("unknown"))))?;
                                }
                            }
                        }
                        
                        if !key_found {
                            return Err(anyhow!("SSH agent authentication failed: {}.\n\nTroubleshooting:\n1. Start SSH agent: eval $(ssh-agent)\n2. Add keys: ssh-add ~/.ssh/id_rsa\n3. Or switch to 'PublicKey' auth method and specify key path", e));
                        }
                    }
                }
            }
        }

        if !session.authenticated() {
            return Err(anyhow!("Authentication failed"));
        }

        self.emit_status(&session_id, "connecting", Some("✅ User authentication completed"))?;

        // Create PTY channel
        self.emit_status(&session_id, "connecting", Some("📺 Creating terminal session"))?;
        let mut channel = session
            .channel_session()
            .map_err(|e| anyhow!("Failed to create SSH channel: {}", e))?;

        // Request PTY
        self.emit_status(&session_id, "connecting", Some("🖥️ Requesting pseudo-terminal (PTY)"))?;
        channel
            .request_pty("xterm-256color", None, Some((80, 24, 0, 0)))
            .map_err(|e| anyhow!("Failed to request PTY: {}", e))?;
        self.emit_status(&session_id, "connecting", Some("✅ PTY allocated"))?;

        // Start shell
        self.emit_status(&session_id, "connecting", Some("🐚 Starting remote shell"))?;
        channel.shell().map_err(|e| anyhow!("Failed to start shell: {}", e))?;
        self.emit_status(&session_id, "connecting", Some("✅ Shell started"))?;

        // Create SFTP session (optional, non-blocking)
        self.emit_status(&session_id, "connecting", Some("📂 Initializing SFTP support"))?;
        let sftp = match session.sftp() {
            Ok(sftp) => {
                self.emit_status(&session_id, "connecting", Some("✅ SFTP support enabled"))?;
                Some(sftp)
            }
            Err(e) => {
                self.emit_status(&session_id, "connecting", Some(&format!("⚠️ SFTP not available: {}", e)))?;
                None
            }
        };

        // Store connection
        self.emit_status(&session_id, "connecting", Some("💾 Storing connection details"))?;
        let ssh_connection = SshConnection {
            session_id: session_id.clone(),
            session: Arc::new(Mutex::new(session)),
            channel: Arc::new(Mutex::new(Some(channel))),
            sftp: Arc::new(Mutex::new(sftp)),
            app_session: app_session.clone(),
        };

        {
            let mut connections = self.connections.lock().unwrap();
            connections.insert(session_id.clone(), ssh_connection.clone());
        }

        self.emit_status(&session_id, "connecting", Some("✅ Connection stored successfully"))?;

        // Send initial newline to trigger shell prompt after a small delay
        let ssh_connection_for_init = ssh_connection.clone();
        let app_handle_clone = self.app_handle.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(2000)).await;
            if let Ok(mut channel_guard) = ssh_connection_for_init.channel.lock() {
                if let Some(ref mut channel) = *channel_guard {
                    let _ = channel.write_all(b"\n");
                    let _ = channel.flush();
                }
            }
            
            // Start terminal reader after initial setup
            let session_id_clone = ssh_connection_for_init.session_id.clone();
            tokio::spawn(async move {
                let mut buffer = [0u8; 4096];
                loop {
                    let read_result = tokio::task::spawn_blocking({
                        let connection_clone = ssh_connection_for_init.clone();
                        move || {
                            if let Ok(mut channel_guard) = connection_clone.channel.lock() {
                                if let Some(ref mut channel) = *channel_guard {
                                    channel.read(&mut buffer).map(|n| (n, buffer))
                                } else {
                                    Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "Channel closed"))
                                }
                            } else {
                                Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "Cannot lock channel"))
                            }
                        }
                    }).await;

                    match read_result {
                        Ok(Ok((0, _))) => break,
                        Ok(Ok((n, buffer))) => {
                            let output = String::from_utf8_lossy(&buffer[..n]).to_string();
                            let _ = app_handle_clone.emit("terminal_output", serde_json::json!({
                                "session_id": session_id_clone,
                                "data": output
                            }));
                        }
                        _ => {
                            tokio::time::sleep(Duration::from_millis(100)).await;
                        }
                    }
                    
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            });
        });

        // Emit connected status
        self.emit_status(&session_id, "connected", Some("🎉 SSH connection established successfully!"))?;
        self.emit_status(&session_id, "connected", Some(&format!("Ready to use terminal on {}@{}", app_session.username, app_session.host)))?;

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
            if let Ok(mut channel_guard) = connection.channel.lock() {
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
                } else {
                    return Err(anyhow!("SFTP is not available for this session. SFTP may not be supported by the server or failed to initialize during connection."));
                }
            }
        }
        Err(anyhow!("No SSH connection found for session {}", session_id))
    }

    pub fn get_connection_status(&self, session_id: &str) -> Option<ConnectionStatus> {
        let connections = self.connections.lock().unwrap();
        if connections.contains_key(session_id) {
            Some(ConnectionStatus::Connected)
        } else {
            Some(ConnectionStatus::Disconnected)
        }
    }

    fn start_terminal_reader(&self, connection: SshConnection) -> Result<()> {
        let app_handle = self.app_handle.clone();
        let session_id = connection.session_id.clone();

        tokio::spawn(async move {
            // Give the shell a moment to start up
            tokio::time::sleep(Duration::from_millis(500)).await;
            
            loop {
                let connection_clone = connection.clone();
                let session_id_clone = session_id.clone();
                
                // Use spawn_blocking to handle the potentially blocking read operation
                let read_result = tokio::task::spawn_blocking(move || {
                    let mut buffer = [0u8; 4096];
                    if let Ok(mut channel_guard) = connection_clone.channel.lock() {
                        if let Some(ref mut channel) = *channel_guard {
                            channel.read(&mut buffer).map(|n| (n, buffer))
                        } else {
                            Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "Channel closed"))
                        }
                    } else {
                        Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "Cannot lock channel"))
                    }
                }).await;

                match read_result {
                    Ok(Ok((0, _))) => break, // EOF
                    Ok(Ok((n, buffer))) => {
                        let output = String::from_utf8_lossy(&buffer[..n]).to_string();
                        let _ = app_handle.emit("terminal_output", serde_json::json!({
                            "session_id": session_id,
                            "data": output
                        }));
                    }
                    Ok(Err(e)) => {
                        eprintln!("Error reading from channel: {}", e);
                        // Don't break immediately, give it another chance
                        tokio::time::sleep(Duration::from_millis(1000)).await;
                    }
                    Err(e) => {
                        eprintln!("Task join error: {}", e);
                        break;
                    }
                }
                
                // Small delay to prevent busy loop
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });

        Ok(())
    }

    fn emit_status(&self, session_id: &str, status: &str, message: Option<&str>) -> Result<()> {
        let status_enum = match status {
            "connecting" => ConnectionStatus::Connecting,
            "connected" => ConnectionStatus::Connected,
            "disconnected" => ConnectionStatus::Disconnected,
            _ => ConnectionStatus::Error(message.unwrap_or("Unknown error").to_string()),
        };
        
        self.app_handle
            .emit("connection_status", &serde_json::json!({
                "session_id": session_id,
                "status": status,
                "status_enum": status_enum,
                "message": message
            }))
            .map_err(|e| anyhow!("Failed to emit status: {}", e))?;
        
        Ok(())
    }
}
