use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

mod ssh_new;
use ssh_new::SshManager;

// Session data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_method: AuthMethod,
    pub protocol: Protocol,
    pub created_at: String,
    pub last_used: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    Password,
    PublicKey { key_path: String },
    Agent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Protocol {
    SSH,
    SFTP,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Connecting,
    Error(String),
}

// File item for SFTP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileItem {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_directory: bool,
    pub modified: String,
}

// Application state
pub struct AppState {
    pub sessions: Mutex<HashMap<String, Session>>,
    pub active_connections: Mutex<HashMap<String, ConnectionStatus>>,
    pub ssh_manager: std::sync::Arc<SshManager>,
}

impl AppState {
    pub fn new(_app_handle: AppHandle) -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            active_connections: Mutex::new(HashMap::new()),
            ssh_manager: std::sync::Arc::new(SshManager::new()),
        }
    }
}

// Tauri commands
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn list_sessions(state: State<'_, AppState>) -> Result<Vec<Session>, String> {
    let sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    Ok(sessions.values().cloned().collect())
}

#[tauri::command]
async fn load_sessions_from_store(app: AppHandle, state: State<'_, AppState>) -> Result<Vec<Session>, String> {
    use tauri_plugin_store::StoreExt;
    
    let store = app.store("sessions.json").map_err(|e| e.to_string())?;
    
    if let Some(sessions_value) = store.get("sessions") {
        let sessions: Vec<Session> = serde_json::from_value(sessions_value.clone())
            .map_err(|e| e.to_string())?;
        
        // Load into state
        let mut state_sessions = state.sessions.lock().map_err(|e| e.to_string())?;
        for session in &sessions {
            state_sessions.insert(session.id.clone(), session.clone());
        }
        
        Ok(sessions)
    } else {
        Ok(vec![])
    }
}

async fn save_sessions_to_store(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;
    
    let sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    let sessions_vec: Vec<Session> = sessions.values().cloned().collect();
    
    let store = app.store("sessions.json").map_err(|e| e.to_string())?;
    store.set("sessions", serde_json::to_value(sessions_vec).map_err(|e| e.to_string())?);
    store.save().map_err(|e| e.to_string())?;
    
    Ok(())
}

fn get_default_ssh_key_path() -> String {
    #[cfg(target_os = "windows")]
    {
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            format!("{}\\.ssh\\id_ed25519", userprofile)
        } else {
            "%USERPROFILE%\\.ssh\\id_ed25519".to_string()
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(home) = std::env::var("HOME") {
            format!("{}/.ssh/id_ed25519", home)
        } else {
            "~/.ssh/id_ed25519".to_string()
        }
    }
}

#[tauri::command]
async fn create_session(
    state: State<'_, AppState>,
    app: AppHandle,
    name: String,
    host: String,
    port: u16,
    username: String,
    protocol: Protocol,
) -> Result<Session, String> {
    // Generate a unique session ID
    let session_id = Uuid::new_v4().to_string();
    
    // Get default SSH key path based on platform
    let default_key_path = get_default_ssh_key_path();
    
    let session = Session {
        id: session_id.clone(),
        name,
        host,
        port,
        username,
        protocol,
        auth_method: AuthMethod::PublicKey { key_path: default_key_path },
        created_at: chrono::Utc::now().to_rfc3339(),
        last_used: None,
    };

    // Insert session and drop guard before await
    {
        let mut sessions = state.sessions.lock().map_err(|e| e.to_string())?;
        sessions.insert(session_id.clone(), session.clone());
    }

    save_sessions_to_store(app, state).await?;
    Ok(session)
}

#[tauri::command]
async fn update_session(
    state: State<'_, AppState>,
    app: AppHandle,
    session: Session,
) -> Result<Session, String> {
    // Update session and drop guard before await
    {
        let mut sessions = state.sessions.lock().map_err(|e| e.to_string())?;
        sessions.insert(session.id.clone(), session.clone());
    }

    save_sessions_to_store(app, state).await?;
    Ok(session)
}

