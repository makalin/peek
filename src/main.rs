use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
};
use std::{cmp::Ordering, error::Error, io, process::Command};
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
    files: Vec<FileInfo>,
    services: Vec<ServiceInfo>,
    system_info: SystemInfo,
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
            files: Vec::new(),
            services: Vec::new(),
            system_info: SystemInfo {
                hostname: "Unknown".to_string(),
                uptime: "Unknown".to_string(),
                cpu_count: 0,
                os_version: "Unknown".to_string(),
            },
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

        self.ports = output
            .as_ref()
            .map(|output| String::from_utf8_lossy(&output.stdout))
            .map(|output| output.lines().skip(1).filter_map(parse_lsof_line).collect())
            .unwrap_or_default();

        self.clamp_selected_index();
    }

    fn refresh_files(&mut self) {
        let output = Command::new("ls").args(["-la", "/tmp"]).output().ok();

        self.files = output
            .as_ref()
            .map(|output| String::from_utf8_lossy(&output.stdout))
            .map(|output| output.lines().skip(1).filter_map(parse_ls_line).collect())
            .unwrap_or_default();

        self.clamp_selected_index();
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
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('h') => app_state.show_help = !app_state.show_help,
                KeyCode::Char('r') => app_state.refresh_data(),
                KeyCode::Up => app_state.move_up(),
                KeyCode::Down => app_state.move_down(),
                KeyCode::Left => app_state.prev_tab(),
                KeyCode::Right => app_state.next_tab(),
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
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer
        ])
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

    // Footer
    let footer_text = if app_state.show_help {
        "Help: q=quit, h=toggle help, r=refresh, ↑↓=navigate, ←→=tabs, k=kill process, K=terminate, s=start service, S=stop service"
    } else {
        "Press 'h' for help, 'q' to quit, 'r' to refresh, arrow keys to navigate"
    };

    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
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

    let list = List::new(items).block(Block::default().title("Files (/tmp)").borders(Borders::ALL));
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
                "🌐 {}:{} {} {} {}",
                port.protocol, port.port, port.process, port.local_address, port.state
            ))
            .style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title("Network Ports")
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
