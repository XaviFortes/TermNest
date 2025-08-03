<div align="center">

# 🏡 TermNest

**A Modern, Cross-Platform SSH Client**

*Lightweight • Secure • Beautiful*

[![GitHub Release](https://img.shields.io/github/v/release/XaviFortes/TermNest?style=for-the-badge&logo=github&color=blue)](https://github.com/XaviFortes/TermNest/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/XaviFortes/TermNest/total?style=for-the-badge&logo=github&color=green)](https://github.com/XaviFortes/TermNest/releases)
[![License](https://img.shields.io/github/license/XaviFortes/TermNest?style=for-the-badge&color=orange)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey?style=for-the-badge)](https://github.com/XaviFortes/TermNest/releases)

[🚀 **Download**](https://github.com/XaviFortes/TermNest/releases/latest) • [📖 **Documentation**](#-getting-started) • [🐛 **Issues**](https://github.com/XaviFortes/TermNest/issues) • [💬 **Discussions**](https://github.com/XaviFortes/TermNest/discussions)

---

</div>

## 🌟 Why TermNest?

TermNest is a modern SSH client designed as a lightweight, cross-platform alternative to MobaXterm. Built with **Tauri v2** and **Vue 3**, it combines the performance of Rust with the elegance of modern web technologies.

### 🆚 Comparison with Alternatives

| Feature | TermNest | MobaXterm | PuTTY | iTerm2 |
|---------|----------|-----------|-------|--------|
| **Size** | ~8MB | ~244MB | ~3MB | ~15MB |
| **Cross-Platform** | ✅ | ❌ (Windows only) | ✅ | ❌ (macOS only) |
| **Modern UI** | ✅ | ❌ | ❌ | ✅ |
| **SFTP Integration** | ✅🟡(WIP) | ✅ | ❌ | ❌ |
| **Session Management** | ✅ | ✅ | ✅ | ✅ |
| **Resource Usage** | 🟢 Low | 🔴 High | 🟢 Low | 🟡 Medium |
| **Security** | 🟢 Tauri sandboxing | 🟡 Traditional | 🟡 Traditional | 🟢 macOS sandbox |

---

## ✨ Features

### 🚀 **Core Functionality**
- **🔐 SSH Connection Management**: Create, edit, and organize SSH sessions with ease
- **🔑 Multiple Authentication**: Support for SSH keys (RSA, Ed25519, ECDSA), SSH agent, and password auth
- **🌐 Protocol Support**: SSH and SFTP with integrated file browser (RDP, VNC, Telnet coming soon)
- **💾 Session Persistence**: All sessions saved and restored between app launches
- **📊 Real-time Status**: Live connection monitoring and status updates
- **🏷️ Session Tagging**: Organize connections with custom tags and categories

### 🎨 **Modern Interface**
- **🌈 Beautiful UI**: Clean, modern interface inspired by VS Code
- **🎭 Theme Support**: Light, dark, and system themes with customizable colors
- **📱 Responsive Design**: Adapts seamlessly to different screen sizes
- **⚡ Terminal Emulation**: Full-featured terminal with xterm.js
- **📁 File Management**: Integrated SFTP browser with drag-and-drop support
- **⚙️ Settings Hub**: Centralized configuration management

### 🔧 **Technical Excellence**
- **🪶 Lightweight**: Only ~8MB compared to 244MB alternatives
- **⚡ Performance**: Native Rust backend with minimal resource usage
- **🔒 Security**: Tauri's permission system and secure IPC
- **🧩 Extensible**: Plugin architecture for future protocol support
- **🔄 Cross-Platform**: Single codebase for Windows, macOS, and Linux

---

## 📸 Screenshots

<div align="center">

### Main Dashboard
![Dashboard](docs/screenshots/dashboard.png)

### SSH Terminal in Action
![Terminal](docs/screenshots/terminal.png)

### SFTP File Browser
![SFTP](docs/screenshots/sftp.png)

### Settings Panel
![Settings](docs/screenshots/settings.png)

</div>

---

## 🚀 Quick Start

### 📥 **Download & Install**

#### Windows
```powershell
# Download the latest release
Invoke-WebRequest -Uri "https://github.com/XaviFortes/TermNest/releases/latest/download/TermNest_x64.msi" -OutFile "TermNest.msi"
# Install
Start-Process "TermNest.msi"
```

#### macOS
```bash
# Download and install via Homebrew (coming soon)
brew install --cask termnest

# Or download directly
curl -L "https://github.com/XaviFortes/TermNest/releases/latest/download/TermNest.dmg" -o TermNest.dmg
```

#### Linux
```bash
# Ubuntu/Debian
wget https://github.com/XaviFortes/TermNest/releases/latest/download/termnest_amd64.deb
sudo dpkg -i termnest_amd64.deb

# Fedora/RHEL
wget https://github.com/XaviFortes/TermNest/releases/latest/download/termnest.rpm
sudo rpm -i termnest.rpm

# AppImage (Universal) (Not Yet Bundled)
# wget https://github.com/XaviFortes/TermNest/releases/latest/download/TermNest.AppImage
# chmod +x TermNest.AppImage
# ./TermNest.AppImage
```

### 🏁 **First Steps**

1. **Launch TermNest** from your applications menu
2. **Create your first session** by clicking the "+" button
3. **Configure SSH settings**:
   - Host: `your-server.com`
   - Username: `your-username`
   - Auth Method: Browse for your SSH key or use SSH agent
4. **Connect** and enjoy your secure shell session!

---

## 🛠️ Development Setup

### Prerequisites
- **Node.js** 18+ ([Download](https://nodejs.org/))
- **Rust** 1.75+ ([Install](https://rustup.rs/))
- **pnpm** ([Install](https://pnpm.io/installation))

### Build from Source

```bash
# Clone the repository
git clone https://github.com/XaviFortes/TermNest.git
cd TermNest

# Install frontend dependencies
pnpm install

# Install Tauri CLI
cargo install tauri-cli --version "^2.0.0"

# Development mode
cargo tauri dev

# Build for production
cargo tauri build
```

### 🏗️ Project Structure

```
TermNest/
├── 📁 src/                    # Vue.js frontend
│   ├── 📁 components/         # UI components
│   ├── 📁 stores/            # Pinia state stores
│   ├── 📁 views/             # Page components
│   └── 📁 assets/            # Static assets
├── 📁 src-tauri/             # Rust backend
│   ├── 📁 src/               # Rust source code
│   ├── 📄 Cargo.toml         # Rust dependencies
│   └── 📄 tauri.conf.json    # Tauri configuration
├── 📁 docs/                  # Documentation
└── 📄 package.json           # Node.js dependencies
```

---

## 🔧 Configuration

### SSH Key Setup

TermNest supports various SSH key formats:

```bash
# Generate a new Ed25519 key (recommended)
ssh-keygen -t ed25519 -C "your_email@example.com"

# Generate a new RSA key (legacy)
ssh-keygen -t rsa -b 4096 -C "your_email@example.com"

# Copy public key to server
ssh-copy-id -i ~/.ssh/id_ed25519.pub user@server.com
```

### Application Settings

Settings are stored in:
- **Windows**: `%APPDATA%/TermNest/`
- **macOS**: `~/Library/Application Support/TermNest/`
- **Linux**: `~/.config/TermNest/`

---

## 🗺️ Roadmap

### 🎯 **Next Release (v0.2)**
- [ ] 🔐 SSH agent integration
- [ ] 📋 Connection history and favorites
- [ ] 🎨 Custom theme editor
- [ ] 📁 Improved/Working SFTP file operations

### 🚀 **Future Releases**
- [ ] 🖥️ RDP protocol support
- [ ] 📺 VNC integration
- [ ] 📞 Telnet support
- [ ] 🔌 Plugin system
- [ ] ☁️ Cloud sync for sessions
- [ ] 📱 Mobile companion app

---

## 🤝 Contributing

We welcome contributions! Here's how you can help:

### 🐛 **Report Bugs**
Found a bug? [Open an issue](https://github.com/XaviFortes/TermNest/issues/new?template=bug_report.md)

### 💡 **Suggest Features**
Have an idea? [Start a discussion](https://github.com/XaviFortes/TermNest/discussions/new?category=ideas)

### 🔧 **Submit Code**
1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Commit your changes: `git commit -m 'Add amazing feature'`
4. Push to the branch: `git push origin feature/amazing-feature`
5. Open a Pull Request

### 📝 **Improve Documentation**
Documentation improvements are always welcome!

---

## 🏆 Acknowledgments

### Built With
- ⚡ **[Tauri](https://tauri.app/)** - Secure, fast, cross-platform desktop apps
- 🎯 **[Vue 3](https://vuejs.org/)** - Progressive JavaScript framework
- 🦀 **[Rust](https://www.rust-lang.org/)** - Systems programming language
- 🔐 **[SSH2](https://crates.io/crates/ssh2)** - SSH client library for Rust
- 💻 **[xterm.js](https://xtermjs.org/)** - Terminal emulator for the web

---

## 📄 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

---

## 💖 Support

If you find TermNest useful, consider:

- ⭐ **Star** the repository
- 🐛 **Report bugs** and suggest features
- 💬 **Share** with your friends and colleagues
- ☕ **[Buy me a coffee](https://ko-fi.com/XaviFortes)** (if you want to support development)

---

<div align="center">

**Made with ❤️ by [XaviFortes](https://github.com/XaviFortes)**

*TermNest - Your cozy terminal for remote connections*

[⬆️ Back to Top](#-termnest)

</div>
