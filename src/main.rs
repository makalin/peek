use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
};
use std::{cmp::Ordering, error::Error, fs, io, path::PathBuf, process::Command};
use sysinfo::*;

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let res = run_app(&mut terminal);

    // restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}")
    }

    Ok(())
}

struct AppState {
    current_tab: Tab,
    selected_index: usize,
    show_help: bool,
    processes: Vec<ProcessInfo>,
    networks: Vec<NetworkInfo>,
    ports: Vec<PortInfo>,
    all_ports: Vec<PortInfo>, // Store unfiltered ports
    files: Vec<FileInfo>,
    services: Vec<ServiceInfo>,
    system_info: SystemInfo,
    current_path: PathBuf, // Current directory for file browser
    port_filter: String,   // Filter text for ports
    port_filter_mode: bool, // Whether we're in filter input mode
    command_line: String,  // Embedded command line input
    command_mode: bool,    // Whether we're in command input mode
    file_operation: Option<FileOperation>, // Current file operation state
    message: String,       // Status message to display
}

#[derive(Clone)]
enum FileOperation {
    Copy(String),  // Source file path
    Move(String),  // Source file path
    Delete(String), // File path to delete
}

#[derive(PartialEq, Clone, Copy)]
enum Tab {
    Dashboard,
    Files,
    Processes,
    Network,
    Ports,
    Services,
    System,
    Tools,
    Settings,
}

#[derive(Clone)]
struct ProcessInfo {
    pid: String,
    name: String,
    cpu: f32,
    memory: f32,
    user: String,
}

#[derive(Clone)]
struct NetworkInfo {
    interface: String,
    received_bytes: u64,
    transmitted_bytes: u64,
}

#[derive(Clone)]
struct PortInfo {
    port: String,
    protocol: String,
    process: String,
    local_address: String,
    state: String,
    pid: String, // Add PID for kill by port functionality
}

#[derive(Clone)]
struct FileInfo {
    name: String,
    size: String,
    date: String,
    permissions: String,
    is_directory: bool,
}

#[derive(Clone)]
struct ServiceInfo {
    name: String,
    status: String,
    pid: String,
}

#[derive(Clone)]
struct SystemInfo {
    hostname: String,
    uptime: String,
    cpu_count: usize,
    os_version: String,
}

impl AppState {
    fn new() -> AppState {
        let mut app = AppState {
            current_tab: Tab::Dashboard,
            selected_index: 0,
            show_help: false,
            processes: Vec::new(),
            networks: Vec::new(),
            ports: Vec::new(),
            all_ports: Vec::new(),
            files: Vec::new(),
            services: Vec::new(),
            system_info: SystemInfo {
                hostname: "Unknown".to_string(),
                uptime: "Unknown".to_string(),
                cpu_count: 0,
                os_version: "Unknown".to_string(),
            },
            current_path: PathBuf::from("/"),
            port_filter: String::new(),
            port_filter_mode: false,
            command_line: String::new(),
            command_mode: false,
            file_operation: None,
            message: String::new(),
        };
        app.refresh_data();
        app
    }

    fn refresh_data(&mut self) {
        self.refresh_processes();
        self.refresh_networks();
        self.refresh_ports();
        self.refresh_files();
        self.refresh_services();
        self.refresh_system_info();
        self.clamp_selected_index();
    }

    fn refresh_processes(&mut self) {
        let mut sys = System::new_all();
        sys.refresh_all();

        self.processes = sys
            .processes()
            .values()
            .map(|p| ProcessInfo {
                pid: p.pid().to_string(),
                name: p.name().to_string(),
                cpu: p.cpu_usage(),
                memory: p.memory() as f32 / 1024.0 / 1024.0,
                user: p
                    .user_id()
                    .map(|u| u.to_string())
                    .unwrap_or_else(|| "Unknown".to_string()),
            })
            .collect();

        self.processes
            .sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(Ordering::Equal));

