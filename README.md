# Peek

![Peek Logo](peek_logo.png)

> A Norton Commander-inspired TUI admin dashboard for macOS, built in Rust.

Peek consolidates daily system tasks into one fast, tab-based interface. View and manage running ports, processes, and services, browse files, and monitor system information. A powerful, all-in-one utility for developers and system admins.

![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![Rust](https://img.shields.io/badge/rust-stable-orange)
![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)

## 🚀 Concept

Peek is a modern, fast, and efficient terminal admin UI for macOS, written in Rust. It draws inspiration from the classic Norton Commander dual-pane layout but reimagines it for modern system administration.

Stop switching between `lsof -i`, `launchctl list`, and `ps aux`. Peek combines these daily-driver utilities into a single, cohesive, and fast application that runs directly in your terminal.

## ✨ Core Features

* **Tab-Based Interface:** Sidebar navigation with 9 different tabs for various system management tasks.
* **All-in-One Dashboard:** Combines network, process, file, and service management in one interface.
* **Fast & Lightweight:** Built in Rust for native performance and reliability.
* **Keyboard-Driven:** Quick keyboard shortcuts for navigation and actions.
* **Real-Time Data:** Refresh system information on demand with the 'r' key.
* **Embedded Command Line:** Press ':' to execute shell commands directly from the interface.
* **File Operations:** Copy, move, and delete files and directories without leaving the TUI.

## 🛠️ Implemented Features

### 1. Dashboard
* **System Overview:** Displays hostname, uptime, CPU cores, OS version, and counts of processes, ports, and services.
* **Quick Actions:** Shows available keyboard shortcuts and commands.

### 2. Process Manager
* **List Processes:** Displays all running processes sorted by CPU usage.
* **Process Details:** Shows PID, name, CPU%, memory usage, and user for each process.
* **Kill Process:** Press 'k' to send `SIGKILL` to selected process.
* **Terminate Process:** Press 'K' to send `SIGTERM` to selected process.

### 3. Network Interfaces
* **List Interfaces:** Displays all network interfaces with received and transmitted bytes.
* **Traffic Monitoring:** Shows network traffic statistics for each interface.

### 4. Port Manager
* **List Ports:** Lists all running ports (`TCP`/`UDP`) using `lsof`.
* **Port Details:** Shows protocol, port number, process name, local address, connection state, and PID.
* **Kill Process by Port:** Press 'k' to kill the process using the selected port.
* **Filter Ports:** Press '/' to enter filter mode, type to filter by port number, process name, protocol, or address. Press Enter to apply, Esc to cancel/clear.

### 5. Service Manager
* **List Services:** Displays all macOS services managed by `launchctl`.
* **Service Status:** View running/stopped status and process IDs.
* **Start Service:** Press 's' to start selected service.
* **Stop Service:** Press 'S' to stop selected service.

### 6. File Browser
* **Full File System Navigation:** Browse the entire file system, starting from root (`/`).
* **Directory Navigation:** Press Enter to navigate into directories, or select ".." to go to parent directory.
* **File Details:** Shows file type (directory/file), size, modification date, and permissions.
* **Current Path Display:** Current directory path is shown in the title bar.
* **File Operations:**
  * **Copy:** Press 'c' to copy selected file/directory (then enter destination path)
  * **Move:** Press 'm' to move selected file/directory (then enter destination path)
  * **Delete:** Press 'd' to delete selected file/directory (confirm with 'y' or cancel with 'n')

### 7. System Information
* **System Stats:** Display hostname, uptime, CPU cores, and OS version.
* **Resource Counts:** Shows total counts of processes, ports, and services.

### 8. Settings
* **Application Settings:** Displays current settings and available keyboard shortcuts.

## 🔮 Planned Features

The following features are planned for future releases:

### Enhanced File Management
* File editing capabilities
* Directory creation and management

### Admin Tools
* Disk Analyzer - Analyze disk usage and storage
* Network Monitor - Monitor network traffic and connections
* System Cleaner - Clean temporary files
* Backup Manager - Manage system backups
* Security Scanner - Scan for security issues
* Performance Monitor - Monitor system performance with graphs
* System Repair - Repair system issues

### UI Enhancements
* True dual-pane split-window layout
* Mouse support for navigation and selection
* Function key support (F1-F10)
* Notes feature for quick note-taking

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
* **Arrow Keys (↑↓):** Navigate through items in the current tab
* **Left/Right Arrow Keys (←→):** Switch between tabs
* **Tab Key:** Switch to next tab (alternative to Right arrow)
* **Number Keys (1-9):** Quick access to different tabs (1=Dashboard, 2=Files, etc.)
* **Enter:** Open directory in Files tab

### Commands
* **q:** Quit application
* **h:** Toggle help information
* **r:** Refresh all data
* **:** (colon): Enter command line mode to execute shell commands
* **k:** Kill selected process with SIGKILL (Processes tab) or kill process by port (Ports tab)
* **K:** Terminate selected process with SIGTERM (Processes tab only)
* **s:** Start selected service (Services tab only)
* **S:** Stop selected service (Services tab only)
* **/:** Enter filter mode (Ports tab only)
* **Esc:** Exit filter mode and clear filter (Ports tab only), or cancel current operation

### File Operations (Files tab)
* **c:** Copy selected file/directory
* **m:** Move selected file/directory
* **d:** Delete selected file/directory
* **Enter:** Confirm file operation (after entering destination for copy/move, or 'y' for delete)
* **y/n:** Confirm or cancel delete operation

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
- **serde:** Serialization for configuration (planned)
- **serde_json:** JSON serialization (planned)

### Architecture
- **TUI Framework:** Built with `ratatui` for cross-platform terminal support
- **Event Handling:** Uses `crossterm` for keyboard input
- **System Integration:** Leverages `sysinfo` for real-time system data and system commands (`lsof`, `launchctl`, `ls`) for additional information

## 🚨 Known Issues

- **Terminal Compatibility:** Some macOS terminals may show "Device not configured" error
- **Color Support:** Requires terminal with 256-color support for best experience
- **Port Filtering:** Filter mode is active while typing; use Enter to exit filter mode after typing

## 🔮 Future Roadmap

- [x] Full file system navigation (completed)
- [x] Kill process by port selection (completed)
- [x] Port filtering capabilities (completed)
- [x] Tab key to switch between tabs (completed)
- [x] Enhanced file operations (copy, move, delete) (completed)
- [x] Embedded command line for shell commands (completed)
- [ ] File editing capabilities
- [ ] Integrated text editor with syntax highlighting
- [ ] Real-time system monitoring with graphs
- [ ] True dual-pane split-window layout
- [ ] Mouse support for navigation
- [ ] Function key support (F1-F10)
- [ ] Notes feature for quick note-taking
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