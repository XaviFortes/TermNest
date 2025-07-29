use anyhow::{anyhow, Result};
use russh::*;
use russh_keys::*;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_method: AuthenticationMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthenticationMethod {
    Password(String),
    PublicKey { 
        private_key_path: String, 
        passphrase: Option<String> 
    },
    Agent,
}

pub struct SshClient {
    session: Option<russh::client::Handle<SshClientHandler>>,
    config: SshConfig,
}

pub struct SshClientHandler;

#[async_trait::async_trait]
impl client::Handler for SshClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &key::PublicKey,
    ) -> Result<bool, Self::Error> {
        // TODO: Implement proper host key verification
        Ok(true)
    }
}

impl SshClient {
    pub fn new(config: SshConfig) -> Self {
        Self {
            session: None,
            config,
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        let config = client::Config {
            inactivity_timeout: Some(std::time::Duration::from_secs(5)),
            .Default::default()
        };

        let sh = SshClientHandler;
        
        let mut session = client::connect(config, (self.config.host.as_str(), self.config.port), sh).await?;
        
        // Authenticate
        match &self.config.auth_method {
            AuthenticationMethod::Password(password) => {
                let auth_res = session
                    .authenticate_password(self.config.username.clone(), password.clone())
                    .await?;
                if !auth_res {
                    return Err(anyhow!("Authentication failed"));
                }
            }
            AuthenticationMethod::PublicKey { private_key_path, passphrase } => {
                let key_pair = load_secret_key(private_key_path, passphrase.as_deref())?;
                let auth_res = session
                    .authenticate_publickey(self.config.username.clone(), Arc::new(key_pair))
                    .await?;
                if !auth_res {
                    return Err(anyhow!("Public key authentication failed"));
                }
            }
            AuthenticationMethod::Agent => {
                // TODO: Implement SSH agent authentication
                return Err(anyhow!("SSH agent authentication not yet implemented"));
            }
        }

        self.session = Some(session);
        Ok(())
    }

    pub async fn create_channel(&mut self) -> Result<russh::client::Channel<russh::client::Msg>> {
        let session = self.session.as_mut()
            .ok_or_else(|| anyhow!("Not connected"))?;
        
        let channel = session.channel_open_session().await?;
        Ok(channel)
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(session) = self.session.take() {
            session.disconnect(russh::Disconnect::ByApplication, "", "").await?;
        }
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.session.is_some()
    }
}

fn load_secret_key(path: &str, passphrase: Option<&str>) -> Result<key::KeyPair> {
    let key_data = std::fs::read(path)?;
    
    match passphrase {
        Some(pass) => decode_secret_key(&key_data, Some(pass)),
        None => decode_secret_key(&key_data, None),
    }
    .map_err(|e| anyhow!("Failed to load private key: {}", e))
}
