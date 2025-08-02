use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
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
            format!("{}\\.ssh\\id_rsa", userprofile)
        } else {
            "%USERPROFILE%\\.ssh\\id_rsa".to_string()
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        "~/.ssh/id_rsa".to_string()
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

    match state.ssh_manager.connect(session).await {
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
    state.ssh_manager.disconnect(&sessionId).await.map_err(|e| e.to_string())?;
    
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
    state.ssh_manager.send_input(&sessionId, &input).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn browse_ssh_key(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    
    let file_path = app.dialog().file()
        .set_title("Select SSH Private Key")
        .add_filter("SSH Keys", &["*"])
        .add_filter("All Files", &["*"])
        .blocking_pick_file();
    
    match file_path {
        Some(path) => Ok(Some(path.to_string())),
        None => Ok(None),
    }
}

#[tauri::command]
async fn list_remote_directory(
    state: State<'_, AppState>,
    #[allow(non_snake_case)] sessionId: String,
    path: String,
) -> Result<Vec<FileItem>, String> {
    state.ssh_manager.list_directory(&sessionId, &path).await.map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            app.manage(AppState::new(app_handle));
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
            browse_ssh_key
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
