use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use ssh2::{Channel, Session};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_method: AuthMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    Password { password: String },
    PublicKey { private_key_path: String },
    Agent,
}

#[derive(Clone, serde::Serialize)]
struct TerminalEvent {
    session_id: String,
    event_type: String,
    data: String,
}

// Separate reader and writer handles to avoid mutex contention
pub struct SshConnection {
    session_id: String,
    input_tx: mpsc::UnboundedSender<String>,
    reader_shutdown: Arc<AtomicBool>,
    writer_shutdown: Arc<AtomicBool>,
    input_shutdown: Arc<AtomicBool>,
    reader_handle: Option<thread::JoinHandle<()>>,
    writer_handle: Option<thread::JoinHandle<()>>,
    input_handle: Option<thread::JoinHandle<()>>,
}

impl SshConnection {
    pub fn new(
        session_id: String,
        channel: Channel,
        app_handle: AppHandle,
    ) -> Result<Self> {
        let (writer_tx, mut writer_rx) = mpsc::unbounded_channel::<Vec<u8>>();
        let (input_tx, mut input_rx) = mpsc::unbounded_channel::<String>();
        
        let reader_shutdown = Arc::new(AtomicBool::new(false));
        let writer_shutdown = Arc::new(AtomicBool::new(false));
        let input_shutdown = Arc::new(AtomicBool::new(false));

        // Use Arc<Mutex<Channel>> to share the channel safely between threads
        let shared_channel = Arc::new(Mutex::new(channel));
        
        // Reader thread
        let reader_channel = shared_channel.clone();
        let reader_shutdown_clone = reader_shutdown.clone();
        let session_id_clone = session_id.clone();
        let app_handle_clone = app_handle.clone();
        
        let reader_handle = thread::spawn(move || {
            let mut buffer = [0u8; 4096];
            
            while !reader_shutdown_clone.load(Ordering::Relaxed) {
                let read_result = {
                    let mut channel = reader_channel.lock().unwrap();
                    channel.read(&mut buffer)
                };
                
                match read_result {
                    Ok(0) => {
                        // EOF - connection closed
                        println!("SSH connection {} closed", session_id_clone);
                        break;
                    }
                    Ok(n) => {
                        let data = String::from_utf8_lossy(&buffer[..n]).to_string();
                        
                        let event = TerminalEvent {
                            session_id: session_id_clone.clone(),
                            event_type: "data".to_string(),
                            data,
                        };
                        
                        if let Err(e) = app_handle_clone.emit("terminal-data", &event) {
                            eprintln!("Failed to emit terminal data: {}", e);
                        }
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::WouldBlock {
                            // Non-blocking read with no data, sleep briefly
                            thread::sleep(Duration::from_millis(10));
                            continue;
                        }
                        eprintln!("SSH read error: {}", e);
                        break;
                    }
                }
            }
            
            println!("SSH reader thread for {} exiting", session_id_clone);
        });
        
        // Writer thread
        let writer_channel = shared_channel.clone();
        let writer_shutdown_clone = writer_shutdown.clone();
        let session_id_writer = session_id.clone();
        
        let writer_handle = thread::spawn(move || {
            while !writer_shutdown_clone.load(Ordering::Relaxed) {
                match writer_rx.try_recv() {
                    Ok(data) => {
                        let write_result = {
                            let mut channel = writer_channel.lock().unwrap();
                            channel.write_all(&data).and_then(|_| channel.flush())
                        };
                        
                        if let Err(e) = write_result {
                            eprintln!("SSH write error for {}: {}", session_id_writer, e);
                            break;
                        }
                    }
                    Err(mpsc::error::TryRecvError::Empty) => {
                        // No data available, sleep briefly and check shutdown
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(mpsc::error::TryRecvError::Disconnected) => {
                        println!("Writer channel disconnected for {}", session_id_writer);
                        break;
                    }
                }
            }
            
            println!("SSH writer thread for {} exiting", session_id_writer);
        });

        // ---- Input buffering and debouncing thread ----
        let input_writer_tx = writer_tx.clone();
        let input_shutdown_clone = input_shutdown.clone();
        let input_handle = thread::spawn(move || {
            use std::time::{Instant, Duration};
            let mut buffer = String::new();
            let mut last_flush = Instant::now();
            let flush_interval = Duration::from_millis(100); // You can tune this!

            loop {
                if input_shutdown_clone.load(Ordering::Relaxed) {
                    break;
                }

                // Non-blocking receive: drain all available input
                let mut received_any = false;
                while let Ok(data) = input_rx.try_recv() {
                    buffer.push_str(&data);
                    received_any = true;
                }

                let now = Instant::now();
                // Flush if interval has elapsed or there is accumulated input after waiting
                if (!buffer.is_empty() && now.duration_since(last_flush) > flush_interval) || (received_any && buffer.len() > 1024) {
                    // Write to the SSH writer
                    let bytes = buffer.clone().into_bytes();
                    if let Err(e) = input_writer_tx.send(bytes) {
                        eprintln!("[SSH] Failed to send buffered input: {e}");
                    }
                    buffer.clear();
                    last_flush = now;
                }
                thread::sleep(Duration::from_millis(10));
            }
            // Flush any remaining buffer on exit
            if !buffer.is_empty() {
                let _ = input_writer_tx.send(buffer.into_bytes());
            }
        });
        
        Ok(SshConnection {
            session_id,
            input_tx,
            reader_shutdown,
            writer_shutdown,
            input_shutdown,
            reader_handle: Some(reader_handle),
            writer_handle: Some(writer_handle),
            input_handle: Some(input_handle),
        })
    }
    
    pub fn send_input(&self, input: &str) -> Result<()> {
        self.input_tx
            .send(input.to_string())
            .map_err(|e| anyhow!("Failed to send input for buffering: {}", e))?;
        // let data = input.as_bytes().to_vec();
        // self.writer_tx.send(data)
            // .map_err(|e| anyhow!("Failed to send input: {}", e))?;
        Ok(())
    }
    
    pub fn close(&mut self) {
        println!("Closing SSH connection {}", self.session_id);
        
        // Signal threads to shutdown
        self.reader_shutdown.store(true, Ordering::Relaxed);
        self.writer_shutdown.store(true, Ordering::Relaxed);
        self.input_shutdown.store(true, Ordering::Relaxed);
        
        // Wait for threads to finish
        if let Some(handle) = self.reader_handle.take() {
            if let Err(e) = handle.join() {
                eprintln!("Reader thread join error: {:?}", e);
            }
        }
        
        if let Some(handle) = self.writer_handle.take() {
            if let Err(e) = handle.join() {
                eprintln!("Writer thread join error: {:?}", e);
            }
        }

        if let Some(handle) = self.input_handle.take() {
            if let Err(e) = handle.join() {
                eprintln!("Input thread join error: {:?}", e);
            }
        }
    }
}

impl Drop for SshConnection {
    fn drop(&mut self) {
        self.close();
    }
}

pub struct SshManager {
    connections: Arc<Mutex<HashMap<String, SshConnection>>>,
}

impl SshManager {
    pub fn new() -> Self {
        SshManager {
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn connect(
        &self,
        session_id: String,
        config: SshConfig,
        app_handle: AppHandle,
    ) -> Result<()> {
        println!("Connecting to SSH host: {}@{}:{}", config.username, config.host, config.port);
        
        // Establish TCP connection
        let tcp_stream = TcpStream::connect(format!("{}:{}", config.host, config.port))?;
        
        // Create SSH session
        let mut session = Session::new()?;
        session.set_tcp_stream(tcp_stream);
        session.handshake()?;
        
        // Authenticate based on auth method
        match &config.auth_method {
            AuthMethod::Password { password } => {
                println!("Authenticating with password for user: {}", config.username);
                session.userauth_password(&config.username, password)?;
            }
            AuthMethod::PublicKey { private_key_path } => {
                println!("Authenticating with public key: {}", private_key_path);
                let private_key_path = std::path::Path::new(private_key_path);
                session.userauth_pubkey_file(&config.username, None, private_key_path, None)?;
            }
            AuthMethod::Agent => {
                println!("Authenticating with SSH agent for user: {}", config.username);
                let mut agent = session.agent()?;
                agent.connect()?;
                agent.list_identities()?;
                
                let identities = agent.identities()?;
                let mut authenticated = false;
                
                for identity in identities {
                    if agent.userauth(&config.username, &identity).is_ok() {
                        authenticated = true;
                        break;
                    }
                }
                
                if !authenticated {
                    return Err(anyhow!("SSH agent authentication failed - no suitable identity found"));
                }
            }
        }
        
        if !session.authenticated() {
            return Err(anyhow!("SSH authentication failed"));
        }
        
        println!("SSH authentication successful for {}", session_id);
        
        // Open channel and request PTY
        let mut channel = session.channel_session()?;
        channel.request_pty("xterm-256color", None, Some((80, 24, 0, 0)))?;
        
        // Set up the shell - this is crucial for interactive terminal
        channel.shell()?;
        
        // Important: Set the channel to non-blocking mode to prevent deadlocks
        session.set_blocking(false);
        
        println!("SSH channel established for {}", session_id);
        
        // Create connection wrapper
        let connection = SshConnection::new(session_id.clone(), channel, app_handle)?;
        
        // Store connection
        {
            let mut connections = self.connections.lock().unwrap();
            connections.insert(session_id.clone(), connection);
        }
        
        println!("SSH connection {} ready", session_id);
        Ok(())
    }
    
    pub fn send_input(&self, session_id: &str, input: &str) -> Result<()> {
        let connections = self.connections.lock().unwrap();
        
        if let Some(connection) = connections.get(session_id) {
            connection.send_input(input)?;
            Ok(())
        } else {
            Err(anyhow!("Session not found: {}", session_id))
        }
    }
    
    pub fn disconnect(&self, session_id: &str) -> Result<()> {
        let mut connections = self.connections.lock().unwrap();
        
        if let Some(mut connection) = connections.remove(session_id) {
            connection.close();
            println!("Disconnected SSH session: {}", session_id);
            Ok(())
        } else {
            Err(anyhow!("Session not found: {}", session_id))
        }
    }
    
    pub fn list_sessions(&self) -> Vec<String> {
        let connections = self.connections.lock().unwrap();
        connections.keys().cloned().collect()
    }
}

// Tauri commands
#[tauri::command]
pub async fn ssh_connect(
    session_id: String,
    config: SshConfig,
    app_handle: AppHandle,
    state: tauri::State<'_, Arc<SshManager>>,
) -> Result<(), String> {
    state
        .connect(session_id, config, app_handle)
        .map_err(|e| format!("Connection failed: {}", e))
}

#[tauri::command]
pub async fn ssh_connect_with_password(
    session_id: String,
    mut config: SshConfig,
    password: String,
    app_handle: AppHandle,
    state: tauri::State<'_, Arc<SshManager>>,
) -> Result<(), String> {
    // Update config with password
    config.auth_method = AuthMethod::Password { password };
    
    state
        .connect(session_id, config, app_handle)
        .map_err(|e| format!("Connection failed: {}", e))
}

#[tauri::command]
pub async fn ssh_send_input(
    session_id: String,
    input: String,
    state: tauri::State<'_, Arc<SshManager>>,
) -> Result<(), String> {
    state
        .send_input(&session_id, &input)
        .map_err(|e| format!("Send input failed: {}", e))
}

#[tauri::command]
pub async fn ssh_disconnect(
    session_id: String,
    state: tauri::State<'_, Arc<SshManager>>,
) -> Result<(), String> {
    state
        .disconnect(&session_id)
        .map_err(|e| format!("Disconnect failed: {}", e))
}

#[tauri::command]
pub async fn ssh_list_sessions(
    state: tauri::State<'_, Arc<SshManager>>,
) -> Result<Vec<String>, String> {
    Ok(state.list_sessions())
}
