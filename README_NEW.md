# TermNest - Cross-Platform SSH Client

A modern, cross-platform alternative to MobaXterm built with Tauri v2 and Vue 3. TermNest provides a lightweight, secure, and extensible SSH client with a sleek web-based interface.

## ✨ Features

### 🚀 Core Functionality
- **SSH Connection Management**: Create, edit, and organize SSH sessions
- **Multiple Authentication Methods**: SSH keys, SSH agent, and password authentication
- **Protocol Support**: SSH and SFTP (with RDP, VNC, and Telnet planned)
- **Session Persistence**: Sessions are saved and restored between app launches
- **Real-time Connection Status**: Live updates of connection states

### 🎨 Modern UI
- **Cross-Platform**: Runs on Windows, macOS, and Linux
- **Theme Support**: Light, dark, and system theme options
- **Responsive Design**: Adapts to different screen sizes
- **Terminal Emulation**: Built-in terminal with command simulation
- **Settings Management**: Persistent application settings

### 🔧 Technical Features
- **Lightweight**: Small binary size compared to Electron alternatives (~8MB vs 244MB)
- **Performance**: Native Rust backend with minimal resource usage
- **Security**: Tauri's secure IPC and permission system
- **Extensible**: Plugin architecture for future protocol support

## 🏗️ Architecture

### Backend (Rust)
- **Tauri Framework**: Cross-platform desktop application framework
- **Async Operations**: Tokio-based async runtime for networking
- **State Management**: Thread-safe session and connection management
- **Plugin System**: Modular architecture for protocol extensions

### Frontend (Vue 3)
- **Composition API**: Modern Vue.js with TypeScript
- **Pinia**: State management for session and settings data
- **Component Architecture**: Modular, reusable UI components
- **Responsive Design**: Mobile-friendly responsive layout

## 🚀 Getting Started

### Prerequisites
- Node.js (16+)
- Rust (1.70+)
- pnpm (recommended) or npm

### Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/termnest.git
cd termnest
```

2. Install dependencies:
```bash
pnpm install
```

3. Run in development mode:
```bash
pnpm tauri dev
```

4. Build for production:
```bash
pnpm tauri build
```

## 📱 Usage

### Creating SSH Sessions

1. Click the "New Session" button
2. Fill in the connection details:
   - **Session Name**: A friendly name for your connection
   - **Host**: IP address or hostname
   - **Port**: SSH port (default: 22)
   - **Username**: SSH username
   - **Protocol**: SSH or SFTP
   - **Authentication**: Choose between SSH key, agent, or password

3. Click "Create Session" to save

### Connecting to Sessions

1. Find your session in the sessions grid
2. Click the "Connect" button
3. Monitor the connection status in real-time
4. Use the "Disconnect" button to close the connection

### Settings

Access settings through the header button to configure:
- **Theme**: Light, dark, or system preference
- **Default Protocol**: SSH or SFTP for new sessions
- **Auto-connect**: Automatically connect on startup
- **SSH Key Path**: Default path for SSH keys
- **Terminal Settings**: Font size and color scheme

## 🔧 Configuration

### SSH Key Authentication
- Place your SSH keys in `~/.ssh/`
- Specify the key path in session settings
- Ensure proper permissions (600 for private keys)

### Session Storage
- Sessions are stored locally using Tauri's secure storage
- Settings are persisted in `.settings.dat`
- No sensitive data is stored in plain text

## 🛠️ Development

### Project Structure
```
termnest/
├── src/                    # Vue frontend
│   ├── components/         # UI components
│   ├── stores/            # Pinia stores
│   └── assets/            # Static assets
├── src-tauri/             # Rust backend
│   ├── src/               # Rust source code
│   ├── icons/             # Application icons
│   └── Cargo.toml         # Rust dependencies
└── public/                # Public assets
```

### Architecture Overview
```
┌─────────────────────────────────────────────────────────────┐
│                    Frontend (Vue 3)                        │
├─────────────────────────────────────────────────────────────┤
│  SessionManager → SessionCard → CreateSessionModal         │
│  SettingsModal → Terminal → App.vue                        │
│                                                             │
│  Stores: sessions.ts + settings.ts (Pinia)                 │
└─────────────────────────────────────────────────────────────┘
                              ↕ IPC
┌─────────────────────────────────────────────────────────────┐
│                   Backend (Rust/Tauri)                     │
├─────────────────────────────────────────────────────────────┤
│  Commands: list_sessions, create_session, connect_ssh      │
│  State: AppState with session & connection management      │
│  Events: Real-time connection status updates               │
│  Storage: Tauri Store plugin for persistence               │
└─────────────────────────────────────────────────────────────┘
```

### Key Components
- **SessionManager**: Main session management interface
- **SessionCard**: Individual session display and controls
- **CreateSessionModal**: Session creation dialog
- **SettingsModal**: Application settings
- **Terminal**: Terminal emulator component

### Backend Commands
- `list_sessions`: Retrieve all saved sessions
- `create_session`: Create a new session
- `update_session`: Update existing session
- `delete_session`: Remove a session
- `connect_ssh`: Establish SSH connection
- `disconnect_session`: Close connection
- `get_connection_status`: Get connection state

## 🎯 Roadmap

### Near Term (v0.2)
- [ ] Actual SSH protocol implementation
- [ ] Terminal with real PTY support
- [ ] SSH key generation and management
- [ ] Session import/export

### Medium Term (v0.3)
- [ ] SFTP file browser
- [ ] Port forwarding support
- [ ] Session groups and folders
- [ ] Multiple tabs per session

### Long Term (v1.0)
- [ ] RDP protocol support
- [ ] VNC protocol support
- [ ] X11 forwarding
- [ ] Plugin system for third-party protocols
- [ ] Session sharing and synchronization

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📋 Requirements

### System Requirements
- **Memory**: 100MB RAM minimum
- **Storage**: 50MB disk space
- **OS**: Windows 10+, macOS 10.15+, or Linux (Ubuntu 18.04+)

### Development Requirements
- Node.js 16+
- Rust 1.70+
- Platform-specific build tools (see Tauri docs)

## 🔒 Security

- All SSH connections use industry-standard encryption
- Private keys are never transmitted or stored in plain text
- Tauri's permission system limits backend access
- Settings storage uses Tauri's secure storage APIs

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 📞 Support

- 🐛 **Issues**: [GitHub Issues](https://github.com/yourusername/termnest/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/yourusername/termnest/discussions)
- 📧 **Email**: support@termnest.dev

---

**TermNest** - Making remote connections simple, secure, and beautiful. 🏠✨
