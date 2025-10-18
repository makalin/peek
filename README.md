# Peek

> A Norton Commander-inspired TUI admin dashboard for macOS, built in Rust.

Peek consolidates daily system tasks into one fast, split-window interface. Manage running ports (check/kill), list processes and services, edit text files, and take notes. A powerful, all-in-one utility for developers and system admins.

![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![Rust](https://img.shields.io/badge/rust-stable-orange)
![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)

## 🚀 Concept

Peek is a modern, fast, and efficient terminal admin UI for macOS, written in Rust. It draws inspiration from the classic Norton Commander dual-pane layout but reimagines it for modern system administration.

Stop switching between `lsof -i`, `nano`, `launchctl list`, `ps aux`, and your notes app. Peek aims to combine all these daily-driver utilities into a single, cohesive, and fast application that runs directly in your terminal.

## ✨ Core Features

* **Split-Window Layout:** A classic dual-pane view for managing multiple tasks at once.
* **All-in-One Dashboard:** Combines network, process, file, and service management.
* **Fast & Lightweight:** Built in Rust for native performance and reliability.
* **Embedded Command Line:** A persistent input line at the bottom to run quick shell commands.
* **Mouse and Keyboard Driven:** Designed for both quick keyboard shortcuts and mouse clicks.

## 🛠️ Included Tools & Functions

Peek integrates several modules, each accessible within a "pane" of the application:

### 1. Network Manager
* **List Ports:** Lists all running ports (`TCP`/`UDP`) and the processes using them.
* **Check/Kill Port:** Select a port to gracefully kill the associated process.
* **Filter:** Quickly filter by port number, process name, or protocol.

### 2. Task (Process) Manager
* **List Processes:** A live-updating list of all running tasks (similar to `top` or `htop`).
* **Kill Process:** Send `SIGTERM` or `SIGKILL` to selected processes.
* **View Details:** Inspect process details (PID, User, CPU%, Mem%).

### 3. Service Manager
* **List Services:** Displays all macOS services managed by `launchctl`.
* **Start/Stop Services:** Control service states directly from the interface.
* **Service Status:** View running/stopped status and process IDs.

### 4. File Manager
* **Browse Files:** Navigate through the file system.
* **Quick Edits:** A simple, built-in text editor for quick configuration changes.
* **File Operations:** Basic file operations without leaving the TUI.

### 5. System Information
* **System Stats:** Display hostname, uptime, CPU cores, OS version.
* **Resource Usage:** Monitor processes, ports, services, and system logs.
* **Real-time Updates:** Live system information with refresh capabilities.

### 6. Admin Tools
* **Disk Analyzer:** Analyze disk usage and storage.
* **Network Monitor:** Monitor network traffic and connections.
* **Process Killer:** Kill problematic processes.
* **System Cleaner:** Clean temporary files.
* **Backup Manager:** Manage system backups.
* **Security Scanner:** Scan for security issues.
* **Performance Monitor:** Monitor system performance.
* **System Repair:** Repair system issues.

## 📦 Installation

To build from source:
```bash
git clone https://github.com/makalin/peek.git
cd peek
cargo build --release
./target/release/peek
```

## ⌨️ Usage

### Navigation
* **Arrow Keys:** Navigate through items in the current pane
* **Left/Right:** Switch between tabs
* **Tab:** Switch between split panes
* **Number Keys (1-9):** Quick access to different modules

### Function Keys
* **F1:** Help - Toggle help information
* **F2:** Menu - Access main menu
* **F3:** View - View selected item details
* **F4:** Edit - Edit selected item
* **F5:** Copy - Copy selected item
* **F6:** Move - Move selected item
* **F7:** Make Directory - Create new directory
* **F8:** Delete - Delete selected item
* **F9:** Menu - Access context menu
* **F10:** Quit - Exit application

### Commands
* **q:** Quit application
* **h:** Toggle help
* **r:** Refresh data
* **k:** Kill selected process (Processes tab)
* **K:** Terminate selected process (Processes tab)
* **s:** Start selected service (Services tab)
* **S:** Stop selected service (Services tab)

### Tabs
1. **Dashboard** - System overview and quick actions
2. **Files** - File system browser
3. **Processes** - Process management
4. **Network** - Network interfaces
5. **Ports** - Network ports and connections
6. **Services** - macOS service management
7. **System** - System information
8. **Tools** - Admin tools and utilities
9. **Settings** - Application settings

## 🔧 Technical Details

### Dependencies
- **crossterm:** Terminal manipulation and event handling
- **ratatui:** Terminal User Interface framework
- **sysinfo:** System information gathering
- **tokio:** Async runtime for background tasks
- **serde:** Serialization for configuration

### Architecture
- **TUI Framework:** Built with `ratatui` for cross-platform terminal support
- **Event Handling:** Uses `crossterm` for keyboard and mouse input
- **System Integration:** Leverages `sysinfo` for real-time system data
- **Async Support:** Background data refresh with `tokio`

## 🚨 Known Issues

- **Terminal Compatibility:** Some macOS terminals may show "Device not configured" error
- **Mouse Support:** Limited mouse interaction in certain terminal emulators
- **Color Support:** Requires terminal with 256-color support for best experience

## 🔮 Future Roadmap

- [ ] Enhanced file operations (copy, move, delete)
- [ ] Integrated text editor with syntax highlighting
- [ ] Real-time system monitoring with graphs
- [ ] Plugin system for custom tools
- [ ] Configuration file support
- [ ] Theme customization
- [ ] Remote system management
- [ ] Log file viewer with filtering
- [ ] Package management integration
- [ ] Backup and restore functionality

## ⚖️ License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📞 Support

If you encounter any issues or have questions, please open an issue on GitHub.

-----

Created by [Mehmet T. AKALIN](https://github.com/makalin)