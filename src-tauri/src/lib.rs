use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

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
pub struct ConnectionStatus {
    pub session_id: String,
    pub status: String,
    pub message: Option<String>,
}

// Create session request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub private_key_path: Option<String>,
    pub protocol: Protocol,
    pub auth_method: AuthMethod,
}

// Terminal input/output structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalInput {
    pub session_id: String,
    pub data: String,
}

// File info structure for SFTP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub size: u64,
    pub modified: Option<String>,
}

// Application state
pub struct AppState {
    pub sessions: Mutex<HashMap<String, Session>>,
    pub active_connections: Mutex<HashMap<String, ConnectionStatus>>,
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
async fn create_session(
    request: CreateSessionRequest,
    state: State<'_, AppState>,
) -> Result<Session, String> {
    let session_id = Uuid::new_v4().to_string();
    
    let session = Session {
        id: session_id.clone(),
        name: request.name,
        host: request.host,
        port: request.port,
        username: request.username,
        auth_method: request.auth_method,
        protocol: request.protocol,
        created_at: chrono::Utc::now().to_rfc3339(),
        last_used: None,
    };

    let mut sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    sessions.insert(session_id.clone(), session.clone());

    Ok(session)
}

#[tauri::command]
async fn update_session(
    session: Session,
    state: State<'_, AppState>,
) -> Result<Session, String> {
    let mut sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    sessions.insert(session.id.clone(), session.clone());
    Ok(session)
}

#[tauri::command]
async fn delete_session(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    sessions.remove(&session_id);
    Ok(())
}

#[tauri::command]
async fn connect_ssh(
    session_id: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<String, String> {
    // Get session info
    let (username, host) = {
        let sessions = state.sessions.lock().map_err(|e| e.to_string())?;
        let session = sessions.get(&session_id)
            .ok_or_else(|| "Session not found".to_string())?;
        (session.username.clone(), session.host.clone())
    };
    
    // Update connection status
    {
        let mut connections = state.active_connections.lock().map_err(|e| e.to_string())?;
        connections.insert(session_id.clone(), ConnectionStatus {
            session_id: session_id.clone(),
            status: "connecting".to_string(),
            message: Some(format!("Connecting to {}@{}", username, host)),
        });
    }
    
    // Emit connection status event
    app.emit("connection_status", ConnectionStatus {
        session_id: session_id.clone(),
        status: "connecting".to_string(),
        message: Some(format!("Connecting to {}@{}", username, host)),
    }).map_err(|e| e.to_string())?;
    
    // Simulate connection process
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    // Update to connected status
    {
        let mut connections = state.active_connections.lock().map_err(|e| e.to_string())?;
        connections.insert(session_id.clone(), ConnectionStatus {
            session_id: session_id.clone(),
            status: "connected".to_string(),
            message: Some("Connection established".to_string()),
        });
    }
    
    app.emit("connection_status", ConnectionStatus {
        session_id: session_id.clone(),
        status: "connected".to_string(),
        message: Some("Connection established".to_string()),
    }).map_err(|e| e.to_string())?;
    
    Ok(format!("Connected to session {}", session_id))
}

#[tauri::command]
async fn disconnect_session(
    session_id: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let mut connections = state.active_connections.lock().map_err(|e| e.to_string())?;
    connections.remove(&session_id);
    
    app.emit("connection_status", ConnectionStatus {
        session_id: session_id.clone(),
        status: "disconnected".to_string(),
        message: Some("Connection closed".to_string()),
    }).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
async fn send_terminal_input(
    input: TerminalInput,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    // For now, just log the input
    println!("Terminal input for {}: {}", input.session_id, input.data);
    Ok(())
}

#[tauri::command]
async fn get_connection_status(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Option<ConnectionStatus>, String> {
    let connections = state.active_connections.lock().map_err(|e| e.to_string())?;
    Ok(connections.get(&session_id).cloned())
}

#[tauri::command]
async fn create_sftp_session(
    session_id: String,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    // Simulate SFTP session creation
    println!("Creating SFTP session for {}", session_id);
    Ok(())
}

#[tauri::command]
async fn list_remote_directory(
    session_id: String,
    path: String,
    _state: State<'_, AppState>,
) -> Result<Vec<FileInfo>, String> {
    // Simulate file listing
    println!("Listing directory {} for session {}", path, session_id);
    
    let files = vec![
        FileInfo {
            name: "..".to_string(),
            path: "/home".to_string(),
            is_directory: true,
            size: 0,
            modified: Some("2024-01-29 12:00".to_string()),
        },
        FileInfo {
            name: "documents".to_string(),
            path: format!("{}/documents", path),
            is_directory: true,
            size: 0,
            modified: Some("2024-01-29 12:00".to_string()),
        },
        FileInfo {
            name: "file1.txt".to_string(),
            path: format!("{}/file1.txt", path),
            is_directory: false,
            size: 1024,
            modified: Some("2024-01-29 12:00".to_string()),
        },
        FileInfo {
            name: "file2.txt".to_string(),
            path: format!("{}/file2.txt", path),
            is_directory: false,
            size: 2048,
            modified: Some("2024-01-29 12:00".to_string()),
        },
    ];
    
    Ok(files)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState {
        sessions: Mutex::new(HashMap::new()),
        active_connections: Mutex::new(HashMap::new()),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            list_sessions,
            create_session,
            update_session,
            delete_session,
            connect_ssh,
            disconnect_session,
            send_terminal_input,
            get_connection_status,
            create_sftp_session,
            list_remote_directory
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
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
async fn create_session(
    request: CreateSessionRequest,
    state: State<'_, AppState>,
) -> Result<Session, String> {
    let session_id = Uuid::new_v4().to_string();
    
    // Create legacy session for storage
    let session = Session {
        id: session_id.clone(),
        name: request.name.clone(),
        host: request.host.clone(),
        port: request.port,
        username: request.username.clone(),
        auth_method: request.auth_method.clone(),
        protocol: request.protocol.clone(),
        created_at: chrono::Utc::now().to_rfc3339(),
        last_used: None,
    };

    // Create SSH config for session manager
    let session_type = match request.protocol {
        Protocol::SSH | Protocol::SFTP => {
            let auth_method = match request.auth_method {
                AuthMethod::Password => {
                    if let Some(password) = request.password {
                        AuthenticationMethod::Password(password)
                    } else {
                        return Err("Password required for password authentication".to_string());
                    }
                }
                AuthMethod::PublicKey { key_path } => {
                    AuthenticationMethod::PublicKey {
                        private_key_path: key_path,
                        passphrase: None,
                    }
                }
                AuthMethod::Agent => AuthenticationMethod::Agent,
            };

            SessionType::Ssh(SshConfig {
                host: request.host,
                port: request.port,
                username: request.username,
                auth_method,
            })
        }
        Protocol::Local => SessionType::Local { shell: None },
    };

    let session_info = SessionInfo {
        id: session_id.clone(),
        name: request.name,
        session_type,
        status: SessionStatus::Disconnected,
        created_at: chrono::Utc::now().to_rfc3339(),
        last_connected: None,
    };

    // Add to session manager
    state.session_manager.create_session(session_info).await
        .map_err(|e| e.to_string())?;

    // Add to legacy storage
    let mut sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    sessions.insert(session_id.clone(), session.clone());

    Ok(session)
}

#[tauri::command]
async fn update_session(
    session: Session,
    state: State<'_, AppState>,
) -> Result<Session, String> {
    let mut sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    sessions.insert(session.id.clone(), session.clone());
    Ok(session)
}

#[tauri::command]
async fn delete_session(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Remove from session manager
    state.session_manager.remove_session(&session_id)
        .map_err(|e| e.to_string())?;

    // Remove from legacy storage
    let mut sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    sessions.remove(&session_id);
    Ok(())
}

#[tauri::command]
async fn connect_ssh(
    session_id: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<String, String> {
    // Update connection status
    {
        let mut connections = state.active_connections.lock().map_err(|e| e.to_string())?;
        connections.insert(session_id.clone(), ConnectionStatus {
            session_id: session_id.clone(),
            status: "connecting".to_string(),
            message: Some("Establishing connection...".to_string()),
        });
    }
    
    // Emit connection status event
    app.emit("connection_status", ConnectionStatus {
        session_id: session_id.clone(),
        status: "connecting".to_string(),
        message: Some("Establishing connection...".to_string()),
    }).map_err(|e| e.to_string())?;
    
    // Connect using session manager
    match state.session_manager.connect_session(&session_id).await {
        Ok(_) => {
            // Update to connected status
            {
                let mut connections = state.active_connections.lock().map_err(|e| e.to_string())?;
                connections.insert(session_id.clone(), ConnectionStatus {
                    session_id: session_id.clone(),
                    status: "connected".to_string(),
                    message: Some("Connection established".to_string()),
                });
            }
            
            app.emit("connection_status", ConnectionStatus {
                session_id: session_id.clone(),
                status: "connected".to_string(),
                message: Some("Connection established".to_string()),
            }).map_err(|e| e.to_string())?;
            
            Ok(format!("Connected to session {}", session_id))
        }
        Err(e) => {
            // Update to error status
            {
                let mut connections = state.active_connections.lock().map_err(|e| e.to_string())?;
                connections.insert(session_id.clone(), ConnectionStatus {
                    session_id: session_id.clone(),
                    status: "error".to_string(),
                    message: Some(e.to_string()),
                });
            }
            
            app.emit("connection_status", ConnectionStatus {
                session_id: session_id.clone(),
                status: "error".to_string(),
                message: Some(e.to_string()),
            }).map_err(|e| e.to_string())?;
            
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn disconnect_session(
    session_id: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    // Disconnect using session manager
    state.session_manager.disconnect_session(&session_id).await
        .map_err(|e| e.to_string())?;

    let mut connections = state.active_connections.lock().map_err(|e| e.to_string())?;
    connections.remove(&session_id);
    
    app.emit("connection_status", ConnectionStatus {
        session_id: session_id.clone(),
        status: "disconnected".to_string(),
        message: Some("Connection closed".to_string()),
    }).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
async fn send_terminal_input(
    input: TerminalInput,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.session_manager.send_input(&input.session_id, input.data).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn resize_terminal(
    session_id: String,
    cols: u16,
    rows: u16,
    width: u16,
    height: u16,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let size = TerminalSize { cols, rows, width, height };
    state.session_manager.resize_terminal(&session_id, size).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_connection_status(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Option<ConnectionStatus>, String> {
    let connections = state.active_connections.lock().map_err(|e| e.to_string())?;
    Ok(connections.get(&session_id).cloned())
}

// SSH Key Management Commands
#[tauri::command]
async fn list_ssh_keys(
    state: State<'_, AppState>,
) -> Result<Vec<SshKey>, String> {
    state.key_manager.list_keys()
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn generate_ssh_key(
    name: String,
    key_type: String,
    bits: Option<u32>,
    passphrase: Option<String>,
    state: State<'_, AppState>,
) -> Result<SshKey, String> {
    state.key_manager.generate_key(&name, &key_type, bits, passphrase.as_deref()).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_ssh_key(
    key_path: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.key_manager.delete_key(&key_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn test_ssh_key(
    key_path: String,
    passphrase: Option<String>,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    state.key_manager.test_key(&key_path, passphrase.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_public_key_content(
    key_path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    state.key_manager.get_public_key_content(&key_path)
        .map_err(|e| e.to_string())
}

// SFTP Commands
#[tauri::command]
async fn create_sftp_session(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.session_manager.create_sftp_session(&session_id).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_remote_directory(
    session_id: String,
    path: String,
    state: State<'_, AppState>,
) -> Result<Vec<FileInfo>, String> {
    state.session_manager.list_remote_directory(&session_id, &path).await
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let key_manager = KeyManager::new().expect("Failed to initialize key manager");
    let session_manager = SessionManager::new();
    
    let app_state = AppState {
        sessions: Mutex::new(HashMap::new()),
        active_connections: Mutex::new(HashMap::new()),
        session_manager,
        key_manager,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            list_sessions,
            create_session,
            update_session,
            delete_session,
            connect_ssh,
            disconnect_session,
            send_terminal_input,
            resize_terminal,
            get_connection_status,
            list_ssh_keys,
            generate_ssh_key,
            delete_ssh_key,
            test_ssh_key,
            get_public_key_content,
            create_sftp_session,
            list_remote_directory
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
