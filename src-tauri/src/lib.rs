use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

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
    // Future protocols
    RDP,
    VNC,
    Telnet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub session_id: String,
    pub status: String,
    pub message: Option<String>,
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
    session: Session,
    state: State<'_, AppState>,
) -> Result<Session, String> {
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
    // Get session info and clone necessary data
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
    
    // TODO: Implement actual SSH connection logic
    // For now, we'll simulate a connection
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
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
async fn get_connection_status(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<Option<ConnectionStatus>, String> {
    let connections = state.active_connections.lock().map_err(|e| e.to_string())?;
    Ok(connections.get(&session_id).cloned())
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
            get_connection_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