        self.clamp_selected_index();
    }

    fn refresh_networks(&mut self) {
        let mut networks = Networks::new_with_refreshed_list();
        networks.refresh();

        self.networks = networks
            .iter()
            .map(|(name, data)| NetworkInfo {
                interface: name.to_string(),
                received_bytes: data.total_received(),
                transmitted_bytes: data.total_transmitted(),
            })
            .collect();

        self.networks.sort_by(|a, b| {
            b.received_bytes
                .cmp(&a.received_bytes)
                .then(b.transmitted_bytes.cmp(&a.transmitted_bytes))
        });

        self.clamp_selected_index();
    }

    fn refresh_ports(&mut self) {
        let output = Command::new("lsof").args(["-i", "-P", "-n"]).output().ok();

        self.all_ports = output
            .as_ref()
            .map(|output| String::from_utf8_lossy(&output.stdout))
            .map(|output| output.lines().skip(1).filter_map(parse_lsof_line).collect())
            .unwrap_or_default();

        self.apply_port_filter();
    }

    fn apply_port_filter(&mut self) {
        if self.port_filter.is_empty() {
            self.ports = self.all_ports.clone();
        } else {
            let filter_lower = self.port_filter.to_lowercase();
            self.ports = self
                .all_ports
                .iter()
                .filter(|port| {
                    port.port.to_lowercase().contains(&filter_lower)
                        || port.process.to_lowercase().contains(&filter_lower)
                        || port.protocol.to_lowercase().contains(&filter_lower)
                        || port.local_address.to_lowercase().contains(&filter_lower)
                })
                .cloned()
                .collect();
        }
        self.clamp_selected_index();
    }

    fn refresh_files(&mut self) {
        let path_str = self.current_path.to_string_lossy().to_string();
        let output = Command::new("ls")
            .args(["-la", &path_str])
            .output()
            .ok();

        self.files = output
            .as_ref()
            .map(|output| String::from_utf8_lossy(&output.stdout))
            .map(|output| output.lines().skip(1).filter_map(parse_ls_line).collect())
            .unwrap_or_default();

        // Add parent directory entry if not at root
        if self.current_path != PathBuf::from("/") {
            let parent = FileInfo {
                name: "..".to_string(),
                size: "DIR".to_string(),
                date: "".to_string(),
                permissions: "d".to_string(),
                is_directory: true,
            };
            self.files.insert(0, parent);
        }

        self.clamp_selected_index();
    }

    fn navigate_into_directory(&mut self) {
        if let Some(file) = self.files.get(self.selected_index) {
            if file.is_directory {
                if file.name == ".." {
                    // Go to parent directory
                    if let Some(parent) = self.current_path.parent() {
                        self.current_path = parent.to_path_buf();
                    }
                } else {
                    // Navigate into directory
                    let new_path = self.current_path.join(&file.name);
                    if new_path.is_dir() {
                        self.current_path = new_path;
                    }
                }
                self.selected_index = 0;
                self.refresh_files();
            }
        }
    }

    fn refresh_services(&mut self) {
        let output = Command::new("launchctl").args(["list"]).output().ok();

        self.services = output
            .as_ref()
            .map(|output| String::from_utf8_lossy(&output.stdout))
            .map(|output| {
                output
                    .lines()
                    .skip(1)
                    .filter_map(parse_launchctl_line)
                    .collect()
            })
            .unwrap_or_default();

        self.clamp_selected_index();
    }

    fn refresh_system_info(&mut self) {
        let mut sys = System::new_all();
        sys.refresh_all();

        self.system_info = SystemInfo {
            hostname: sysinfo::System::host_name().unwrap_or_else(|| "Unknown".to_string()),
            uptime: format!("{} days", sysinfo::System::uptime() / 86400),
            cpu_count: sys.cpus().len(),
            os_version: sysinfo::System::os_version().unwrap_or_else(|| "Unknown".to_string()),
        };
    }

    fn get_current_list_len(&self) -> usize {
        match self.current_tab {
            Tab::Files => self.files.len(),
            Tab::Processes => self.processes.len(),
            Tab::Network => self.networks.len(),
            Tab::Ports => self.ports.len(),
            Tab::Services => self.services.len(),
            Tab::System => 6,
            Tab::Tools => 8,
            _ => 0,
        }
    }

    fn kill_process_by_port(&mut self) {
        if let Some(port) = self.ports.get(self.selected_index) {
            if !port.pid.is_empty() && port.pid != "-" {
                let _ = Command::new("kill").arg("-9").arg(&port.pid).output();
                self.refresh_ports();
            }
        }
    }

    fn copy_file(&mut self) {
        if let Some(file) = self.files.get(self.selected_index) {
            if file.name != ".." {
                let source_path = self.current_path.join(&file.name);
                self.file_operation = Some(FileOperation::Copy(source_path.to_string_lossy().to_string()));
                self.message = format!("Copy: {} -> Enter destination path", file.name);
            }
        }
    }

    fn move_file(&mut self) {
        if let Some(file) = self.files.get(self.selected_index) {
            if file.name != ".." {
                let source_path = self.current_path.join(&file.name);
                self.file_operation = Some(FileOperation::Move(source_path.to_string_lossy().to_string()));
                self.message = format!("Move: {} -> Enter destination path", file.name);
            }
        }
    }

    fn delete_file(&mut self) {
        if let Some(file) = self.files.get(self.selected_index) {
            if file.name != ".." {
                let file_path = self.current_path.join(&file.name);
                self.file_operation = Some(FileOperation::Delete(file_path.to_string_lossy().to_string()));
                self.message = format!("Delete: {}? (y/n)", file.name);
            }
        }
    }

    fn execute_file_operation(&mut self, destination: &str) {
        match &self.file_operation {
            Some(FileOperation::Copy(source)) => {
                let src = PathBuf::from(source);
                let dst = PathBuf::from(destination);
                match fs::copy(&src, &dst) {
                    Ok(_) => {
                        self.message = format!("Copied {} to {}", src.display(), dst.display());
                        self.file_operation = None;
                        self.refresh_files();
                    }
                    Err(e) => {
                        self.message = format!("Error copying file: {}", e);
                    }
                }
            }
            Some(FileOperation::Move(source)) => {
                let src = PathBuf::from(source);
                let dst = PathBuf::from(destination);
                match fs::rename(&src, &dst) {
                    Ok(_) => {
                        self.message = format!("Moved {} to {}", src.display(), dst.display());
                        self.file_operation = None;
                        self.refresh_files();
                    }
                    Err(e) => {
                        self.message = format!("Error moving file: {}", e);
                    }
                }
            }
            Some(FileOperation::Delete(path)) => {
                let path_buf = PathBuf::from(path);
                if path_buf.is_dir() {
                    match fs::remove_dir_all(&path_buf) {
                        Ok(_) => {
                            self.message = format!("Deleted directory: {}", path);
                            self.file_operation = None;
                            self.refresh_files();
                        }
                        Err(e) => {
                            self.message = format!("Error deleting directory: {}", e);
                        }
                    }
                } else {
                    match fs::remove_file(&path_buf) {
                        Ok(_) => {
                            self.message = format!("Deleted file: {}", path);
                            self.file_operation = None;
                            self.refresh_files();
                        }
                        Err(e) => {
                            self.message = format!("Error deleting file: {}", e);
                        }
                    }
                }
            }
            None => {}
        }
    }

    fn execute_command(&mut self) {
        if !self.command_line.trim().is_empty() {
            let output = Command::new("sh")
                .arg("-c")
                .arg(&self.command_line)
                .output();

            match output {
                Ok(result) => {
                    if result.status.success() {
                        let stdout = String::from_utf8_lossy(&result.stdout);
                        self.message = format!("Command executed. Output: {}", stdout.lines().take(3).collect::<Vec<_>>().join(" "));
                    } else {
                        let stderr = String::from_utf8_lossy(&result.stderr);
                        self.message = format!("Command failed: {}", stderr.lines().next().unwrap_or("Unknown error"));
                    }
                }
                Err(e) => {
                    self.message = format!("Error executing command: {}", e);
                }
            }
            self.command_line.clear();
            self.command_mode = false;
            self.refresh_data();
        }
    }

    fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    fn move_down(&mut self) {
        let max_index = self.get_current_list_len().saturating_sub(1);
        if self.selected_index < max_index {
            self.selected_index += 1;
        }
    }

    fn next_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::Dashboard => Tab::Files,
            Tab::Files => Tab::Processes,
            Tab::Processes => Tab::Network,
            Tab::Network => Tab::Ports,
            Tab::Ports => Tab::Services,
            Tab::Services => Tab::System,
            Tab::System => Tab::Tools,
            Tab::Tools => Tab::Settings,
            Tab::Settings => Tab::Dashboard,
        };
        self.selected_index = 0;
    }

    fn prev_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::Dashboard => Tab::Settings,
            Tab::Files => Tab::Dashboard,
            Tab::Processes => Tab::Files,
            Tab::Network => Tab::Processes,
            Tab::Ports => Tab::Network,
            Tab::Services => Tab::Ports,
            Tab::System => Tab::Services,
            Tab::Tools => Tab::System,
            Tab::Settings => Tab::Tools,
        };
        self.selected_index = 0;
    }

    fn clamp_selected_index(&mut self) {
        let len = self.get_current_list_len();
        if len == 0 {
            self.selected_index = 0;
        } else if self.selected_index >= len {
            self.selected_index = len - 1;
        }
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let mut app_state = AppState::new();

    loop {
        terminal.draw(|f| ui(f, &app_state))?;

        if let Event::Key(key) = event::read()? {
            // Handle command mode or file operation input
            if app_state.command_mode {
                match key.code {
                    KeyCode::Esc => {
                        app_state.command_mode = false;
                        app_state.command_line.clear();
                        app_state.message.clear();
                    }
                    KeyCode::Enter => {
                        app_state.execute_command();
                    }
                    KeyCode::Char(c) => {
                        if c == '\x08' || c == '\x7f' {
                            // Backspace
                            app_state.command_line.pop();
                        } else if !c.is_control() {
                            app_state.command_line.push(c);
                        }
                    }
                    _ => {}
                }
                continue;
            } else if app_state.file_operation.is_some() {
                // Handle file operation input
                match key.code {
                    KeyCode::Esc => {
                        app_state.file_operation = None;
                        app_state.message.clear();
                    }
                    KeyCode::Enter => {
                        if let Some(FileOperation::Delete(_)) = app_state.file_operation {
                            // For delete, we need 'y' confirmation
                            continue;
                        } else {
                            // For copy/move, use current path or command_line as destination
                            let dest = if app_state.command_line.is_empty() {
                                app_state.current_path.to_string_lossy().to_string()
                            } else {
                                app_state.command_line.clone()
                            };
                            app_state.execute_file_operation(&dest);
                            app_state.command_line.clear();
                        }
                    }
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        if let Some(FileOperation::Delete(path)) = app_state.file_operation.clone() {
                            app_state.execute_file_operation(&path);
                            app_state.command_line.clear();
                        }
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        if let Some(FileOperation::Delete(_)) = app_state.file_operation {
                            app_state.file_operation = None;
                            app_state.message.clear();
                        }
                    }
                    KeyCode::Char(c) => {
                        if c == '\x08' || c == '\x7f' {
                            // Backspace
                            app_state.command_line.pop();
                        } else if !c.is_control() {
                            app_state.command_line.push(c);
                        }
                    }
                    _ => {}
                }
                continue;
            }

            // Normal mode key handling
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('h') => app_state.show_help = !app_state.show_help,
                KeyCode::Char('r') => app_state.refresh_data(),
                KeyCode::Char(':') => {
                    // Enter command mode
                    app_state.command_mode = true;
                    app_state.command_line.clear();
                    app_state.message = "Command: ".to_string();
                }
                KeyCode::Up => app_state.move_up(),
                KeyCode::Down => app_state.move_down(),
                KeyCode::Left => app_state.prev_tab(),
                KeyCode::Right => app_state.next_tab(),
                KeyCode::Tab => app_state.next_tab(),
                KeyCode::Enter if app_state.current_tab == Tab::Files && app_state.file_operation.is_none() => {
                    app_state.navigate_into_directory();
                }
                KeyCode::Char('c') if app_state.current_tab == Tab::Files && app_state.file_operation.is_none() => {
                    app_state.copy_file();
                }
                KeyCode::Char('m') if app_state.current_tab == Tab::Files && app_state.file_operation.is_none() => {
                    app_state.move_file();
                }
                KeyCode::Char('d') if app_state.current_tab == Tab::Files && app_state.file_operation.is_none() => {
                    app_state.delete_file();
                }
                KeyCode::Char('k') if app_state.current_tab == Tab::Processes => {
                    if let Some(process) = app_state.processes.get(app_state.selected_index) {
                        let _ = Command::new("kill").arg("-9").arg(&process.pid).output();
                        app_state.refresh_processes();
                    }
                }
                KeyCode::Char('K') if app_state.current_tab == Tab::Processes => {
                    if let Some(process) = app_state.processes.get(app_state.selected_index) {
                        let _ = Command::new("kill").arg("-TERM").arg(&process.pid).output();
                        app_state.refresh_processes();
                    }
                }
                KeyCode::Char('k') if app_state.current_tab == Tab::Ports && !app_state.port_filter_mode => {
                    app_state.kill_process_by_port();
                }
                KeyCode::Char('/') if app_state.current_tab == Tab::Ports => {
                    // Enter filter mode
                    app_state.port_filter_mode = true;
                }
                KeyCode::Esc if app_state.current_tab == Tab::Ports => {
                    // Exit filter mode and clear filter
                    app_state.port_filter_mode = false;
                    app_state.port_filter.clear();
                    app_state.apply_port_filter();
                }
                KeyCode::Char(c) if app_state.current_tab == Tab::Ports && app_state.port_filter_mode => {
                    // Only filter when in filter mode
                    if c == '\x08' || c == '\x7f' {
                        // Backspace
                        app_state.port_filter.pop();
                        app_state.apply_port_filter();
                    } else if c == '\r' || c == '\n' {
                        // Enter to exit filter mode (keep filter active)
                        app_state.port_filter_mode = false;
                    } else if !c.is_control() {
                        // Add character to filter
                        app_state.port_filter.push(c);
                        app_state.apply_port_filter();
                    }
                }
                KeyCode::Char('s') if app_state.current_tab == Tab::Services => {
                    if let Some(service) = app_state.services.get(app_state.selected_index) {
                        let _ = Command::new("launchctl")
                            .arg("start")
                            .arg(&service.name)
                            .output();
                        app_state.refresh_services();
                    }
                }
                KeyCode::Char('S') if app_state.current_tab == Tab::Services => {
                    if let Some(service) = app_state.services.get(app_state.selected_index) {
                        let _ = Command::new("launchctl")
                            .arg("stop")
                            .arg(&service.name)
                            .output();
                        app_state.refresh_services();
                    }
                }
                KeyCode::Char('1') => app_state.current_tab = Tab::Dashboard,
                KeyCode::Char('2') => app_state.current_tab = Tab::Files,
                KeyCode::Char('3') => app_state.current_tab = Tab::Processes,
                KeyCode::Char('4') => app_state.current_tab = Tab::Network,
                KeyCode::Char('5') => app_state.current_tab = Tab::Ports,
                KeyCode::Char('6') => app_state.current_tab = Tab::Services,
                KeyCode::Char('7') => app_state.current_tab = Tab::System,
                KeyCode::Char('8') => app_state.current_tab = Tab::Tools,
                KeyCode::Char('9') => app_state.current_tab = Tab::Settings,
                _ => {}
            }
        }
    }
}

