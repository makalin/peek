# Peek

> A Norton Commander-inspired TUI admin dashboard for macOS, built in Rust.

Peek consolidates daily system tasks into one fast, split-window interface. Manage running ports (check/kill), list processes and cron jobs, edit text files, and take notes. A powerful, all-in-one utility for developers and system admins.

![Build Status](https://img.shields.io/badge/build-passing-brightgreen)
![Rust](https://img.shields.io/badge/rust-stable-orange)
![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)


## 🚀 Concept

Peek is a modern, fast, and efficient terminal admin UI for macOS, written in Rust. It draws inspiration from the classic Norton Commander dual-pane layout but reimagines it for modern system administration.

Stop switching between `lsof -i`, `nano`, `crontab -l`, `ps aux`, and your notes app. Peek aims to combine all these daily-driver utilities into a single, cohesive, and fast application that runs directly in your terminal.

## ✨ Core Features

* **Split-Window Layout:** A classic dual-pane view for managing multiple tasks at once.
* **All-in-One Dashboard:** Combines network, process, file, and task management.
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

### 3. Cron Job Viewer
* **List Cron Jobs:** Parses and displays all active `crontab` jobs for the current user.
* **Quick View:** See the command and the schedule for each job.
* *(Future Goal: Integrate with `launchd` for a complete macOS task overview.)*

### 4. File Editor
* **Quick Edits:** A simple, built-in text editor (similar to `nano`) for quick configuration changes, editing scripts, or updating text files.
* **Save/Close:** Basic file operations without leaving the TUI.

### 5. Notes & Lists
* **Scratchpad:** A persistent pane for taking quick notes, storing temporary commands, or managing a simple to-do list.
* **Auto-Save:** Notes are saved automatically for your next session.

### 6. Embedded Terminal
* **Command Line:** A single-line input at the bottom of the screen (like in `vim` or `Norton Commander`) to execute any arbitrary shell command.

## 📦 Installation

*(This section will be updated once build binaries are available.)*

To build from source:
```bash
git clone [https://github.com/makalin/peek.git](https://github.com/makalin/peek.git)
cd peek
cargo build --release
./target/release/peek
````

## ⌨️ Usage

*(Details on keybindings and commands will be added here.)*

  * `Tab`: Switch between split panes.
  * `F1-F10`: Access different tool modules (e.g., `F1` for Ports, `F2` for Processes).
  * `/`: Access the embedded terminal line at the bottom.

## ⚖️ License

This project is licensed under the MIT License. See the [LICENSE](https://www.google.com/search?q=LICENSE) file for details.

-----

Created by [Mehmet T. AKALIN](https://github.com/makalin)
