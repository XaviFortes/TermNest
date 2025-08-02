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

// Utility functions
fn get_home_directory() -> String {
    #[cfg(target_os = "windows")]
    {
        std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Default".to_string())
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
    }
}

fn get_filename_from_path(path: &str) -> &str {
    #[cfg(target_os = "windows")]
    {
        path.split('\\').last().unwrap_or("unknown")
    }
    #[cfg(not(target_os = "windows"))]
    {
        path.split('/').last().unwrap_or("unknown")
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct SshConnection {
    pub session_id: String,
    pub session: Arc<Mutex<Session>>,
    pub channel: Arc<Mutex<Option<Channel>>>,
    pub sftp: Arc<Mutex<Option<Sftp>>>,
    pub app_session: AppSession,
    pub reader_started: Arc<Mutex<bool>>,
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

        // Check session state and server capabilities
        println!("DEBUG: Querying authentication methods for user: {}", app_session.username);
        let auth_methods = match session.auth_methods(&app_session.username) {
            Ok(methods) => {
                println!("DEBUG: Server supports auth methods: {}", methods);
                self.emit_status(&session_id, "connecting", Some(&format!("🔍 Server auth methods: {}", methods)))?;
                methods.to_string()
            }
            Err(e) => {
                println!("DEBUG: Failed to query auth methods: {}", e);
                self.emit_status(&session_id, "connecting", Some(&format!("⚠️ Could not query auth methods: {}", e)))?;
                "publickey,password".to_string() // Default assumption
            }
        };

        // Check if server supports public key authentication
        if !auth_methods.contains("publickey") {
            return Err(anyhow!("Server does not support public key authentication. Supported methods: {}", auth_methods));
        }

        println!("DEBUG: Starting user authentication");
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
                println!("DEBUG: Checking key file: {}", expanded_path);
                if std::path::Path::new(&expanded_path).exists() {
                    self.emit_status(&session_id, "connecting", Some(&format!("🔍 Key file found: {}", expanded_path)))?;
                    
                    // Check if this is an Ed25519 key and try different approaches
                    if expanded_path.contains("id_ed25519") {
                        println!("DEBUG: Detected Ed25519 key, trying different authentication methods");
                        
                        // Method 1: Try with explicit None passphrase
                        println!("DEBUG: Method 1 - Ed25519 with None passphrase");
                        
                        // Check if corresponding public key exists
                        let pub_key_path = format!("{}.pub", expanded_path);
                        println!("DEBUG: Checking for public key: {}", pub_key_path);
                        if std::path::Path::new(&pub_key_path).exists() {
                            println!("DEBUG: Public key found: {}", pub_key_path);
                        } else {
                            println!("DEBUG: Public key NOT found: {}", pub_key_path);
                        }
                        
                        // Try to read the private key content to check format
                        match std::fs::read_to_string(&expanded_path) {
                            Ok(key_content) => {
                                let lines: Vec<&str> = key_content.lines().collect();
                                println!("DEBUG: Private key has {} lines", lines.len());
                                if let Some(first_line) = lines.first() {
                                    println!("DEBUG: Key starts with: {}", first_line);
                                }
                                if let Some(last_line) = lines.last() {
                                    println!("DEBUG: Key ends with: {}", last_line);
                                }
                            }
                            Err(e) => {
                                println!("DEBUG: Failed to read private key: {}", e);
                            }
                        }
                        
                        println!("DEBUG: Session authenticated before attempt: {}", session.authenticated());
                        match session.userauth_pubkey_file(&app_session.username, Some(Path::new(&pub_key_path)), Path::new(&expanded_path), None) {
                            Ok(_) => {
                                println!("DEBUG: Ed25519 authentication successful with None passphrase!");
                                println!("DEBUG: Session authenticated after success: {}", session.authenticated());
                                self.emit_status(&session_id, "connecting", Some("✅ Ed25519 key authentication successful"))?;
                                auth_success = true;
                            }
                            Err(e) => {
                                println!("DEBUG: Ed25519 Method 1 failed: {}", e);
                                println!("DEBUG: Session authenticated after failure: {}", session.authenticated());
                                
                                // Method 2: Try with empty string passphrase
                                println!("DEBUG: Method 2 - Ed25519 with empty string passphrase");
                                match session.userauth_pubkey_file(&app_session.username, None, Path::new(&expanded_path), Some("")) {
                                    Ok(_) => {
                                        println!("DEBUG: Ed25519 authentication successful with empty passphrase!");
                                        println!("DEBUG: Session authenticated after success: {}", session.authenticated());
                                        self.emit_status(&session_id, "connecting", Some("✅ Ed25519 key authentication successful"))?;
                                        auth_success = true;
                                    }
                                    Err(e2) => {
                                        println!("DEBUG: Ed25519 Method 2 failed: {}", e2);
                                        println!("DEBUG: Session authenticated after failure: {}", session.authenticated());
                                        self.emit_status(&session_id, "connecting", Some(&format!("❌ Ed25519 key failed: {}", e)))?;
                                    }
                                }
                            }
                        }
                    } else {
                        // Standard RSA/ECDSA key handling
                        println!("DEBUG: Standard key type, using normal authentication");
                        
                        // Check if corresponding public key exists
                        let pub_key_path = format!("{}.pub", expanded_path);
                        println!("DEBUG: Checking for public key: {}", pub_key_path);
                        let pub_key_exists = std::path::Path::new(&pub_key_path).exists();
                        if pub_key_exists {
                            println!("DEBUG: Public key found: {}", pub_key_path);
                        } else {
                            println!("DEBUG: Public key NOT found: {}", pub_key_path);
                        }
                        
                        // Try to read the private key content to check format
                        match std::fs::read_to_string(&expanded_path) {
                            Ok(key_content) => {
                                let lines: Vec<&str> = key_content.lines().collect();
                                println!("DEBUG: Private key has {} lines", lines.len());
                                if let Some(first_line) = lines.first() {
                                    println!("DEBUG: Key starts with: {}", first_line);
                                }
                                if let Some(last_line) = lines.last() {
                                    println!("DEBUG: Key ends with: {}", last_line);
                                }
                            }
                            Err(e) => {
                                println!("DEBUG: Failed to read private key: {}", e);
                            }
                        }
                        
                        println!("DEBUG: Session authenticated before attempt: {}", session.authenticated());
                        
                        // Try different authentication methods
                        
                        // Method 1: Try with public key file if available
                        if pub_key_exists {
                            match session.userauth_pubkey_file(&app_session.username, Some(Path::new(&pub_key_path)), Path::new(&expanded_path), None) {
                                Ok(_) => {
                                    println!("DEBUG: Public key file authentication successful!");
                                    println!("DEBUG: Session authenticated after success: {}", session.authenticated());
                                    self.emit_status(&session_id, "connecting", Some("✅ Public key file authentication successful"))?;
                                    auth_success = true;
                                }
                                Err(e) => {
                                    println!("DEBUG: Public key file authentication failed: {}", e);
                                    println!("DEBUG: Session authenticated after failure: {}", session.authenticated());
                                }
                            }
                        }
                        
                        // Method 2: Try without public key file (let SSH2 derive it)
                        if !auth_success {
                            match session.userauth_pubkey_file(&app_session.username, None, Path::new(&expanded_path), None) {
                                Ok(_) => {
                                    println!("DEBUG: Private key only authentication successful!");
                                    println!("DEBUG: Session authenticated after success: {}", session.authenticated());
                                    self.emit_status(&session_id, "connecting", Some("✅ Private key only authentication successful"))?;
                                    auth_success = true;
                                }
                                Err(e) => {
                                    println!("DEBUG: Private key only authentication failed: {}", e);
                                    println!("DEBUG: Session authenticated after failure: {}", session.authenticated());
                                }
                            }
                        }
                        
                        // Method 3: Try with empty passphrase explicitly
                        if !auth_success {
                            match session.userauth_pubkey_file(&app_session.username, None, Path::new(&expanded_path), Some("")) {
                                Ok(_) => {
                                    println!("DEBUG: Empty passphrase authentication successful!");
                                    println!("DEBUG: Session authenticated after success: {}", session.authenticated());
                                    self.emit_status(&session_id, "connecting", Some("✅ Empty passphrase authentication successful"))?;
                                    auth_success = true;
                                }
                                Err(e) => {
                                    println!("DEBUG: Empty passphrase authentication failed: {}", e);
                                    println!("DEBUG: Session authenticated after failure: {}", session.authenticated());
                                }
                            }
                        }
                    }
                } else {
                    println!("DEBUG: Primary key file not found: {}", expanded_path);
                    self.emit_status(&session_id, "connecting", Some(&format!("❌ Primary key file not found: {}", expanded_path)))?;
                }
                
                // If specified key fails, try SSH agent authentication first
                if !auth_success {
                    // Try SSH agent authentication as fallback
                    println!("DEBUG: Attempting SSH agent authentication");
                    match session.userauth_agent(&app_session.username) {
                        Ok(_) => {
                            println!("DEBUG: SSH agent authentication successful!");
                            println!("DEBUG: Session authenticated after agent success: {}", session.authenticated());
                            self.emit_status(&session_id, "connecting", Some("✅ SSH agent authentication successful"))?;
                            auth_success = true;
                        }
                        Err(e) => {
                            println!("DEBUG: SSH agent authentication failed: {}", e);
                            println!("DEBUG: Session authenticated after agent failure: {}", session.authenticated());
                        }
                    }
                }
                
                // If specified key and agent fail, try common key locations as fallback
                if !auth_success {
                    println!("DEBUG: Primary key failed, trying fallback keys");
                    self.emit_status(&session_id, "connecting", Some("🔄 Trying fallback SSH keys"))?;
                    let home = get_home_directory();
                    
                    // Prioritize id_rsa first since it worked in manual testing
                    #[cfg(target_os = "windows")]
                    let common_keys = vec![
                        format!("{}\\.ssh\\id_rsa", home),
                        format!("{}\\.ssh\\id_ecdsa", home), 
                        format!("{}\\.ssh\\id_ed25519", home),
                    ];
                    #[cfg(not(target_os = "windows"))]
                    let common_keys = vec![
                        format!("{}/.ssh/id_rsa", home),
                        format!("{}/.ssh/id_ecdsa", home),
                        format!("{}/.ssh/id_ed25519", home),
                    ];
                    
                    for fallback_key in common_keys {
                        println!("DEBUG: Checking fallback key: {}", fallback_key);
                        if std::path::Path::new(&fallback_key).exists() {
                            let key_name = get_filename_from_path(&fallback_key);
                            println!("DEBUG: Found fallback key: {}", key_name);
                            self.emit_status(&session_id, "connecting", Some(&format!("🔑 Trying fallback key: {}", key_name)))?;
                            
                            println!("DEBUG: Session authenticated before fallback attempt: {}", session.authenticated());
                            match session.userauth_pubkey_file(&app_session.username, None, Path::new(&fallback_key), None) {
                                Ok(_) => {
                                    println!("DEBUG: Fallback key {} authentication successful!", key_name);
                                    println!("DEBUG: Session authenticated after success: {}", session.authenticated());
                                    self.emit_status(&session_id, "connecting", Some(&format!("✅ Fallback key authentication successful: {}", key_name)))?;
                                    auth_success = true;
                                    break;
                                }
                                Err(e) => {
                                    println!("DEBUG: Fallback key {} failed: {}", key_name, e);
                                    println!("DEBUG: Session authenticated after failure: {}", session.authenticated());
                                    self.emit_status(&session_id, "connecting", Some(&format!("❌ Fallback key failed: {}", key_name)))?;
                                }
                            }
                        } else {
                            println!("DEBUG: Fallback key not found: {}", fallback_key);
                            self.emit_status(&session_id, "connecting", Some(&format!("⚠️ Fallback key not found: {}", get_filename_from_path(&fallback_key))))?;
                        }
                    }
                }
                
                if !auth_success {
                    #[cfg(target_os = "windows")]
                    let fallback_paths = format!("{}\\.ssh\\id_rsa, {}\\.ssh\\id_ecdsa, {}\\.ssh\\id_ed25519", 
                        get_home_directory(), get_home_directory(), get_home_directory());
                    #[cfg(not(target_os = "windows"))]
                    let fallback_paths = "~/.ssh/id_rsa, ~/.ssh/id_ecdsa, ~/.ssh/id_ed25519".to_string();
                    
                    return Err(anyhow!("Public key authentication failed. Tried:\n1. Specified key: {}\n2. Fallback keys: {}\n\nPlease check:\n1. Key files exist and have correct permissions (600)\n2. Key is in correct format\n3. Key matches server's authorized_keys\n\nDebugging info: Check console for detailed SSH2 messages", expanded_path, fallback_paths));
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

        // Set environment variables for proper terminal behavior
        self.emit_status(&session_id, "connecting", Some("🌐 Setting up terminal environment"))?;
        let _ = channel.setenv("TERM", "xterm-256color");
        let _ = channel.setenv("LC_ALL", "en_US.UTF-8");
        let _ = channel.setenv("LANG", "en_US.UTF-8");
        
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
            reader_started: Arc::new(Mutex::new(false)),
        };

        {
            let mut connections = self.connections.lock().unwrap();
            connections.insert(session_id.clone(), ssh_connection.clone());
        }

        // Send initial setup commands to ensure proper terminal behavior
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Send a command to set up proper terminal echo and line discipline
        // {
            // if let Ok(mut channel_guard) = ssh_connection.channel.lock() {
                // if let Some(ref mut ch) = *channel_guard {
                    //  // Send stty settings to ensure proper echo behavior
                    // let setup_cmd = "stty echo icanon icrnl\n";
                    // let _ = ch.write_all(setup_cmd.as_bytes());
                    // let _ = ch.flush();
                // }
            // }
        // }
        
        // Wait a bit and send a newline to get the initial prompt
        tokio::time::sleep(Duration::from_millis(200)).await;
        {
            if let Ok(mut channel_guard) = ssh_connection.channel.lock() {
                if let Some(ref mut ch) = *channel_guard {
                    let _ = ch.write_all(b"\n");
                    let _ = ch.flush();
                }
            }
        }

        self.emit_status(&session_id, "connecting", Some("✅ Connection stored successfully"))?;

        // Start terminal reader after initial setup
        let ssh_connection_for_init = ssh_connection.clone();
        let app_handle_clone = self.app_handle.clone();
        
        // Check if reader is already started
        {
            let mut reader_started = ssh_connection_for_init.reader_started.lock().unwrap();
            if *reader_started {
                println!("DEBUG: Terminal reader already started for session {}", session_id);
            } else {
                *reader_started = true;
                println!("DEBUG: Starting NEW terminal reader for session {}", session_id);
                
                let connection_for_reader = ssh_connection_for_init.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                    
                    // Start terminal reader after initial setup
                    let session_id_clone = connection_for_reader.session_id.clone();
                    let mut buffer = [0u8; 4096];
                    println!("DEBUG: Terminal reader loop started for session {}", session_id_clone);
                    loop {
                        let read_result = tokio::task::spawn_blocking({
                            let connection_clone = connection_for_reader.clone();
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
                            Ok(Ok((0, _))) => {
                                println!("DEBUG: EOF received for session {}", session_id_clone);
                                break;
                            }
                            Ok(Ok((n, buffer))) => {
                                let output = String::from_utf8_lossy(&buffer[..n]).to_string();
                                println!("DEBUG: Emitting {} bytes for session {}: {:?}", n, session_id_clone, output);
                                let _ = app_handle_clone.emit("terminal_output", serde_json::json!({
                                    "session_id": session_id_clone,
                                    "data": output
                                }));
                            }
                            Ok(Err(e)) => {
                                println!("DEBUG: Read error for session {}: {}", session_id_clone, e);
                                tokio::time::sleep(Duration::from_millis(100)).await;
                            }
                            Err(e) => {
                                println!("DEBUG: Task error for session {}: {}", session_id_clone, e);
                                tokio::time::sleep(Duration::from_millis(100)).await;
                            }
                        }
                        
                        tokio::time::sleep(Duration::from_millis(50)).await;
                    }
                    println!("DEBUG: Terminal reader exited for session {}", session_id_clone);
                });
            }
        }

        // Emit connected status
        self.emit_status(&session_id, "connected", Some("🎉 SSH connection established successfully!"))?;
        self.emit_status(&session_id, "connected", Some(&format!("Ready to use terminal on {}@{}", app_session.username, app_session.host)))?;

        Ok(format!("Connected to {}@{}", app_session.username, app_session.host))
    }

    pub async fn disconnect(&self, session_id: &str) -> Result<()> {
        let mut connections = self.connections.lock().unwrap();
        if let Some(connection) = connections.remove(session_id) {
            // Reset reader started flag
            if let Ok(mut reader_started) = connection.reader_started.lock() {
                *reader_started = false;
            }
            
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
        println!("DEBUG: Sending input to session {}: {:?}", session_id, input);
        let connections = self.connections.lock().unwrap();
        if let Some(connection) = connections.get(session_id) {
            if let Ok(mut channel_guard) = connection.channel.lock() {
                if let Some(ref mut channel) = *channel_guard {
                    println!("DEBUG: Writing {} bytes to channel", input.len());
                    channel.write_all(input.as_bytes())
                        .map_err(|e| anyhow!("Failed to send input: {}", e))?;
                    channel.flush()
                        .map_err(|e| anyhow!("Failed to flush input: {}", e))?;
                    println!("DEBUG: Input sent and flushed successfully");
                } else {
                    println!("DEBUG: No channel available for session {}", session_id);
                }
            } else {
                println!("DEBUG: Failed to lock channel for session {}", session_id);
            }
        } else {
            println!("DEBUG: No connection found for session {}", session_id);
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