fn ui(f: &mut Frame, app_state: &AppState) {
    // Determine if we need extra space for command line or file operation
    let mut constraints = vec![
        Constraint::Length(3), // Header
        Constraint::Min(0),    // Main content
    ];
    
    if app_state.command_mode || app_state.file_operation.is_some() {
        constraints.push(Constraint::Length(3)); // Command/Operation input line
    }
    
    constraints.push(Constraint::Length(3)); // Footer
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(f.size());

    // Header
    let header = Paragraph::new("Peek - Terminal System Admin Dashboard")
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // Main content
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20), // Sidebar
            Constraint::Percentage(80), // Content
        ])
        .split(chunks[1]);

    // Sidebar with tabs
    let sidebar_items = vec![
        "1. Dashboard",
        "2. Files",
        "3. Processes",
        "4. Network",
        "5. Ports",
        "6. Services",
        "7. System",
        "8. Tools",
        "9. Settings",
    ];

    let sidebar = List::new(sidebar_items)
        .block(Block::default().title("Tabs").borders(Borders::ALL))
        .highlight_style(Style::default().fg(Color::Yellow));
    f.render_widget(sidebar, main_chunks[0]);

    // Content area
    match app_state.current_tab {
        Tab::Dashboard => render_dashboard(f, main_chunks[1], app_state),
        Tab::Files => render_files(f, main_chunks[1], app_state),
        Tab::Processes => render_processes(f, main_chunks[1], app_state),
        Tab::Network => render_network(f, main_chunks[1], app_state),
        Tab::Ports => render_ports(f, main_chunks[1], app_state),
        Tab::Services => render_services(f, main_chunks[1], app_state),
        Tab::System => render_system(f, main_chunks[1], app_state),
        Tab::Tools => render_tools(f, main_chunks[1], app_state),
        Tab::Settings => render_settings(f, main_chunks[1], app_state),
    }

    // Command line or file operation input
    let mut footer_idx = 2;
    if app_state.command_mode || app_state.file_operation.is_some() {
        let input_text = if app_state.command_mode {
            format!(":{}", app_state.command_line)
        } else if let Some(FileOperation::Copy(_)) = app_state.file_operation {
            format!("{} Destination: {}", app_state.message, app_state.command_line)
        } else if let Some(FileOperation::Move(_)) = app_state.file_operation {
            format!("{} Destination: {}", app_state.message, app_state.command_line)
        } else if let Some(FileOperation::Delete(_)) = app_state.file_operation {
            app_state.message.clone()
        } else {
            String::new()
        };
        
        let input_widget = Paragraph::new(input_text)
            .style(Style::default().fg(if app_state.command_mode { Color::Green } else { Color::Yellow }))
            .block(Block::default().borders(Borders::ALL).title(if app_state.command_mode { "Command Line" } else { "File Operation" }));
        f.render_widget(input_widget, chunks[2]);
        footer_idx = 3;
    }
    
    // Status message or footer
    let footer_text = if !app_state.message.is_empty() && !app_state.command_mode && app_state.file_operation.is_none() {
        app_state.message.clone()
    } else if app_state.show_help {
        "Help: q=quit, h=toggle help, r=refresh, : =command, ↑↓=navigate, ←→/Tab=tabs, Enter=open dir, c=copy, m=move, d=delete (Files), k=kill, K=terminate, s/S=start/stop service, /=filter (Ports)".to_string()
    } else {
        "Press 'h' for help, 'q' to quit, ':' for command line, 'r' to refresh, arrow keys to navigate".to_string()
    };

    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[footer_idx]);
}

