use anyhow::{anyhow, Result};
use russh_sftp::client::SftpSession;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub size: u64,
    pub modified: Option<String>,
    pub permissions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferProgress {
    pub total_bytes: u64,
    pub transferred_bytes: u64,
    pub percentage: f64,
    pub speed_bps: u64,
}

pub struct SftpClient {
    session: SftpSession,
}

impl SftpClient {
    pub async fn new(channel: russh::client::Channel<russh::client::Msg>) -> Result<Self> {
        let session = SftpSession::new(channel).await?;
        Ok(Self { session })
    }

    pub async fn list_directory(&self, path: &str) -> Result<Vec<FileInfo>> {
        let mut files = Vec::new();
        let mut dir = self.session.read_dir(path).await?;

        while let Some(entry) = dir.next().await {
            let entry = entry?;
            let file_name = entry.filename();
            let attrs = entry.attributes();

            let file_info = FileInfo {
                name: file_name.to_string(),
                path: format!("{}/{}", path.trim_end_matches('/'), file_name),
                is_directory: attrs.is_dir(),
                size: attrs.size().unwrap_or(0),
                modified: attrs.mtime().map(|t| {
                    chrono::DateTime::from_timestamp(t as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_default()
                }),
                permissions: attrs.permissions().map(|p| format!("{:o}", p)),
            };

            files.push(file_info);
        }

        files.sort_by(|a, b| {
            // Directories first, then by name
            match (a.is_directory, b.is_directory) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });

        Ok(files)
    }

    pub async fn download_file<P: AsRef<Path>>(
        &self,
        remote_path: &str,
        local_path: P,
        progress_callback: Option<Box<dyn Fn(TransferProgress) + Send + Sync>>,
    ) -> Result<()> {
        let mut remote_file = self.session.open(remote_path).await?;
        let attrs = remote_file.metadata().await?;
        let total_size = attrs.size().unwrap_or(0);

        let mut local_file = tokio::fs::File::create(local_path).await?;
        let mut buffer = vec![0u8; 32768]; // 32KB buffer
        let mut transferred = 0u64;
        let start_time = std::time::Instant::now();

        loop {
            let bytes_read = remote_file.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }

            local_file.write_all(&buffer[..bytes_read]).await?;
            transferred += bytes_read as u64;

            if let Some(ref callback) = progress_callback {
                let elapsed = start_time.elapsed().as_secs_f64();
                let speed = if elapsed > 0.0 {
                    (transferred as f64 / elapsed) as u64
                } else {
                    0
                };

                let progress = TransferProgress {
                    total_bytes: total_size,
                    transferred_bytes: transferred,
                    percentage: if total_size > 0 {
                        (transferred as f64 / total_size as f64) * 100.0
                    } else {
                        0.0
                    },
                    speed_bps: speed,
                };

                callback(progress);
            }
        }

        local_file.flush().await?;
        Ok(())
    }

    pub async fn upload_file<P: AsRef<Path>>(
        &self,
        local_path: P,
        remote_path: &str,
        progress_callback: Option<Box<dyn Fn(TransferProgress) + Send + Sync>>,
    ) -> Result<()> {
        let mut local_file = tokio::fs::File::open(&local_path).await?;
        let local_metadata = local_file.metadata().await?;
        let total_size = local_metadata.len();

        let mut remote_file = self.session.create(remote_path).await?;
        let mut buffer = vec![0u8; 32768]; // 32KB buffer
        let mut transferred = 0u64;
        let start_time = std::time::Instant::now();

        loop {
            let bytes_read = local_file.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }

            remote_file.write_all(&buffer[..bytes_read]).await?;
            transferred += bytes_read as u64;

            if let Some(ref callback) = progress_callback {
                let elapsed = start_time.elapsed().as_secs_f64();
                let speed = if elapsed > 0.0 {
                    (transferred as f64 / elapsed) as u64
                } else {
                    0
                };

                let progress = TransferProgress {
                    total_bytes: total_size,
                    transferred_bytes: transferred,
                    percentage: (transferred as f64 / total_size as f64) * 100.0,
                    speed_bps: speed,
                };

                callback(progress);
            }
        }

        remote_file.flush().await?;
        Ok(())
    }

    pub async fn create_directory(&self, path: &str) -> Result<()> {
        self.session.create_dir(path).await?;
        Ok(())
    }

    pub async fn remove_file(&self, path: &str) -> Result<()> {
        self.session.remove_file(path).await?;
        Ok(())
    }

    pub async fn remove_directory(&self, path: &str) -> Result<()> {
        self.session.remove_dir(path).await?;
        Ok(())
    }

    pub async fn rename(&self, old_path: &str, new_path: &str) -> Result<()> {
        self.session.rename(old_path, new_path).await?;
        Ok(())
    }

    pub async fn get_file_info(&self, path: &str) -> Result<FileInfo> {
        let attrs = self.session.metadata(path).await?;
        let path_obj = PathBuf::from(path);
        let name = path_obj
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path)
            .to_string();

        Ok(FileInfo {
            name,
            path: path.to_string(),
            is_directory: attrs.is_dir(),
            size: attrs.size().unwrap_or(0),
            modified: attrs.mtime().map(|t| {
                chrono::DateTime::from_timestamp(t as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_default()
            }),
            permissions: attrs.permissions().map(|p| format!("{:o}", p)),
        })
    }
}
