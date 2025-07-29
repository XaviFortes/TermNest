use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use dirs;
use russh_keys::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKey {
    pub name: String,
    pub path: String,
    pub public_key_path: String,
    pub fingerprint: String,
    pub key_type: String,
    pub has_passphrase: bool,
    pub created_at: String,
}

pub struct KeyManager {
    ssh_dir: PathBuf,
}

impl KeyManager {
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
        let ssh_dir = home_dir.join(".ssh");
        
        // Create .ssh directory if it doesn't exist
        if !ssh_dir.exists() {
            fs::create_dir_all(&ssh_dir)?;
            // Set proper permissions (700)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&ssh_dir)?.permissions();
                perms.set_mode(0o700);
                fs::set_permissions(&ssh_dir, perms)?;
            }
        }

        Ok(Self { ssh_dir })
    }

    pub fn list_keys(&self) -> Result<Vec<SshKey>> {
        let mut keys = Vec::new();
        
        if !self.ssh_dir.exists() {
            return Ok(keys);
        }

        for entry in fs::read_dir(&self.ssh_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            // Skip if it's not a file
            if !path.is_file() {
                continue;
            }

            // Skip public keys and known_hosts, config files
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
                
            if file_name.ends_with(".pub") 
                || file_name == "known_hosts" 
                || file_name == "config"
                || file_name == "authorized_keys" {
                continue;
            }

            // Try to load as private key
            if let Ok(key_info) = self.analyze_key(&path) {
                keys.push(key_info);
            }
        }

        keys.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(keys)
    }

    fn analyze_key(&self, private_key_path: &Path) -> Result<SshKey> {
        let private_key_data = fs::read(private_key_path)?;
        
        // Try to determine if key has passphrase by attempting to decode
        let has_passphrase = match decode_secret_key(&private_key_data, None) {
            Ok(_) => false,
            Err(_) => true, // Assume it has a passphrase if decoding fails
        };

        // Get public key path
        let public_key_path = private_key_path.with_extension("pub");
        
        let (fingerprint, key_type) = if public_key_path.exists() {
            let public_key_data = fs::read_to_string(&public_key_path)?;
            self.parse_public_key(&public_key_data)?
        } else {
            // Try to extract public key from private key
            if !has_passphrase {
                if let Ok(key_pair) = decode_secret_key(&private_key_data, None) {
                    let public_key = key_pair.clone_public_key()?;
                    let fingerprint = self.calculate_fingerprint(&public_key)?;
                    let key_type = self.get_key_type(&public_key);
                    (fingerprint, key_type)
                } else {
                    ("unknown".to_string(), "unknown".to_string())
                }
            } else {
                ("encrypted".to_string(), "unknown".to_string())
            }
        };

        let metadata = fs::metadata(private_key_path)?;
        let created_at = metadata
            .created()
            .or_else(|_| metadata.modified())
            .map(|time| {
                let datetime: chrono::DateTime<chrono::Utc> = time.into();
                datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string()
            })
            .unwrap_or_else(|_| "unknown".to_string());

        Ok(SshKey {
            name: private_key_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            path: private_key_path.to_string_lossy().to_string(),
            public_key_path: public_key_path.to_string_lossy().to_string(),
            fingerprint,
            key_type,
            has_passphrase,
            created_at,
        })
    }

    fn parse_public_key(&self, public_key_data: &str) -> Result<(String, String)> {
        let parts: Vec<&str> = public_key_data.trim().split_whitespace().collect();
        if parts.len() < 2 {
            return Err(anyhow!("Invalid public key format"));
        }

        let key_type = parts[0].to_string();
        let key_data = parts[1];

        // Decode the base64 key data to calculate fingerprint
        let decoded = general_purpose::STANDARD.decode(key_data)?;
        let public_key = parse_public_key_base64(&decoded)?;
        let fingerprint = self.calculate_fingerprint(&public_key)?;

        Ok((fingerprint, key_type))
    }

    fn calculate_fingerprint(&self, public_key: &key::PublicKey) -> Result<String> {
        // Get the public key in SSH wire format
        let mut buffer = Vec::new();
        public_key.write_public_key_base64(&mut buffer)?;
        
        // Calculate SHA256 hash
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        buffer.hash(&mut hasher);
        let hash = hasher.finish();
        
        // Format as SHA256 fingerprint
        Ok(format!("SHA256:{:016x}", hash))
    }

    fn get_key_type(&self, public_key: &key::PublicKey) -> String {
        public_key.name().to_string()
    }

    pub async fn generate_key(
        &self,
        name: &str,
        key_type: &str,
        bits: Option<u32>,
        passphrase: Option<&str>,
    ) -> Result<SshKey> {
        let private_key_path = self.ssh_dir.join(name);
        let public_key_path = private_key_path.with_extension("pub");

        // Generate key pair
        let key_pair = match key_type.to_lowercase().as_str() {
            "rsa" => {
                let bits = bits.unwrap_or(2048);
                key::KeyPair::generate_rsa(bits, key::SignatureHash::SHA2_256)?
            }
            "ed25519" => {
                key::KeyPair::generate_ed25519()?
            }
            "ecdsa" => {
                // Default to P-256 curve
                key::KeyPair::generate_ecdsa()?
            }
            _ => return Err(anyhow!("Unsupported key type: {}", key_type)),
        };

        // Save private key
        let private_key_data = if let Some(pass) = passphrase {
            encode_pkcs8_pem_encrypted(&key_pair, pass)?
        } else {
            encode_pkcs8_pem(&key_pair)?
        };

        fs::write(&private_key_path, private_key_data)?;

        // Set proper permissions for private key (600)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&private_key_path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&private_key_path, perms)?;
        }

        // Save public key
        let public_key = key_pair.clone_public_key()?;
        let mut public_key_data = Vec::new();
        public_key.write_public_key_base64(&mut public_key_data)?;
        let public_key_string = format!("{} {} {}\n", 
            public_key.name(),
            String::from_utf8(public_key_data)?,
            name
        );
        
        fs::write(&public_key_path, public_key_string)?;

        // Create SshKey info
        let fingerprint = self.calculate_fingerprint(&public_key)?;
        let key_type = self.get_key_type(&public_key);
        
        Ok(SshKey {
            name: name.to_string(),
            path: private_key_path.to_string_lossy().to_string(),
            public_key_path: public_key_path.to_string_lossy().to_string(),
            fingerprint,
            key_type,
            has_passphrase: passphrase.is_some(),
            created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        })
    }

    pub fn delete_key(&self, key_path: &str) -> Result<()> {
        let private_key_path = Path::new(key_path);
        let public_key_path = private_key_path.with_extension("pub");

        // Remove private key
        if private_key_path.exists() {
            fs::remove_file(private_key_path)?;
        }

        // Remove public key
        if public_key_path.exists() {
            fs::remove_file(public_key_path)?;
        }

        Ok(())
    }

    pub fn test_key(&self, key_path: &str, passphrase: Option<&str>) -> Result<bool> {
        let key_data = fs::read(key_path)?;
        match decode_secret_key(&key_data, passphrase) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    pub fn get_public_key_content(&self, key_path: &str) -> Result<String> {
        let private_key_path = Path::new(key_path);
        let public_key_path = private_key_path.with_extension("pub");
        
        if public_key_path.exists() {
            Ok(fs::read_to_string(public_key_path)?)
        } else {
            Err(anyhow!("Public key file not found"))
        }
    }
}
