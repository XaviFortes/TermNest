use anyhow::{anyhow, Result};
use crossbeam_channel::{Receiver, Sender};
use portable_pty::{native_pty_system, Child, MasterPty, PtySize};
use russh::client::Channel;
use russh::client::Msg;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSize {
    pub cols: u16,
    pub rows: u16,
    pub width: u16,
    pub height: u16,
}

impl Default for TerminalSize {
    fn default() -> Self {
        Self {
            cols: 80,
            rows: 24,
            width: 640,
            height: 480,
        }
    }
}

pub struct SshTerminal {
    channel: Arc<Mutex<Channel<Msg>>>,
    size: TerminalSize,
    output_sender: Sender<String>,
    input_receiver: Receiver<String>,
    _read_task: JoinHandle<()>,
    _write_task: JoinHandle<()>,
}

impl SshTerminal {
    pub async fn new(
        mut channel: Channel<Msg>,
        size: TerminalSize,
    ) -> Result<(Self, Receiver<String>, Sender<String>)> {
        // Request PTY
        channel
            .request_pty(
                false,
                "xterm-256color",
                size.cols as u32,
                size.rows as u32,
                size.width as u32,
                size.height as u32,
                &[],
            )
            .await?;

        // Start shell
        channel.request_shell(false).await?;

        let channel = Arc::new(Mutex::new(channel));

        // Create channels for communication
        let (output_sender, output_receiver) = crossbeam_channel::unbounded();
        let (input_sender, input_receiver) = crossbeam_channel::unbounded();

        // Spawn read task
        let read_channel = channel.clone();
        let read_output_sender = output_sender.clone();
        let read_task = tokio::spawn(async move {
            let mut channel_guard = read_channel.lock().await;
            while let Some(msg) = channel_guard.wait().await {
                match msg {
                    russh::ChannelMsg::Data { ref data } => {
                        if let Ok(text) = String::from_utf8(data.to_vec()) {
                            let _ = read_output_sender.send(text);
                        }
                    }
                    russh::ChannelMsg::ExtendedData { ref data, ext: 1 } => {
                        // stderr
                        if let Ok(text) = String::from_utf8(data.to_vec()) {
                            let _ = read_output_sender.send(text);
                        }
                    }
                    _ => {}
                }
            }
        });

        // Spawn write task
        let write_channel = channel.clone();
        let write_input_receiver = input_receiver.clone();
        let write_task = tokio::spawn(async move {
            while let Ok(input) = write_input_receiver.recv() {
                let mut channel_guard = write_channel.lock().await;
                let _ = channel_guard.data(input.as_bytes()).await;
            }
        });

        let terminal = Self {
            channel,
            size,
            output_sender,
            input_receiver,
            _read_task: read_task,
            _write_task: write_task,
        };

        Ok((terminal, output_receiver, input_sender))
    }

    pub async fn send_input(&self, data: String) -> Result<()> {
        self.input_receiver.try_send(data)
            .map_err(|e| anyhow!("Failed to send input: {}", e))?;
        Ok(())
    }

    pub async fn resize(&mut self, new_size: TerminalSize) -> Result<()> {
        self.size = new_size.clone();
        let mut channel = self.channel.lock().await;
        channel
            .window_change(
                new_size.cols as u32,
                new_size.rows as u32,
                new_size.width as u32,
                new_size.height as u32,
            )
            .await?;
        Ok(())
    }

    pub async fn close(&self) -> Result<()> {
        let mut channel = self.channel.lock().await;
        channel.eof().await?;
        Ok(())
    }
}

// Local PTY terminal for local shell access
pub struct LocalTerminal {
    master: Box<dyn MasterPty + Send + Sync>,
    child: Box<dyn Child + Send + Sync>,
    output_sender: Sender<String>,
    input_receiver: Receiver<String>,
    _read_task: JoinHandle<()>,
    _write_task: JoinHandle<()>,
}

impl LocalTerminal {
    pub fn new(
        size: TerminalSize,
        shell: Option<String>,
    ) -> Result<(Self, Receiver<String>, Sender<String>)> {
        let pty_system = native_pty_system();

        let pty_size = PtySize {
            rows: size.rows,
            cols: size.cols,
            pixel_width: size.width,
            pixel_height: size.height,
        };

        let pty_pair = pty_system.openpty(pty_size)?;

        // Determine shell command
        let shell_cmd = shell.unwrap_or_else(|| {
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
        });

        let mut cmd = portable_pty::CommandBuilder::new(&shell_cmd);
        cmd.env("TERM", "xterm-256color");

        let child = pty_pair.slave.spawn_command(cmd)?;

        // Create channels for communication
        let (output_sender, output_receiver) = crossbeam_channel::unbounded();
        let (input_sender, input_receiver) = crossbeam_channel::unbounded();

        // Spawn read task
        let mut reader = pty_pair.master.try_clone_reader()?;
        let read_output_sender = output_sender.clone();
        let read_task = tokio::spawn(async move {
            let mut buffer = [0u8; 1024];
            loop {
                match reader.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        if let Ok(text) = String::from_utf8(buffer[..n].to_vec()) {
                            let _ = read_output_sender.send(text);
                        }
                    }
                    Ok(_) => break, // EOF
                    Err(_) => break, // Error
                }
            }
        });

        // Spawn write task
        let mut writer = pty_pair.master.take_writer()?;
        let write_input_receiver = input_receiver.clone();
        let write_task = tokio::spawn(async move {
            while let Ok(input) = write_input_receiver.recv() {
                let _ = writer.write_all(input.as_bytes());
                let _ = writer.flush();
            }
        });

        let terminal = Self {
            master: pty_pair.master,
            child,
            output_sender,
            input_receiver,
            _read_task: read_task,
            _write_task: write_task,
        };

        Ok((terminal, output_receiver, input_sender))
    }

    pub fn send_input(&self, data: String) -> Result<()> {
        self.input_receiver.try_send(data)
            .map_err(|e| anyhow!("Failed to send input: {}", e))?;
        Ok(())
    }

    pub fn resize(&mut self, new_size: TerminalSize) -> Result<()> {
        let pty_size = PtySize {
            rows: new_size.rows,
            cols: new_size.cols,
            pixel_width: new_size.width,
            pixel_height: new_size.height,
        };
        self.master.resize(pty_size)?;
        Ok(())
    }

    pub fn kill(&mut self) -> Result<()> {
        self.child.kill()?;
        Ok(())
    }
}
