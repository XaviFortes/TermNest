use anyhow::{anyhow, Result};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};

use crate::ssh::{
    client::{SshClient, SshConfig},
    terminal::{SshTerminal, LocalTerminal, TerminalSize},
    sftp::{SftpClient, FileInfo, TransferProgress},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub session_type: SessionType,
    pub status: SessionStatus,
    pub created_at: String,
    pub last_connected: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionType {
    Ssh(SshConfig),
    Local { shell: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

pub enum TerminalType {
    Ssh(SshTerminal),
    Local(LocalTerminal),
}

pub struct ActiveSession {
    pub info: SessionInfo,
    pub ssh_client: Option<SshClient>,
    pub terminal: Option<TerminalType>,
    pub sftp_client: Option<SftpClient>,
    pub output_receiver: Option<Receiver<String>>,
    pub input_sender: Option<Sender<String>>,
}

pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, Arc<RwLock<ActiveSession>>>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_session(&self, session_info: SessionInfo) -> Result<String> {
        let session_id = session_info.id.clone();
        
        let active_session = ActiveSession {
            info: session_info,
            ssh_client: None,
            terminal: None,
            sftp_client: None,
            output_receiver: None,
            input_sender: None,
        };

        let mut sessions = self.sessions.write();
        sessions.insert(session_id.clone(), Arc::new(RwLock::new(active_session)));

        Ok(session_id)
    }

    pub async fn connect_session(&self, session_id: &str) -> Result<()> {
        let sessions = self.sessions.read();
        let session_arc = sessions.get(session_id)
            .ok_or_else(|| anyhow!("Session not found"))?
            .clone();
        drop(sessions);

        let mut session = session_arc.write();
        
        // Update status to connecting
        session.info.status = SessionStatus::Connecting;

        match &session.info.session_type {
            SessionType::Ssh(ssh_config) => {
                // Create SSH client
                let mut ssh_client = SshClient::new(ssh_config.clone());
                ssh_client.connect().await.map_err(|e| {
                    session.info.status = SessionStatus::Error(e.to_string());
                    e
                })?;

                // Create terminal
                let channel = ssh_client.create_channel().await?;
                let (terminal, output_receiver, input_sender) = SshTerminal::new(
                    channel,
                    TerminalSize::default(),
                ).await?;

                session.ssh_client = Some(ssh_client);
                session.terminal = Some(TerminalType::Ssh(terminal));
                session.output_receiver = Some(output_receiver);
                session.input_sender = Some(input_sender);
            }
            SessionType::Local { shell } => {
                let (terminal, output_receiver, input_sender) = LocalTerminal::new(
                    TerminalSize::default(),
                    shell.clone(),
                )?;

                session.terminal = Some(TerminalType::Local(terminal));
                session.output_receiver = Some(output_receiver);
                session.input_sender = Some(input_sender);
            }
        }

        session.info.status = SessionStatus::Connected;
        session.info.last_connected = Some(chrono::Utc::now().to_rfc3339());

        Ok(())
    }

    pub async fn disconnect_session(&self, session_id: &str) -> Result<()> {
        let sessions = self.sessions.read();
        let session_arc = sessions.get(session_id)
            .ok_or_else(|| anyhow!("Session not found"))?
            .clone();
        drop(sessions);

        let mut session = session_arc.write();

        // Disconnect SSH client if present
        if let Some(mut ssh_client) = session.ssh_client.take() {
            let _ = ssh_client.disconnect().await;
        }

        // Close terminal
        match session.terminal.take() {
            Some(TerminalType::Ssh(terminal)) => {
                let _ = terminal.close().await;
            }
            Some(TerminalType::Local(mut terminal)) => {
                let _ = terminal.kill();
            }
            None => {}
        }

        session.sftp_client = None;
        session.output_receiver = None;
        session.input_sender = None;
        session.info.status = SessionStatus::Disconnected;

        Ok(())
    }

    pub fn get_session_info(&self, session_id: &str) -> Result<SessionInfo> {
        let sessions = self.sessions.read();
        let session = sessions.get(session_id)
            .ok_or_else(|| anyhow!("Session not found"))?;
        
        Ok(session.read().info.clone())
    }

    pub fn list_sessions(&self) -> Vec<SessionInfo> {
        let sessions = self.sessions.read();
        sessions.values()
            .map(|session| session.read().info.clone())
            .collect()
    }

    pub async fn send_input(&self, session_id: &str, data: String) -> Result<()> {
        let sessions = self.sessions.read();
        let session_arc = sessions.get(session_id)
            .ok_or_else(|| anyhow!("Session not found"))?
            .clone();
        drop(sessions);

        let session = session_arc.read();
        if let Some(ref input_sender) = session.input_sender {
            input_sender.try_send(data)
                .map_err(|e| anyhow!("Failed to send input: {}", e))?;
        }

        Ok(())
    }

    pub fn get_output_receiver(&self, session_id: &str) -> Result<Option<Receiver<String>>> {
        let sessions = self.sessions.read();
        let session = sessions.get(session_id)
            .ok_or_else(|| anyhow!("Session not found"))?;
        
        // This is tricky - we can't clone the receiver
        // In a real implementation, you'd want to use channels or events
        // For now, return None and implement a different pattern
        Ok(None)
    }

    pub async fn resize_terminal(&self, session_id: &str, size: TerminalSize) -> Result<()> {
        let sessions = self.sessions.read();
        let session_arc = sessions.get(session_id)
            .ok_or_else(|| anyhow!("Session not found"))?
            .clone();
        drop(sessions);

        let mut session = session_arc.write();
        match &mut session.terminal {
            Some(TerminalType::Ssh(terminal)) => {
                terminal.resize(size).await?;
            }
            Some(TerminalType::Local(terminal)) => {
                terminal.resize(size)?;
            }
            None => return Err(anyhow!("No active terminal")),
        }

        Ok(())
    }

    pub async fn create_sftp_session(&self, session_id: &str) -> Result<()> {
        let sessions = self.sessions.read();
        let session_arc = sessions.get(session_id)
            .ok_or_else(|| anyhow!("Session not found"))?
            .clone();
        drop(sessions);

        let mut session = session_arc.write();
        
        // Can only create SFTP for SSH sessions
        if let Some(ref mut ssh_client) = session.ssh_client {
            let channel = ssh_client.create_channel().await?;
            let sftp_client = SftpClient::new(channel).await?;
            session.sftp_client = Some(sftp_client);
            Ok(())
        } else {
            Err(anyhow!("SFTP is only available for SSH sessions"))
        }
    }

    pub async fn list_remote_directory(&self, session_id: &str, path: &str) -> Result<Vec<FileInfo>> {
        let sessions = self.sessions.read();
        let session = sessions.get(session_id)
            .ok_or_else(|| anyhow!("Session not found"))?;
        
        if let Some(ref sftp_client) = session.read().sftp_client {
            sftp_client.list_directory(path).await
        } else {
            Err(anyhow!("No SFTP session available"))
        }
    }

    pub fn remove_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write();
        sessions.remove(session_id)
            .ok_or_else(|| anyhow!("Session not found"))?;
        Ok(())
    }
}
