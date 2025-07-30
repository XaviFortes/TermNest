use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

mod ssh;
use ssh::SshManager;

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
}

// Terminal input structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalInput {
    pub session_id: String,
    pub data: String,
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
    pub ssh_manager: SshManager,
}

impl AppState {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            active_connections: Mutex::new(HashMap::new()),
            ssh_manager: SshManager::new(app_handle),
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
async fn create_session(
    name: String,
    host: String,
    port: u16,
    username: String,
    password: Option<String>,
    private_key_path: Option<String>,
    protocol: Protocol,
    state: State<'_, AppState>,
) -> Result<Session, String> {
    println!("Creating session: {} @ {}:{}", name, host, port);
    
    let session = Session {
        id: Uuid::new_v4().to_string(),
        name,
        host,
        port,
        username,
        auth_method: if password.is_some() {
            AuthMethod::Password
        } else if private_key_path.is_some() {
            AuthMethod::PublicKey { 
                key_path: private_key_path.unwrap_or_default() 
            }
        } else {
            AuthMethod::Agent
        },
        protocol,
        created_at: chrono::Utc::now().to_rfc3339(),
        last_used: None,
    };

    let mut sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    sessions.insert(session.id.clone(), session.clone());
    
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
    sessionId: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    sessions.remove(&sessionId);
    Ok(())
}

#[tauri::command]
async fn connect_ssh(
    sessionId: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<String, String> {
    // Get session details
    let session = {
        let sessions = state.sessions.lock().map_err(|e| e.to_string())?;
        sessions.get(&sessionId).cloned()
            .ok_or_else(|| "Session not found".to_string())?
    };

    // Use real SSH connection
    match state.ssh_manager.connect(session).await {
        Ok(result) => {
            // Update local connection status
            let mut connections = state.active_connections.lock().map_err(|e| e.to_string())?;
            connections.insert(sessionId.clone(), ConnectionStatus {
                session_id: sessionId.clone(),
                status: "connected".to_string(),
                message: Some("Connected".to_string()),
            });
            Ok(result)
        }
        Err(e) => {
            // Update connection status to failed
            let mut connections = state.active_connections.lock().map_err(|e| e.to_string())?;
            connections.insert(sessionId.clone(), ConnectionStatus {
                session_id: sessionId.clone(),
                status: "failed".to_string(),
                message: Some(e.to_string()),
            });
            
            // Emit failed status
            app.emit("connection_status", &ConnectionStatus {
                session_id: sessionId.clone(),
                status: "failed".to_string(),
                message: Some(e.to_string()),
            }).map_err(|e| e.to_string())?;
            
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn disconnect_session(
    sessionId: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    // Disconnect real SSH connection
    state.ssh_manager.disconnect(&sessionId).await.map_err(|e| e.to_string())?;

    // Remove from local active connections
    {
        let mut connections = state.active_connections.lock().map_err(|e| e.to_string())?;
        connections.remove(&sessionId);
    }

    Ok(())
}

#[tauri::command]
async fn send_terminal_input(
    input: TerminalInput,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.ssh_manager.send_input(&input.session_id, &input.data).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_connection_status(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Option<ConnectionStatus>, String> {
    // Check real SSH connection status first
    if let Some(status) = state.ssh_manager.get_connection_status(&session_id) {
        return Ok(Some(status));
    }
    
    // Fall back to local status
    let connections = state.active_connections.lock().map_err(|e| e.to_string())?;
    Ok(connections.get(&session_id).cloned())
}

#[tauri::command]
async fn list_remote_directory(
    sessionId: String,
    path: String,
    state: State<'_, AppState>,
) -> Result<Vec<FileItem>, String> {
    state.ssh_manager.list_directory(&sessionId, &path).await.map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            let app_handle = app.handle().clone();
            app.manage(AppState::new(app_handle));
            Ok(())
        })
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
            list_remote_directory
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