fn render_dashboard(f: &mut Frame, area: Rect, app_state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // System stats
    let stats_text = [
        format!("Hostname: {}", app_state.system_info.hostname),
        format!("Uptime: {}", app_state.system_info.uptime),
        format!("CPU Cores: {}", app_state.system_info.cpu_count),
        format!("OS: {}", app_state.system_info.os_version),
        format!("Processes: {}", app_state.processes.len()),
        format!("Ports: {}", app_state.ports.len()),
        format!("Services: {}", app_state.services.len()),
    ];

    let stats = Paragraph::new(stats_text.join("\n")).block(
        Block::default()
            .title("System Information")
            .borders(Borders::ALL),
    );
    f.render_widget(stats, chunks[0]);

    // Quick actions
    let actions_text = [
        "Quick Actions:",
        "• Press 'r' to refresh all data",
        "• Press 'h' for help",
        "• Press 'q' to quit",
        "• Use number keys (1-9) to switch tabs",
        "• Use arrow keys to navigate",
        "Process Management:",
        "• Press 'k' to kill selected process",
        "• Press 'K' to terminate selected process",
        "Service Management:",
        "• Press 's' to start selected service",
        "• Press 'S' to stop selected service",
    ];

    let actions = Paragraph::new(actions_text.join("\n")).block(
        Block::default()
            .title("Quick Actions")
            .borders(Borders::ALL),
    );
    f.render_widget(actions, chunks[1]);
}