#[tauri::command]
async fn delete_session(
    state: State<'_, AppState>,
    app: AppHandle,
    #[allow(non_snake_case)] sessionId: String,
) -> Result<(), String> {
    // Remove session and drop guard before await
    {
        let mut sessions = state.sessions.lock().map_err(|e| e.to_string())?;
        sessions.remove(&sessionId);
    }

    save_sessions_to_store(app, state).await?;
    Ok(())
}

#[tauri::command]
async fn connect_ssh(
    state: State<'_, AppState>,
    app: AppHandle,
    #[allow(non_snake_case)] sessionId: String,
) -> Result<(), String> {
    let session = {
        let sessions = state.sessions.lock().map_err(|e| e.to_string())?;
        sessions.get(&sessionId).cloned().ok_or("Session not found")?
    };

    // Update connection status
    {
        let mut connections = state.active_connections.lock().map_err(|e| e.to_string())?;
        connections.insert(sessionId.clone(), ConnectionStatus::Connecting);
    }

    // Convert session to SSH config
    let config = ssh_new::SshConfig {
        host: session.host,
        port: session.port,
        username: session.username,
        auth_method: match session.auth_method {
            AuthMethod::Password => ssh_new::AuthMethod::Password { password: String::new() },
            AuthMethod::PublicKey { key_path } => ssh_new::AuthMethod::PublicKey { private_key_path: key_path },
            AuthMethod::Agent => ssh_new::AuthMethod::Agent,
        },
    };

    match state.ssh_manager.connect(sessionId.clone(), config, app) {
        Ok(_) => {
            let mut connections = state.active_connections.lock().map_err(|e| e.to_string())?;
            connections.insert(sessionId, ConnectionStatus::Connected);
            Ok(())
        }
        Err(e) => {
            let mut connections = state.active_connections.lock().map_err(|e| e.to_string())?;
            connections.insert(sessionId, ConnectionStatus::Error(e.to_string()));
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn disconnect_session(
    state: State<'_, AppState>,
    #[allow(non_snake_case)] sessionId: String,
) -> Result<(), String> {
    state.ssh_manager.disconnect(&sessionId).map_err(|e| e.to_string())?;
    
    let mut connections = state.active_connections.lock().map_err(|e| e.to_string())?;
    connections.insert(sessionId, ConnectionStatus::Disconnected);
    
    Ok(())
}

#[tauri::command]
async fn send_terminal_input(
    state: State<'_, AppState>,
    _app: AppHandle,
    #[allow(non_snake_case)] sessionId: String,
    input: String,
) -> Result<(), String> {
    state.ssh_manager.send_input(&sessionId, &input).map_err(|e| e.to_string())
}

#[tauri::command]
async fn browse_ssh_key(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    
    // Get the default SSH directory to start the dialog in
    let default_ssh_dir = {
        #[cfg(target_os = "windows")]
        {
            if let Ok(userprofile) = std::env::var("USERPROFILE") {
                format!("{}\\.ssh", userprofile)
            } else {
                "".to_string()
            }
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            if let Ok(home) = std::env::var("HOME") {
                format!("{}/.ssh", home)
            } else {
                "".to_string()
            }
        }
    };
    
    let mut dialog = app.dialog().file()
        .set_title("Select SSH Private Key");
        // .add_filter("SSH Private Keys", &["id_rsa", "id_ed25519", "id_ecdsa", "id_dsa"])
        // .add_filter("All Files", &["*"]);
    
    // Set initial directory to .ssh folder if it exists
    if !default_ssh_dir.is_empty() && std::path::Path::new(&default_ssh_dir).exists() {
        dialog = dialog.set_directory(&default_ssh_dir);
    }
    
    // On macOS, the dialog should show hidden files by default when starting in .ssh
    let file_path = dialog.blocking_pick_file();
    
    match file_path {
        Some(path) => Ok(Some(path.to_string())),
        None => Ok(None),
    }
}

#[tauri::command]
async fn list_remote_directory(
    state: State<'_, AppState>,
    #[allow(non_snake_case)] session_id: String,
    path: String,
) -> Result<Vec<FileItem>, String> {
    // Get the session configuration and clone it to avoid lifetime issues
    let session = {
        let sessions = state.sessions.lock().unwrap();
        sessions.get(&session_id)
            .ok_or_else(|| "Session not found".to_string())?
            .clone()
    };
    
    // Create SFTP connection using the session's configuration
    list_directory_sftp(&session.host, session.port, &session.username, &session.auth_method, &path).await
}

#[tauri::command]
async fn list_remote_directory_with_password(
    state: State<'_, AppState>,
    #[allow(non_snake_case)] session_id: String,
    path: String,
    password: String,
) -> Result<Vec<FileItem>, String> {
    // Get the session configuration and clone it to avoid lifetime issues
    let session = {
        let sessions = state.sessions.lock().unwrap();
        sessions.get(&session_id)
            .ok_or_else(|| "Session not found".to_string())?
            .clone()
    };
    
    // Use password authentication for SFTP
    let auth_method = AuthMethod::Password;
    list_directory_sftp_with_password(&session.host, session.port, &session.username, &auth_method, &path, &password).await
}

async fn list_directory_sftp(
    host: &str,
    port: u16,
    username: &str,
    auth_method: &AuthMethod,
    path: &str,
) -> Result<Vec<FileItem>, String> {
    use ssh2::{Session};
    use std::net::TcpStream;
    use std::path::Path;
    
    // Connect to SSH server
    let tcp = TcpStream::connect(format!("{}:{}", host, port))
        .map_err(|e| format!("Failed to connect: {}", e))?;
    
    let mut sess = Session::new()
        .map_err(|e| format!("Failed to create session: {}", e))?;
    
    sess.set_tcp_stream(tcp);
    sess.handshake()
        .map_err(|e| format!("SSH handshake failed: {}", e))?;
    
    // Authenticate
    match auth_method {
        AuthMethod::Password => {
            return Err("Password authentication requires interactive input for SFTP".to_string());
        }
        AuthMethod::PublicKey { key_path } => {
            sess.userauth_pubkey_file(username, None, Path::new(key_path), None)
                .map_err(|e| format!("Public key authentication failed: {}", e))?;
        }
        AuthMethod::Agent => {
            sess.userauth_agent(username)
                .map_err(|e| format!("Agent authentication failed: {}", e))?;
        }
    }
    
    if !sess.authenticated() {
        return Err("Authentication failed".to_string());
    }
    
    // Create SFTP channel
    let sftp = sess.sftp()
        .map_err(|e| format!("Failed to create SFTP channel: {}", e))?;
    
    // Read directory
    let remote_path = std::path::Path::new(path);
    let dir_entries = sftp.readdir(remote_path)
        .map_err(|e| format!("Failed to read directory: {}", e))?;
    
    let mut files = Vec::new();
    
    for (path_buf, stat) in dir_entries {
        let name = path_buf.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let full_path = path_buf.to_str().unwrap_or("").to_string();
        let is_directory = stat.is_dir();
        let size = if is_directory { 0 } else { stat.size.unwrap_or(0) };
        
        // Format modification time
        let modified = if let Some(mtime) = stat.mtime {
            let datetime = chrono::DateTime::from_timestamp(mtime as i64, 0)
                .unwrap_or_else(|| chrono::Utc::now());
            datetime.format("%Y-%m-%d %H:%M").to_string()
        } else {
            "unknown".to_string()
        };
        
        files.push(FileItem {
            name,
            path: full_path,
            size,
            is_directory,
            modified,
        });
    }
    
    // Add parent directory entry if we're not at root
    if path != "/" && path != "" {
        let parent_path = std::path::Path::new(path)
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("/")
            .to_string();
        
        files.insert(0, FileItem {
            name: "..".to_string(),
            path: parent_path,
            size: 0,
            is_directory: true,
            modified: "".to_string(),
        });
    }
    
    Ok(files)
}

async fn list_directory_sftp_with_password(
    host: &str,
    port: u16,
    username: &str,
    _auth_method: &AuthMethod,
    path: &str,
    password: &str,
) -> Result<Vec<FileItem>, String> {
    use ssh2::Session;
    use std::net::TcpStream;
    use std::path::Path;
    
    // Connect to SSH server
    let tcp = TcpStream::connect(format!("{}:{}", host, port))
        .map_err(|e| format!("Failed to connect: {}", e))?;
    
    let mut sess = Session::new()
        .map_err(|e| format!("Failed to create session: {}", e))?;
    
    sess.set_tcp_stream(tcp);
    sess.handshake()
        .map_err(|e| format!("SSH handshake failed: {}", e))?;
    
    // Authenticate with password
    sess.userauth_password(username, password)
        .map_err(|e| format!("Password authentication failed: {}", e))?;
    
    if !sess.authenticated() {
        return Err("Authentication failed".to_string());
    }
    
    // Create SFTP channel
    let sftp = sess.sftp()
        .map_err(|e| format!("Failed to create SFTP channel: {}", e))?;
    
    // Read directory
    let remote_path = std::path::Path::new(path);
    let dir_entries = sftp.readdir(remote_path)
        .map_err(|e| format!("Failed to read directory: {}", e))?;
    
    let mut files = Vec::new();
    
    for (path_buf, stat) in dir_entries {
        let name = path_buf.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let full_path = path_buf.to_str().unwrap_or("").to_string();
        let is_directory = stat.is_dir();
        let size = if is_directory { 0 } else { stat.size.unwrap_or(0) };
        
        // Format modification time
        let modified = if let Some(mtime) = stat.mtime {
            let datetime = chrono::DateTime::from_timestamp(mtime as i64, 0)
                .unwrap_or_else(|| chrono::Utc::now());
            datetime.format("%Y-%m-%d %H:%M").to_string()
        } else {
            "unknown".to_string()
        };
        
        files.push(FileItem {
            name,
            path: full_path,
            size,
            is_directory,
            modified,
        });
    }
    
    // Add parent directory entry if we're not at root
    if path != "/" && path != "" {
        let parent_path = std::path::Path::new(path)
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("/")
            .to_string();
        
        files.insert(0, FileItem {
            name: "..".to_string(),
            path: parent_path,
            size: 0,
            is_directory: true,
            modified: "".to_string(),
        });
    }
    
    Ok(files)
}

async fn download_file_sftp_with_password(
    host: &str,
    port: u16,
    username: &str,
    remote_path: &str,
    local_path: &str,
    password: &str,
) -> Result<String, String> {
    use std::net::TcpStream;
    use ssh2::Session;
    
    let tcp = TcpStream::connect(format!("{}:{}", host, port))
        .map_err(|e| format!("Failed to connect: {}", e))?;
    
    let mut sess = Session::new()
        .map_err(|e| format!("Failed to create session: {}", e))?;
    
    sess.set_tcp_stream(tcp);
    sess.handshake()
        .map_err(|e| format!("SSH handshake failed: {}", e))?;
    
    sess.userauth_password(username, password)
        .map_err(|e| format!("SSH authentication failed: {}", e))?;
    
    if !sess.authenticated() {
        return Err("Authentication failed".to_string());
    }
    
    let sftp = sess.sftp()
        .map_err(|e| format!("Failed to create SFTP session: {}", e))?;
    
    let mut remote_file = sftp.open(std::path::Path::new(remote_path))
        .map_err(|e| format!("Failed to open remote file: {}", e))?;
    
    let mut local_file = std::fs::File::create(local_path)
        .map_err(|e| format!("Failed to create local file: {}", e))?;
    
    std::io::copy(&mut remote_file, &mut local_file)
        .map_err(|e| format!("Failed to copy file: {}", e))?;
    
    Ok(format!("File downloaded successfully to: {}", local_path))
}

async fn delete_file_sftp_with_password(
    host: &str,
    port: u16,
    username: &str,
    remote_path: &str,
    password: &str,
) -> Result<String, String> {
    use std::net::TcpStream;
    use ssh2::Session;
    
    let tcp = TcpStream::connect(format!("{}:{}", host, port))
        .map_err(|e| format!("Failed to connect: {}", e))?;
    
    let mut sess = Session::new()
        .map_err(|e| format!("Failed to create session: {}", e))?;
    
    sess.set_tcp_stream(tcp);
    sess.handshake()
        .map_err(|e| format!("SSH handshake failed: {}", e))?;
    
    sess.userauth_password(username, password)
        .map_err(|e| format!("SSH authentication failed: {}", e))?;
    
    if !sess.authenticated() {
        return Err("Authentication failed".to_string());
    }
    
    let sftp = sess.sftp()
        .map_err(|e| format!("Failed to create SFTP session: {}", e))?;
    
    sftp.unlink(std::path::Path::new(remote_path))
        .map_err(|e| format!("Failed to delete file: {}", e))?;
    
    Ok(format!("File deleted successfully: {}", remote_path))
}

#[tauri::command]
async fn download_remote_file(
    state: State<'_, AppState>,
    #[allow(non_snake_case)] session_id: String,
    remote_path: String,
    local_path: String,
) -> Result<String, String> {
    // Get the session configuration and clone it to avoid lifetime issues
    let session = {
        let sessions = state.sessions.lock().unwrap();
        sessions.get(&session_id)
            .ok_or_else(|| "Session not found".to_string())?
            .clone()
    };
    
    download_file_sftp(&session.host, session.port, &session.username, &session.auth_method, &remote_path, &local_path).await
}

#[tauri::command]
async fn download_remote_file_with_password(
    state: State<'_, AppState>,
    #[allow(non_snake_case)] session_id: String,
    remote_path: String,
    local_path: String,
    password: String,
) -> Result<String, String> {
    // Get the session configuration and clone it to avoid lifetime issues
    let session = {
        let sessions = state.sessions.lock().unwrap();
        sessions.get(&session_id)
            .ok_or_else(|| "Session not found".to_string())?
            .clone()
    };
    
    download_file_sftp_with_password(&session.host, session.port, &session.username, &remote_path, &local_path, &password).await
}

async fn download_file_sftp(
    host: &str,
    port: u16,
    username: &str,
    auth_method: &AuthMethod,
    remote_path: &str,
    local_path: &str,
) -> Result<String, String> {
    use ssh2::Session;
    use std::fs::File;
    use std::io::copy;
    use std::net::TcpStream;
    use std::path::Path;
    
    // Connect to SSH server
    let tcp = TcpStream::connect(format!("{}:{}", host, port))
        .map_err(|e| format!("Failed to connect: {}", e))?;
    
    let mut sess = Session::new()
        .map_err(|e| format!("Failed to create session: {}", e))?;
    
    sess.set_tcp_stream(tcp);
    sess.handshake()
        .map_err(|e| format!("SSH handshake failed: {}", e))?;
    
    // Authenticate
    match auth_method {
        AuthMethod::Password => {
            return Err("Password authentication requires interactive input for SFTP".to_string());
        }
        AuthMethod::PublicKey { key_path } => {
            sess.userauth_pubkey_file(username, None, Path::new(key_path), None)
                .map_err(|e| format!("Public key authentication failed: {}", e))?;
        }
        AuthMethod::Agent => {
            sess.userauth_agent(username)
                .map_err(|e| format!("Agent authentication failed: {}", e))?;
        }
    }
    
    if !sess.authenticated() {
        return Err("Authentication failed".to_string());
    }
    
    // Create SFTP channel
    let sftp = sess.sftp()
        .map_err(|e| format!("Failed to create SFTP channel: {}", e))?;
    
    // Open remote file
    let mut remote_file = sftp.open(Path::new(remote_path))
        .map_err(|e| format!("Failed to open remote file: {}", e))?;
    
    // Create local file
    let mut local_file = File::create(local_path)
        .map_err(|e| format!("Failed to create local file: {}", e))?;
    
    // Copy data
    let bytes_copied = copy(&mut remote_file, &mut local_file)
        .map_err(|e| format!("Failed to copy data: {}", e))?;
    
    Ok(format!("Downloaded {} bytes to {}", bytes_copied, local_path))
}

#[tauri::command]
async fn delete_remote_file(
    state: State<'_, AppState>,
    #[allow(non_snake_case)] session_id: String,
    remote_path: String,
) -> Result<String, String> {
    // Get the session configuration and clone it to avoid lifetime issues
    let session = {
        let sessions = state.sessions.lock().unwrap();
        sessions.get(&session_id)
            .ok_or_else(|| "Session not found".to_string())?
            .clone()
    };
    
    delete_file_sftp(&session.host, session.port, &session.username, &session.auth_method, &remote_path).await
}

#[tauri::command]
async fn delete_remote_file_with_password(
    state: State<'_, AppState>,
    #[allow(non_snake_case)] session_id: String,
    remote_path: String,
    password: String,
) -> Result<String, String> {
    // Get the session configuration and clone it to avoid lifetime issues
    let session = {
        let sessions = state.sessions.lock().unwrap();
        sessions.get(&session_id)
            .ok_or_else(|| "Session not found".to_string())?
            .clone()
    };
    
    delete_file_sftp_with_password(&session.host, session.port, &session.username, &remote_path, &password).await
}

async fn delete_file_sftp(
    host: &str,
    port: u16,
    username: &str,
    auth_method: &AuthMethod,
    remote_path: &str,
) -> Result<String, String> {
    use ssh2::Session;
    use std::net::TcpStream;
    use std::path::Path;
    
    // Connect to SSH server
    let tcp = TcpStream::connect(format!("{}:{}", host, port))
        .map_err(|e| format!("Failed to connect: {}", e))?;
    
    let mut sess = Session::new()
        .map_err(|e| format!("Failed to create session: {}", e))?;
    
    sess.set_tcp_stream(tcp);
    sess.handshake()
        .map_err(|e| format!("SSH handshake failed: {}", e))?;
    
    // Authenticate
    match auth_method {
        AuthMethod::Password => {
            return Err("Password authentication requires interactive input for SFTP".to_string());
        }
        AuthMethod::PublicKey { key_path } => {
            sess.userauth_pubkey_file(username, None, Path::new(key_path), None)
                .map_err(|e| format!("Public key authentication failed: {}", e))?;
        }
        AuthMethod::Agent => {
            sess.userauth_agent(username)
                .map_err(|e| format!("Agent authentication failed: {}", e))?;
        }
    }
    
    if !sess.authenticated() {
        return Err("Authentication failed".to_string());
    }
    
    // Create SFTP channel
    let sftp = sess.sftp()
        .map_err(|e| format!("Failed to create SFTP channel: {}", e))?;
    
    // Check if it's a directory or file
    let stat = sftp.stat(Path::new(remote_path))
        .map_err(|e| format!("Failed to stat remote path: {}", e))?;
    
    if stat.is_dir() {
        // Remove directory
        sftp.rmdir(Path::new(remote_path))
            .map_err(|e| format!("Failed to remove directory: {}", e))?;
        Ok(format!("Directory {} removed successfully", remote_path))
    } else {
        // Remove file
        sftp.unlink(Path::new(remote_path))
            .map_err(|e| format!("Failed to remove file: {}", e))?;
        Ok(format!("File {} removed successfully", remote_path))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let ssh_manager = std::sync::Arc::new(ssh_new::SshManager::new());
            
            app.manage(AppState::new(app_handle));
            app.manage(ssh_manager);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            list_sessions,
            load_sessions_from_store,
            create_session,
            update_session,
            delete_session,
            connect_ssh,
            disconnect_session,
            send_terminal_input,
            list_remote_directory,
            list_remote_directory_with_password,
            download_remote_file,
            download_remote_file_with_password,
            delete_remote_file,
            delete_remote_file_with_password,
            browse_ssh_key,
            ssh_new::ssh_connect,
            ssh_new::ssh_connect_with_password,
            ssh_new::ssh_send_input,
            ssh_new::ssh_disconnect,
            ssh_new::ssh_list_sessions
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