fn render_files(f: &mut Frame, area: Rect, app_state: &AppState) {
    let items: Vec<ListItem> = app_state
        .files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let style = if i == app_state.selected_index {
                Style::default().fg(Color::Blue)
            } else {
                Style::default()
            };
            let icon = if file.is_directory { "📁" } else { "📄" };
            ListItem::new(format!(
                "{} {} {} {} {}",
                icon, file.name, file.size, file.date, file.permissions
            ))
            .style(style)
        })
        .collect();

    let path_str = app_state.current_path.to_string_lossy();
    let title = format!("Files ({})", path_str);
    let list = List::new(items)
        .block(Block::default().title(title).borders(Borders::ALL));
    f.render_widget(list, area);
}

fn render_processes(f: &mut Frame, area: Rect, app_state: &AppState) {
    let rows: Vec<Row> = app_state
        .processes
        .iter()
        .enumerate()
        .map(|(i, process)| {
            let style = if i == app_state.selected_index {
                Style::default().fg(Color::Blue)
            } else {
                Style::default()
            };
            Row::new(vec![
                process.pid.clone(),
                process.name.clone(),
                format!("{:.1}%", process.cpu),
                format!("{:.1}MB", process.memory),
                process.user.clone(),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        &[
            Constraint::Length(8),
            Constraint::Length(15),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Min(10),
        ],
    )
    .header(
        Row::new(vec!["PID", "Name", "CPU%", "Memory", "User"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(Block::default().title("Processes").borders(Borders::ALL));

    f.render_widget(table, area);
}

fn render_network(f: &mut Frame, area: Rect, app_state: &AppState) {
    let rows: Vec<Row> = app_state
        .networks
        .iter()
        .enumerate()
        .map(|(i, interface)| {
            let style = if i == app_state.selected_index {
                Style::default().fg(Color::Blue)
            } else {
                Style::default()
            };

            Row::new(vec![
                interface.interface.clone(),
                format_bytes(interface.received_bytes),
                format_bytes(interface.transmitted_bytes),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        &[
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ],
    )
    .header(
        Row::new(vec!["Interface", "Received", "Transmitted"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(
        Block::default()
            .title("Network Interfaces")
            .borders(Borders::ALL),
    );

    f.render_widget(table, area);
}

fn render_ports(f: &mut Frame, area: Rect, app_state: &AppState) {
    let items: Vec<ListItem> = app_state
        .ports
        .iter()
        .enumerate()
        .map(|(i, port)| {
            let style = if i == app_state.selected_index {
                Style::default().fg(Color::Blue)
            } else {
                Style::default()
            };
            ListItem::new(format!(
                "🌐 {}:{} {} {} {} [PID: {}]",
                port.protocol, port.port, port.process, port.local_address, port.state, port.pid
            ))
            .style(style)
        })
        .collect();

    let title = if app_state.port_filter_mode {
        format!("Network Ports (Filter: '{}' | Press Enter to apply, Esc to cancel)", app_state.port_filter)
    } else if app_state.port_filter.is_empty() {
        "Network Ports (Press 'k' to kill, '/' to filter)".to_string()
    } else {
        format!("Network Ports (Filtered: '{}' | Press '/' to filter, Esc to clear)", app_state.port_filter)
    };

    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL),
    );
    f.render_widget(list, area);
}

fn render_services(f: &mut Frame, area: Rect, app_state: &AppState) {
    let items: Vec<ListItem> = app_state
        .services
        .iter()
        .enumerate()
        .map(|(i, service)| {
            let style = if i == app_state.selected_index {
                Style::default().fg(Color::Blue)
            } else {
                Style::default()
            };
            let icon = if service.status == "Running" {
                "▶️"
            } else {
                "⏹️"
            };
            ListItem::new(format!(
                "{} {} {} {}",
                icon, service.name, service.status, service.pid
            ))
            .style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().title("Services").borders(Borders::ALL));
    f.render_widget(list, area);
}

fn render_system(f: &mut Frame, area: Rect, app_state: &AppState) {
    let info_text = [
        format!("Hostname: {}", app_state.system_info.hostname),
        format!("Uptime: {}", app_state.system_info.uptime),
        format!("CPU Cores: {}", app_state.system_info.cpu_count),
        format!("OS Version: {}", app_state.system_info.os_version),
        format!("Processes: {}", app_state.processes.len()),
        format!("Ports: {}", app_state.ports.len()),
        format!("Services: {}", app_state.services.len()),
    ];

    let paragraph = Paragraph::new(info_text.join("\n")).block(
        Block::default()
            .title("System Information")
            .borders(Borders::ALL),
    );
    f.render_widget(paragraph, area);
}

fn render_tools(f: &mut Frame, area: Rect, _app_state: &AppState) {
    let tools_text = [
        "Available Tools:",
        "🔍 Disk Analyzer - Analyze disk usage",
        "📡 Network Monitor - Monitor network traffic",
        "⚡ Process Killer - Kill problematic processes",
        "🧹 System Cleaner - Clean temporary files",
        "💾 Backup Manager - Manage system backups",
        "🔒 Security Scanner - Scan for security issues",
        "📊 Performance Monitor - Monitor system performance",
        "🛠️ System Repair - Repair system issues",
        "Press 'k' to kill selected process",
        "Press 'K' to terminate selected process",
        "Press 's' to start selected service",
        "Press 'S' to stop selected service",
    ];

    let tools = Paragraph::new(tools_text.join("\n"))
        .block(Block::default().title("Admin Tools").borders(Borders::ALL));
    f.render_widget(tools, area);
}

fn render_settings(f: &mut Frame, area: Rect, _app_state: &AppState) {
    let settings_text = vec![
        "Settings:",
        "• Auto-refresh: Every 2 seconds",
        "• Theme: Terminal (Blue/Cyan)",
        "• Navigation: Arrow keys",
        "• Tabs: Number keys (1-9)",
        "Keyboard Shortcuts:",
        "• q - Quit application",
        "• h - Toggle help",
        "• r - Refresh data",
        "• ↑↓ - Navigate items",
        "• ←→ - Switch tabs",
        "• k - Kill process",
        "• K - Terminate process",
        "• s - Start service",
        "• S - Stop service",
    ];

    let settings = Paragraph::new(settings_text.join("\n"))
        .block(Block::default().title("Settings").borders(Borders::ALL));
    f.render_widget(settings, area);
}

fn parse_lsof_line(line: &str) -> Option<PortInfo> {
    if line.is_empty() || line.starts_with("COMMAND") {
        return None;
    }

    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 9 {
        return None;
    }

    let pid = parts.get(1).unwrap_or(&"-").to_string();
    let name_field = parts[8..].join(" ");
    let mut local_address = name_field.clone();
    let mut state = String::from("N/A");

    if let Some((addr, rest)) = name_field.split_once("->") {
        local_address = addr.trim().to_string();
        state = rest.trim().to_string();
    } else if let Some((addr, rest)) = name_field.split_once(" (") {
        local_address = addr.trim().to_string();
        state = rest.trim_end_matches(')').trim().to_string();
    }

    let port = local_address
        .rsplit(':')
        .next()
        .unwrap_or("N/A")
        .to_string();

    Some(PortInfo {
        process: parts[0].to_string(),
        protocol: parts.get(7).unwrap_or(&"Unknown").to_string(),
        local_address,
        state,
        port,
        pid,
    })
}

fn parse_ls_line(line: &str) -> Option<FileInfo> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 9 {
        return None;
    }

    let permissions = parts[0];
    let name = parts[8..].join(" ");

    Some(FileInfo {
        name,
        size: parts[4].to_string(),
        date: parts[5..8].join(" "),
        permissions: permissions.to_string(),
        is_directory: permissions.starts_with('d'),
    })
}

fn parse_launchctl_line(line: &str) -> Option<ServiceInfo> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }

    let status = if parts[0] == "-" {
        "Stopped"
    } else {
        "Running"
    };

    Some(ServiceInfo {
        name: parts[2].to_string(),
        status: status.to_string(),
        pid: parts[0].to_string(),
    })
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit_index = 0;

    while value >= 1024.0 && unit_index < UNITS.len() - 1 {
        value /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{bytes} {}", UNITS[unit_index])
    } else {
        format!("{value:.1} {}", UNITS[unit_index])
    }
}
