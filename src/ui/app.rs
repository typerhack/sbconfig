// src/ui/app.rs
// Main application state and event loop

use crate::db::{decrypt, Database};
use crate::error::Result;
use crate::singbox;
use crate::singbox::{generate_config_json, RoutingPreset, ServiceStatus};
use crate::ssh::{self, KeyType};
use base64::{engine::general_purpose, Engine as _};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use rand::RngCore;
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::fs;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SetupStep {
    SingboxCheck,
    PortConfig,
    ModeSelect,
    Confirm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ServerMode {
    Production,
    Development,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SetupFocus {
    PortChoice,
    ModeChoice,
    DomainInput,
    IpInput,
}

#[derive(Debug, Clone)]
struct SetupState {
    step: SetupStep,
    existing_port: bool,
    port_input: InputField,
    mode: ServerMode,
    domain_input: InputField,
    ip_input: InputField,
    focus: SetupFocus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UserCreateFocus {
    Username,
    KeyType,
}

#[derive(Debug, Clone)]
struct UserCreateState {
    username_input: InputField,
    key_type: KeyType,
    focus: UserCreateFocus,
    error: Option<String>,
}

#[derive(Debug, Clone)]
struct UserDeleteState {
    error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigPlatform {
    Ios,
    Android,
    Windows,
    Macos,
    Linux,
    Generic,
}

impl ConfigPlatform {
    fn all() -> &'static [ConfigPlatform] {
        &[
            Self::Ios,
            Self::Android,
            Self::Windows,
            Self::Macos,
            Self::Linux,
            Self::Generic,
        ]
    }

    fn label(self) -> &'static str {
        match self {
            Self::Ios => "iOS",
            Self::Android => "Android",
            Self::Windows => "Windows",
            Self::Macos => "macOS",
            Self::Linux => "Linux",
            Self::Generic => "Generic",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Ios => "sing-box app from App Store",
            Self::Android => "sing-box app from Play Store",
            Self::Windows => "sing-box for Windows",
            Self::Macos => "sing-box for macOS",
            Self::Linux => "sing-box CLI",
            Self::Generic => "Platform-agnostic config",
        }
    }
}

#[derive(Debug, Clone)]
struct ConfigState {
    platform_index: usize,
    user_id: Option<i64>,
    output_json: Option<String>,
    error: Option<String>,
    scroll: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsFocus {
    ChangeServerAddress,
    UpdateSshPort,
    ChangeMode,
    RemoveSshPort,
    ViewSshConfig,
}

#[derive(Debug, Clone)]
struct SettingsEditState {
    server_input: InputField,
    ssh_port_input: InputField,
    mode: ServerMode,
    focus: SettingsFocus,
    error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsModal {
    None,
    ServerAddress,
    SshPort,
    Mode,
    RemoveSshPort,
    ViewSshConfig,
}

#[derive(Debug, Clone)]
struct LogsState {
    session_index: usize,
    scroll: usize,
}

#[derive(Debug, Clone)]
struct SingboxTaskState {
    title: String,
    running: bool,
    logs: Vec<String>,
    result: Option<String>,
    error: Option<String>,
    step: usize,
    total_steps: usize,
}

#[derive(Debug, Clone)]
enum SingboxTaskEvent {
    Log(String),
    Progress { step: usize, total: usize },
    Done(String),
    Error(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SingboxAction {
    Start,
    Stop,
    Restart,
    Enable,
    Disable,
    Refresh,
    Install,
    Reinstall,
    Uninstall,
}

impl SingboxAction {
    fn label(self) -> &'static str {
        match self {
            Self::Start => "Start Service",
            Self::Stop => "Stop Service",
            Self::Restart => "Restart Service",
            Self::Enable => "Enable Service",
            Self::Disable => "Disable Service",
            Self::Refresh => "Refresh Status",
            Self::Install => "Install sing-box",
            Self::Reinstall => "Reinstall sing-box",
            Self::Uninstall => "Uninstall sing-box",
        }
    }
}

#[derive(Debug, Clone)]
struct InputField {
    value: String,
    cursor: usize,
}

impl InputField {
    fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
            cursor: value.len(),
        }
    }

    fn set(&mut self, value: &str) {
        self.value = value.to_string();
        self.cursor = self.value.len();
    }

    fn insert_char(&mut self, ch: char) {
        if self.cursor >= self.value.len() {
            self.value.push(ch);
        } else {
            self.value.insert(self.cursor, ch);
        }
        self.cursor = (self.cursor + 1).min(self.value.len());
    }

    fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.cursor -= 1;
        if self.cursor < self.value.len() {
            self.value.remove(self.cursor);
        }
    }
}

fn render_input_spans(field: &InputField, width: usize) -> Vec<Span<'static>> {
    let cursor_style = Style::default().add_modifier(Modifier::REVERSED);
    let target_width = width.max(1);
    let mut display = field.value.clone();
    if display.len() < target_width {
        display.push_str(&"_".repeat(target_width - display.len()));
    }
    let cursor = field.cursor.min(display.len().saturating_sub(1));
    let before = &display[..cursor];
    let after = &display[cursor..];
    let cursor_char = after.chars().next().unwrap_or('_');
    let after_rest = if after.is_empty() || cursor + cursor_char.len_utf8() > display.len() {
        ""
    } else {
        &display[cursor + cursor_char.len_utf8()..]
    };
    vec![
        Span::raw(before.to_string()),
        Span::styled(cursor_char.to_string(), cursor_style),
        Span::raw(after_rest.to_string()),
    ]
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return Vec::new();
    }
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let next_len = if current.is_empty() {
            word.len()
        } else {
            current.len() + 1 + word.len()
        };
        if next_len > width && !current.is_empty() {
            lines.push(current.clone());
            current.clear();
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

/// Menu items on the dashboard
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItem {
    Singbox,
    Users,
    Configs,
    Settings,
    Logs,
    Quit,
}

impl MenuItem {
    fn all() -> &'static [MenuItem] {
        &[
            MenuItem::Singbox,
            MenuItem::Users,
            MenuItem::Configs,
            MenuItem::Settings,
            MenuItem::Logs,
            MenuItem::Quit,
        ]
    }

    fn label(&self) -> &'static str {
        match self {
            MenuItem::Singbox => "sing-box Status",
            MenuItem::Users => "Manage Users",
            MenuItem::Configs => "Generate Configs",
            MenuItem::Settings => "Settings",
            MenuItem::Logs => "View Logs",
            MenuItem::Quit => "Quit",
        }
    }

    fn screen(self) -> Option<Screen> {
        match self {
            MenuItem::Singbox => Some(Screen::SingboxStatus),
            MenuItem::Users => Some(Screen::Users),
            MenuItem::Configs => Some(Screen::Configs),
            MenuItem::Settings => Some(Screen::Settings),
            MenuItem::Logs => Some(Screen::Logs),
            MenuItem::Quit => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Screen {
    Setup,
    Dashboard,
    SingboxStatus,
    Users,
    UserCreate,
    UserDelete,
    Configs,
    ConfigPlatform,
    ConfigOutput,
    Settings,
    Logs,
}

pub struct App {
    db: Database,
    running: bool,
    screen: Screen,
    screen_stack: Vec<Screen>,
    // Dashboard state
    menu_index: usize,
    // Users screen state
    user_list_index: usize,
    setup_state: SetupState,
    user_create_state: UserCreateState,
    user_delete_state: UserDeleteState,
    config_state: ConfigState,
    settings_edit_state: SettingsEditState,
    settings_modal: SettingsModal,
    settings_modal_error: Option<String>,
    settings_modal_scroll: u16,
    logs_state: LogsState,
    status_message: Option<String>,
    singbox_action_index: usize,
    singbox_message: Option<String>,
    singbox_status_output: Option<String>,
    singbox_progress: Option<String>,
    singbox_task: Option<SingboxTaskState>,
    singbox_task_rx: Option<Receiver<SingboxTaskEvent>>,
}

impl App {
    fn app_version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    pub fn new(db: Database) -> Self {
        let mut setup_state = SetupState {
            step: SetupStep::SingboxCheck,
            existing_port: true,
            port_input: InputField::new("7344"),
            mode: ServerMode::Production,
            domain_input: InputField::new(""),
            ip_input: InputField::new(""),
            focus: SetupFocus::PortChoice,
        };
        let ssh_port = db
            .get_setting("ssh_port")
            .ok()
            .flatten()
            .unwrap_or_else(|| "7344".to_string());
        setup_state.port_input.set(&ssh_port);

        let server_addr = db
            .get_setting("server_address")
            .ok()
            .flatten()
            .unwrap_or_default();
        if server_addr != "localhost" {
            setup_state.domain_input.set(&server_addr);
        }

        let setup_complete = db
            .get_setting("setup_complete")
            .ok()
            .flatten()
            .map(|value| value == "true")
            .unwrap_or(false);

        let initial_screen = if setup_complete {
            Screen::Dashboard
        } else {
            Screen::Setup
        };

        Self {
            db,
            running: true,
            screen: initial_screen,
            screen_stack: Vec::new(),
            menu_index: 0,
            user_list_index: 0,
            setup_state,
            user_create_state: UserCreateState {
                username_input: InputField::new(""),
                key_type: KeyType::Ed25519,
                focus: UserCreateFocus::Username,
                error: None,
            },
            user_delete_state: UserDeleteState { error: None },
            config_state: ConfigState {
                platform_index: 0,
                user_id: None,
                output_json: None,
                error: None,
                scroll: 0,
            },
            settings_edit_state: SettingsEditState {
                server_input: InputField::new(""),
                ssh_port_input: InputField::new(&ssh_port),
                mode: ServerMode::Production,
                focus: SettingsFocus::ChangeServerAddress,
                error: None,
            },
            settings_modal: SettingsModal::None,
            settings_modal_error: None,
            settings_modal_scroll: 0,
            logs_state: LogsState {
                session_index: 0,
                scroll: 0,
            },
            status_message: None,
            singbox_action_index: 0,
            singbox_message: None,
            singbox_status_output: None,
            singbox_progress: None,
            singbox_task: None,
            singbox_task_rx: None,
        }
    }

    /// Main event loop
    pub fn run(&mut self, terminal: &mut super::Tui) -> Result<()> {
        while self.running {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Header
        let title = match self.screen {
            Screen::Setup => format!(" sbconfig v{} - Setup ", Self::app_version()),
            Screen::Dashboard => format!(" sbconfig v{} - Dashboard ", Self::app_version()),
            Screen::SingboxStatus => {
                format!(" sbconfig v{} > sing-box Status ", Self::app_version())
            }
            Screen::Users => format!(" sbconfig v{} > User Management ", Self::app_version()),
            Screen::UserCreate => format!(" sbconfig v{} > Create User ", Self::app_version()),
            Screen::UserDelete => format!(" sbconfig v{} > Delete User ", Self::app_version()),
            Screen::Configs => format!(" sbconfig v{} > Config Generation ", Self::app_version()),
            Screen::ConfigPlatform => {
                format!(" sbconfig v{} > Select Platform ", Self::app_version())
            }
            Screen::ConfigOutput => format!(" sbconfig v{} > Config Output ", Self::app_version()),
            Screen::Settings => format!(" sbconfig v{} > Settings ", Self::app_version()),
            Screen::Logs => format!(" sbconfig v{} > Logs ", Self::app_version()),
        };
        let header = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(title);
        frame.render_widget(header, chunks[0]);

        // Content based on current screen
        match self.screen {
            Screen::Setup => self.draw_setup(frame, chunks[1]),
            Screen::Dashboard => self.draw_dashboard(frame, chunks[1]),
            Screen::SingboxStatus => self.draw_singbox_status(frame, chunks[1]),
            Screen::Users => self.draw_users(frame, chunks[1]),
            Screen::UserCreate => self.draw_user_create(frame, chunks[1]),
            Screen::UserDelete => self.draw_user_delete(frame, chunks[1]),
            Screen::Configs => self.draw_configs(frame, chunks[1]),
            Screen::ConfigPlatform => self.draw_config_platform(frame, chunks[1]),
            Screen::ConfigOutput => self.draw_config_output(frame, chunks[1]),
            Screen::Settings => self.draw_settings(frame, chunks[1]),
            Screen::Logs => self.draw_logs(frame, chunks[1]),
        }

        // Footer with navigation hints
        let footer_text = match self.screen {
            Screen::Setup => {
                " [Up/Down] Navigate  [Left/Right] Toggle  [Enter] Continue  [Esc] Back "
            }
            Screen::Dashboard => " [Up/Down] Navigate  [Enter] Select  [q] Quit ",
            Screen::SingboxStatus => {
                " [Up/Down] Navigate  [Enter] Run  [1-9] Quick  [Esc] Back  [q] Quit "
            }
            Screen::Users => {
                " [Up/Down] Navigate  [a] Add  [d] Delete  [Enter] Toggle  [Esc] Back "
            }
            Screen::UserCreate => {
                " [Up/Down] Navigate  [Left/Right] Toggle  [Enter] Create  [Esc] Cancel "
            }
            Screen::UserDelete => " [y] Confirm  [n] Cancel  [Esc] Back ",
            Screen::ConfigPlatform => " [Up/Down] Navigate  [Enter] Select  [Esc] Back ",
            Screen::ConfigOutput => " [Up/Down] Scroll  [Esc] Back ",
            Screen::Settings => " [Up/Down] Navigate  [Enter] Edit  [Esc] Back  [q] Quit ",
            _ => " [Esc] Back  [b] Back  [q] Quit ",
        };
        let footer = Paragraph::new(footer_text)
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(footer, chunks[2]);

        if let Some(task) = &self.singbox_task {
            self.draw_task_modal(frame, task);
        }
    }

    fn draw_dashboard(&self, frame: &mut Frame, area: Rect) {
        let width = area.width;
        let (menu_area, status_area) = if width < 60 {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(100)])
                .split(area);
            (chunks[0], None)
        } else if width < 90 {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);
            (chunks[0], Some(chunks[1]))
        } else {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(66), Constraint::Percentage(34)])
                .split(area);
            (chunks[0], Some(chunks[1]))
        };

        // Status panel
        let singbox_info = singbox::detect_singbox().unwrap_or(singbox::SingBoxInfo {
            installed: false,
            path: None,
            version: None,
        });

        let user_count = self.db.count_users().unwrap_or(0);
        let active_users = self.db.count_active_users().unwrap_or(0);

        let status_text = vec![
            Line::from(""),
            Line::from(vec![
                Span::raw("  sing-box: "),
                if singbox_info.installed {
                    Span::styled(
                        "Installed",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Span::styled(
                        "Not Found",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    )
                },
            ]),
            Line::from(format!(
                "  Version:  {}",
                singbox_info.version.unwrap_or_else(|| "-".to_string())
            )),
            Line::from(""),
            Line::from(vec![
                Span::raw("  Total Users:  "),
                Span::styled(
                    format!("{}", user_count),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::raw("  Active Users: "),
                Span::styled(
                    format!("{}", active_users),
                    Style::default().fg(Color::Green),
                ),
            ]),
        ];

        let status = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL).title(" Status "));
        if let Some(area) = status_area {
            frame.render_widget(status, area);
        }

        // Menu panel with selectable items
        let menu_items: Vec<ListItem> = MenuItem::all()
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == self.menu_index {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let prefix = if i == self.menu_index { " > " } else { "   " };
                ListItem::new(format!("{}{}", prefix, item.label())).style(style)
            })
            .collect();

        let menu = List::new(menu_items)
            .block(Block::default().borders(Borders::ALL).title(" Menu "))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        frame.render_widget(menu, menu_area);
    }

    fn draw_setup(&self, frame: &mut Frame, area: Rect) {
        let step_title = match self.setup_state.step {
            SetupStep::SingboxCheck => "sbconfig Setup (1/4)",
            SetupStep::PortConfig => "sbconfig Setup (2/4)",
            SetupStep::ModeSelect => "sbconfig Setup (3/4)",
            SetupStep::Confirm => "sbconfig Setup (4/4)",
        };

        let block = Block::default().borders(Borders::ALL).title(step_title);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let mut content = match self.setup_state.step {
            SetupStep::SingboxCheck => {
                let info = singbox::detect_singbox().unwrap_or(singbox::SingBoxInfo {
                    installed: false,
                    path: None,
                    version: None,
                });
                if info.installed {
                    vec![
                        Line::from(""),
                        Line::from(" sing-box is installed."),
                        Line::from(format!(
                            " Version: {}",
                            info.version.unwrap_or_else(|| "-".to_string())
                        )),
                        Line::from(format!(" Path:    {}", info.path.unwrap_or_default())),
                        Line::from(""),
                        Line::from(" Press [Enter] to continue."),
                    ]
                } else {
                    vec![
                        Line::from(""),
                        Line::from(" sing-box is NOT installed."),
                        Line::from(" Install sing-box before continuing."),
                        Line::from(""),
                        Line::from(" bash <(curl -fsSL https://sing-box.app/deb-install.sh)"),
                        Line::from(" https://sing-box.sagernet.org/installation/package-manager/"),
                        Line::from(""),
                        Line::from(" Press [r] to re-check."),
                    ]
                }
            }
            SetupStep::PortConfig => {
                let existing_mark = if self.setup_state.existing_port {
                    "(●)"
                } else {
                    "( )"
                };
                let new_mark = if self.setup_state.existing_port {
                    "( )"
                } else {
                    "(●)"
                };
                vec![
                    Line::from(""),
                    Line::from(" sbconfig requires a dedicated SSH port for proxy users."),
                    Line::from(" This should NOT be your main SSH port (usually 22)."),
                    Line::from(""),
                    Line::from(format!(
                        " {} Yes, I have an existing port: [{}]",
                        existing_mark, self.setup_state.port_input.value
                    )),
                    Line::from(format!(" {} No, generate a random port for me", new_mark)),
                    Line::from(""),
                    Line::from(" Make sure this port is open in your firewall."),
                ]
            }
            SetupStep::ModeSelect => {
                let prod_mark = if self.setup_state.mode == ServerMode::Production {
                    "(●)"
                } else {
                    "( )"
                };
                let dev_mark = if self.setup_state.mode == ServerMode::Development {
                    "(●)"
                } else {
                    "( )"
                };
                let mode_prefix = if self.setup_state.focus == SetupFocus::ModeChoice {
                    "> "
                } else {
                    "  "
                };
                let domain_prefix = if self.setup_state.focus == SetupFocus::DomainInput {
                    "> "
                } else {
                    "  "
                };
                let ip_prefix = if self.setup_state.focus == SetupFocus::IpInput {
                    "> "
                } else {
                    "  "
                };
                let server_hint = if self.setup_state.mode == ServerMode::Development {
                    " Server address will be set to localhost."
                } else {
                    " Enter a domain or IP address."
                };
                vec![
                    Line::from(""),
                    Line::from(" How will you be using sbconfig?"),
                    Line::from(format!(
                        "{}{} Production - Real VPS with domain/public IP",
                        mode_prefix, prod_mark
                    )),
                    Line::from(format!(
                        "  {} Development - Local testing with localhost",
                        dev_mark
                    )),
                    Line::from(""),
                    Line::from(format!(
                        "{}Domain: [{}]",
                        domain_prefix, self.setup_state.domain_input.value
                    )),
                    Line::from(format!(
                        "{}IP:     [{}]",
                        ip_prefix, self.setup_state.ip_input.value
                    )),
                    Line::from(" Press [a] to auto-detect public IP."),
                    Line::from(server_hint),
                ]
            }
            SetupStep::Confirm => {
                let mode = match self.setup_state.mode {
                    ServerMode::Production => "production",
                    ServerMode::Development => "development",
                };
                let server = if self.setup_state.mode == ServerMode::Development {
                    "localhost".to_string()
                } else if !self.setup_state.domain_input.value.trim().is_empty() {
                    self.setup_state.domain_input.value.clone()
                } else {
                    self.setup_state.ip_input.value.clone()
                };
                vec![
                    Line::from(""),
                    Line::from(" Configuration Summary"),
                    Line::from(format!(" SSH Port:  {}", self.setup_state.port_input.value)),
                    Line::from(format!(" Mode:      {}", mode)),
                    Line::from(format!(" Server:    {}", server)),
                    Line::from(""),
                    Line::from(" Press [Enter] to save settings and start sbconfig."),
                ]
            }
        };

        if let Some(message) = &self.status_message {
            content.push(Line::from(""));
            content.push(Line::from(Span::styled(
                message,
                Style::default().fg(Color::Red),
            )));
        }

        let paragraph = if matches!(self.settings_modal, SettingsModal::ViewSshConfig) {
            Paragraph::new(content)
                .block(Block::default())
                .scroll((self.settings_modal_scroll, 0))
        } else {
            Paragraph::new(content).block(Block::default())
        };
        frame.render_widget(paragraph, inner);
    }

    fn draw_singbox_status(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
            .split(area);
        let wrap_width = chunks[1].width.saturating_sub(4) as usize;

        let info = singbox::detect_singbox().unwrap_or(singbox::SingBoxInfo {
            installed: false,
            path: None,
            version: None,
        });
        let status = singbox::get_service_status().unwrap_or(ServiceStatus::Unknown);
        let status_text = match status {
            ServiceStatus::Running => "Running",
            ServiceStatus::Stopped => "Stopped",
            ServiceStatus::NotFound => "Not Found",
            ServiceStatus::Unknown => "Unknown",
        };

        let actions = self.singbox_actions(&info, status);
        let selected_index = if self.singbox_action_index < actions.len() {
            self.singbox_action_index
        } else {
            0
        };
        let actions: Vec<ListItem> = actions
            .iter()
            .enumerate()
            .map(|(index, (action, enabled))| {
                let prefix = if index == selected_index && *enabled {
                    "> "
                } else {
                    "  "
                };
                let style = if *enabled {
                    Style::default()
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                ListItem::new(Line::from(Span::styled(
                    format!("{}{}", prefix, action.label()),
                    style,
                )))
            })
            .collect();

        let action_block = Block::default().borders(Borders::ALL).title(" Actions ");
        let action_list = List::new(actions).block(action_block);
        frame.render_widget(action_list, chunks[0]);

        let heading_style = Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD);

        let mut status_lines = vec![
            Line::from(Span::styled(" Installation", heading_style)),
            Line::from(format!(
                "  Status:     {}",
                if info.installed {
                    "Installed"
                } else {
                    "Not Found"
                }
            )),
            Line::from(format!(
                "  Version:    {}",
                info.version.unwrap_or_else(|| "-".to_string())
            )),
            Line::from(format!(
                "  Path:       {}",
                info.path.unwrap_or_else(|| "-".to_string())
            )),
            Line::from(""),
            Line::from(Span::styled(" Service", heading_style)),
            Line::from(format!("  State:      {}", status_text)),
            Line::from(format!(
                "  Systemd:    {}",
                match status {
                    ServiceStatus::NotFound => "Not Found",
                    _ => "Available (OK)",
                }
            )),
            Line::from(""),
        ];

        if info.installed {
            if status == ServiceStatus::NotFound {
                status_lines.push(Line::from(Span::styled(" Hints", heading_style)));
                status_lines.push(Line::from("  Activate systemd service:"));
                status_lines.push(Line::from("  sudo systemctl enable --now sing-box"));
                status_lines.push(Line::from("  If the unit is missing, reinstall via:"));
                status_lines.push(Line::from(
                    "  bash <(curl -fsSL https://sing-box.app/deb-install.sh)",
                ));
                status_lines.push(Line::from(
                    "  https://sing-box.sagernet.org/installation/package-manager/",
                ));
                status_lines.push(Line::from(""));
                status_lines.push(Line::from("  Uninstall: use your package manager."));
            } else {
                status_lines.push(Line::from(Span::styled(" Hints", heading_style)));
                status_lines.push(Line::from("  Uninstall: use your package manager."));
            }
        } else {
            status_lines.push(Line::from(Span::styled(" Hints", heading_style)));
            status_lines.push(Line::from("  Install sing-box via official docs:"));
            status_lines.push(Line::from(
                "  bash <(curl -fsSL https://sing-box.app/deb-install.sh)",
            ));
            status_lines.push(Line::from(
                "  https://sing-box.sagernet.org/installation/package-manager/",
            ));
        }

        if let Some(progress) = &self.singbox_progress {
            status_lines.push(Line::from(""));
            status_lines.push(Line::from(Span::styled(" Progress", heading_style)));
            status_lines.push(Line::from(format!("  {}", progress)));
        }

        if let Some(output) = &self.singbox_status_output {
            status_lines.push(Line::from(""));
            status_lines.push(Line::from(Span::styled(" Service Output", heading_style)));
            let mut lines: Vec<&str> = output.lines().collect();
            if lines.len() > 6 {
                lines = lines[lines.len() - 6..].to_vec();
            }
            for line in lines {
                if wrap_width > 0 {
                    for wrapped in wrap_text(line, wrap_width.saturating_sub(2)) {
                        status_lines.push(Line::from(format!("  {}", wrapped)));
                    }
                } else {
                    status_lines.push(Line::from(format!("  {}", line)));
                }
            }
        }

        let mut right_lines = status_lines;
        if let Some(message) = &self.singbox_message {
            right_lines.push(Line::from(""));
            right_lines.push(Line::from(Span::styled(" Last action", heading_style)));
            let wrapped = wrap_text(message, wrap_width.max(10));
            if wrapped.is_empty() {
                right_lines.push(Line::from(Span::styled(
                    format!("  {}", message),
                    Style::default().fg(Color::Cyan),
                )));
            } else {
                for line in wrapped {
                    right_lines.push(Line::from(Span::styled(
                        format!("  {}", line),
                        Style::default().fg(Color::Cyan),
                    )));
                }
            }
        }

        let status_block = Block::default().borders(Borders::ALL).title(" Status ");
        let status_paragraph = Paragraph::new(right_lines).block(status_block);
        frame.render_widget(status_paragraph, chunks[1]);
    }

    fn draw_task_modal(&self, frame: &mut Frame, task: &SingboxTaskState) {
        let area = frame.area();
        let popup_area = centered_rect(80, 70, area);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", task.title));
        frame.render_widget(Clear, popup_area);
        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        let mut lines = Vec::new();
        let status_line = if task.running {
            "Status: Running"
        } else if task.error.is_some() {
            "Status: Failed"
        } else {
            "Status: Complete"
        };
        lines.push(Line::from(status_line));
        lines.push(Line::from(format!(
            "Progress: {}/{}",
            task.step, task.total_steps
        )));
        lines.push(Line::from(""));
        lines.push(Line::from("Logs:"));
        let max_logs = inner.height.saturating_sub(6) as usize;
        let start = task.logs.len().saturating_sub(max_logs);
        for line in task.logs.iter().skip(start) {
            lines.push(Line::from(line.clone()));
        }
        if let Some(result) = &task.result {
            lines.push(Line::from(""));
            lines.push(Line::from(format!("Result: {}", result)));
        }
        if let Some(err) = &task.error {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("Error: {}", err),
                Style::default().fg(Color::Red),
            )));
        }
        if !task.running {
            lines.push(Line::from(""));
            lines.push(Line::from("Press [Esc] to close."));
        }

        let paragraph = Paragraph::new(lines).block(Block::default());
        frame.render_widget(paragraph, inner);
    }

    fn draw_user_create(&self, frame: &mut Frame, area: Rect) {
        let ssh_port = self
            .db
            .get_setting("ssh_port")
            .ok()
            .flatten()
            .unwrap_or_else(|| "7344".to_string());

        let username_style = if self.user_create_state.focus == UserCreateFocus::Username {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };
        let key_style = if self.user_create_state.focus == UserCreateFocus::KeyType {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };

        let key_choice = match self.user_create_state.key_type {
            KeyType::Ed25519 => "(●) ED25519  ( ) RSA",
            KeyType::Rsa => "( ) ED25519  (●) RSA",
        };

        let mut lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(" Username: ", username_style),
                Span::raw(format!("[{}]", self.user_create_state.username_input.value)),
            ]),
            Line::from(format!(" SSH Port: [{}]", ssh_port)),
            Line::from(vec![
                Span::styled(" Key Type: ", key_style),
                Span::raw(key_choice),
            ]),
            Line::from(""),
        ];

        if let Some(error) = &self.user_create_state.error {
            lines.push(Line::from(Span::styled(
                error,
                Style::default().fg(Color::Red),
            )));
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Create User ");
        let paragraph = Paragraph::new(lines).block(block);
        frame.render_widget(paragraph, area);
    }

    fn draw_user_delete(&self, frame: &mut Frame, area: Rect) {
        let users = self.db.list_users().unwrap_or_default();
        let selected = users.get(self.user_list_index);
        let name = selected.map(|u| u.username.as_str()).unwrap_or("(none)");
        let mut lines = vec![
            Line::from(""),
            Line::from(format!(" Delete user '{}' ?", name)),
            Line::from(""),
            Line::from(" Press [y] to confirm or [n] to cancel."),
        ];
        if let Some(error) = &self.user_delete_state.error {
            lines.push(Line::from(Span::styled(
                error,
                Style::default().fg(Color::Red),
            )));
        }
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Delete User ");
        let paragraph = Paragraph::new(lines).block(block);
        frame.render_widget(paragraph, area);
    }

    fn draw_config_platform(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = ConfigPlatform::all()
            .iter()
            .enumerate()
            .map(|(index, platform)| {
                let prefix = if index == self.config_state.platform_index {
                    " > "
                } else {
                    "   "
                };
                ListItem::new(format!(
                    "{}{} - {}",
                    prefix,
                    platform.label(),
                    platform.description()
                ))
            })
            .collect();

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Select Platform ");
        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }

    fn draw_config_output(&self, frame: &mut Frame, area: Rect) {
        let output = self
            .config_state
            .output_json
            .clone()
            .unwrap_or_else(|| "No config generated yet.".to_string());
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Config Output ");
        let paragraph = Paragraph::new(output)
            .block(block)
            .scroll((self.config_state.scroll as u16, 0));
        frame.render_widget(paragraph, area);
    }

    fn draw_users(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(area);

        let body_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(chunks[0]);

        let users = self.db.list_users().unwrap_or_default();

        if users.is_empty() {
            let empty_text = vec![
                Line::from(""),
                Line::from("  No users configured yet."),
                Line::from(""),
                Line::from(Span::styled(
                    "  Press [a] to add a new user",
                    Style::default().fg(Color::Cyan),
                )),
            ];
            let empty = Paragraph::new(empty_text)
                .block(Block::default().borders(Borders::ALL).title(" Users "));
            frame.render_widget(empty, body_chunks[0]);
        } else {
            let items: Vec<ListItem> = users
                .iter()
                .enumerate()
                .map(|(i, u)| {
                    let status_style = if u.is_active {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Red)
                    };
                    let status_text = if u.is_active { "Active" } else { "Disabled" };

                    let is_selected = i == self.user_list_index;
                    let prefix = if is_selected { " > " } else { "   " };

                    let line = Line::from(vec![
                        Span::raw(prefix),
                        Span::styled(format!("[{}]", status_text), status_style),
                        Span::raw(format!(" {} ", u.username)),
                        Span::styled(
                            format!("({})", u.key_type),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]);

                    let style = if is_selected {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    ListItem::new(line).style(style)
                })
                .collect();

            let list = List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" Users ({}) ", users.len())),
            );

            frame.render_widget(list, body_chunks[0]);
        }

        let detail_block = Block::default()
            .borders(Borders::ALL)
            .title(" User Details ");
        let detail_inner = detail_block.inner(body_chunks[1]);
        frame.render_widget(detail_block, body_chunks[1]);

        if let Some(user) = users.get(self.user_list_index) {
            let detail_lines = vec![
                Line::from(format!(" Username: {}", user.username)),
                Line::from(format!(" Key Type: {}", user.key_type)),
                Line::from(format!(
                    " Active:   {}",
                    if user.is_active { "Yes" } else { "No" }
                )),
                Line::from(format!(" Created:  {}", user.created_at)),
            ];
            let paragraph = Paragraph::new(detail_lines);
            frame.render_widget(paragraph, detail_inner);
        }

        let hints = Paragraph::new(
            " [a] Add User  [d] Delete  [Enter] Toggle Active  [g] Generate Config ",
        )
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
        frame.render_widget(hints, chunks[1]);
    }

    fn draw_configs(&self, frame: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(""),
            Line::from("  Select a user from the Users screen to generate a configuration."),
            Line::from(""),
            Line::from(Span::styled(
                "  Supported platforms:",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from("    - iOS (Shadowrocket, Stash)"),
            Line::from("    - Android (sing-box, NekoBox)"),
            Line::from("    - Windows (sing-box GUI)"),
            Line::from("    - macOS (sing-box)"),
            Line::from("    - Linux (sing-box CLI)"),
        ];
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Config Generation ");
        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
    }

    fn draw_settings(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
            .split(area);

        let heading_style = Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD);

        let actions = [
            (SettingsFocus::ChangeServerAddress, "Change Server Address"),
            (SettingsFocus::UpdateSshPort, "Update SSH Port"),
            (SettingsFocus::ChangeMode, "Change Mode"),
            (SettingsFocus::RemoveSshPort, "Remove SSH Port"),
            (SettingsFocus::ViewSshConfig, "View SSH Config"),
        ];

        let selected_index = actions
            .iter()
            .position(|(focus, _)| *focus == self.settings_edit_state.focus)
            .unwrap_or(0);

        let action_items: Vec<ListItem> = actions
            .iter()
            .map(|(_, label)| ListItem::new(Line::from(format!(" {}", label))))
            .collect();

        let action_block = Block::default().borders(Borders::ALL).title(" Actions ");
        let action_list = List::new(action_items)
            .block(action_block)
            .highlight_symbol(" > ")
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );
        let mut action_state = ListState::default();
        action_state.select(Some(selected_index));
        frame.render_stateful_widget(action_list, chunks[0], &mut action_state);

        let server_addr = self
            .db
            .get_setting("server_address")
            .ok()
            .flatten()
            .unwrap_or_default();
        let server_addr = if server_addr.trim().is_empty() {
            "Not configured".to_string()
        } else {
            server_addr
        };
        let ssh_port = self
            .db
            .get_setting("ssh_port")
            .ok()
            .flatten()
            .unwrap_or_default();
        let ssh_port = if ssh_port.trim().is_empty() {
            "Not configured".to_string()
        } else {
            ssh_port
        };
        let mode = self
            .db
            .get_setting("mode")
            .ok()
            .flatten()
            .unwrap_or_else(|| "production".to_string());

        let ports_info = match ssh::list_ports(&ssh::sshd_config_path()) {
            Ok(ports) if !ports.is_empty() => ports
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            Ok(_) => "No ports found".to_string(),
            Err(err) => format!("Unavailable ({})", err),
        };

        let status_lines = vec![
            Line::from(Span::styled(" Current Settings", heading_style)),
            Line::from(""),
            Line::from(format!("  Server Address: {}", server_addr)),
            Line::from(format!("  SSH Port:       {}", ssh_port)),
            Line::from(format!("  Mode:           {}", mode)),
            Line::from(""),
            Line::from(Span::styled(" SSH Config", heading_style)),
            Line::from(""),
            Line::from(format!(
                "  sshd_config: {}",
                ssh::sshd_config_path().display()
            )),
            Line::from(format!("  Ports:       {}", ports_info)),
            Line::from(""),
            Line::from(Span::styled(" Tips", heading_style)),
            Line::from(""),
            Line::from("  Use arrows and Enter to edit."),
        ];

        let status_block = Block::default().borders(Borders::ALL).title(" Status ");
        let status_paragraph = Paragraph::new(status_lines).block(status_block);
        frame.render_widget(status_paragraph, chunks[1]);

        if self.settings_modal != SettingsModal::None {
            self.draw_settings_modal(frame);
        }
    }

    fn draw_settings_modal(&self, frame: &mut Frame) {
        let area = frame.area();
        let popup_area = centered_rect(70, 50, area);
        let (title, lines) = match self.settings_modal {
            SettingsModal::ServerAddress => {
                let mut spans = vec![Span::raw(" Value: ")];
                spans.extend(render_input_spans(
                    &self.settings_edit_state.server_input,
                    32,
                ));
                let lines = vec![
                    Line::from(" Enter server address (domain or IP)."),
                    Line::from(""),
                    Line::from(spans),
                    Line::from(""),
                    Line::from(" [Enter] OK  [Esc] Cancel"),
                ];
                (" Server Address ", lines)
            }
            SettingsModal::SshPort => {
                let mut spans = vec![Span::raw(" Value: ")];
                spans.extend(render_input_spans(
                    &self.settings_edit_state.ssh_port_input,
                    6,
                ));
                let lines = vec![
                    Line::from(" Enter custom SSH port for proxy users."),
                    Line::from(" Must be between 1024 and 60000 (not 22)."),
                    Line::from(" Changes sshd_config and restarts SSH."),
                    Line::from(""),
                    Line::from(spans),
                    Line::from(""),
                    Line::from(" [Enter] OK  [Esc] Cancel"),
                ];
                (" SSH Port ", lines)
            }
            SettingsModal::Mode => {
                let mode_line = match self.settings_edit_state.mode {
                    ServerMode::Production => "(●) Production  ( ) Development",
                    ServerMode::Development => "( ) Production  (●) Development",
                };
                let lines = vec![
                    Line::from(" Select server mode."),
                    Line::from(""),
                    Line::from(format!(" {}", mode_line)),
                    Line::from(""),
                    Line::from(" [Left/Right] Toggle  [Enter] OK  [Esc] Cancel"),
                ];
                (" Server Mode ", lines)
            }
            SettingsModal::RemoveSshPort => {
                let current = self
                    .db
                    .get_setting("ssh_port")
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| "Not configured".to_string());
                let lines = vec![
                    Line::from(" Remove custom SSH port from sshd_config."),
                    Line::from(""),
                    Line::from(format!(" Current: {}", current)),
                    Line::from(""),
                    Line::from(" [Enter] OK  [Esc] Cancel"),
                ];
                (" Remove SSH Port ", lines)
            }
            SettingsModal::ViewSshConfig => {
                let path = ssh::sshd_config_path();
                let contents = fs::read_to_string(&path)
                    .unwrap_or_else(|err| format!("Failed to read sshd_config: {}", err));
                let mut lines = Vec::new();
                lines.push(Line::from(" sshd_config"));
                lines.push(Line::from(""));
                lines.push(Line::from(format!(" Path: {}", path.display())));
                lines.push(Line::from(""));
                for line in contents.lines() {
                    lines.push(Line::from(format!(" {}", line)));
                }
                lines.push(Line::from(""));
                lines.push(Line::from(
                    " [Up/Down] Scroll  [PgUp/PgDn] Page  [Home] Top  [Esc] Close",
                ));
                (" SSH Config ", lines)
            }
            SettingsModal::None => ("", Vec::new()),
        };

        let block = Block::default().borders(Borders::ALL).title(title);
        frame.render_widget(Clear, popup_area);
        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        let mut content = lines;
        if let Some(error) = &self.settings_modal_error {
            content.push(Line::from(""));
            content.push(Line::from(Span::styled(
                format!(" Error: {}", error),
                Style::default().fg(Color::Red),
            )));
        }

        let scroll = if matches!(self.settings_modal, SettingsModal::ViewSshConfig) {
            let max_scroll = content.len().saturating_sub(inner.height as usize) as u16;
            self.settings_modal_scroll.min(max_scroll)
        } else {
            0
        };

        let paragraph = if matches!(self.settings_modal, SettingsModal::ViewSshConfig) {
            Paragraph::new(content)
                .block(Block::default())
                .scroll((scroll, 0))
        } else {
            Paragraph::new(content).block(Block::default())
        };
        frame.render_widget(paragraph, inner);
    }

    fn draw_logs(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(area);

        let sessions = self.db.list_sessions(20).unwrap_or_default();
        let session_items: Vec<ListItem> = sessions
            .iter()
            .enumerate()
            .map(|(index, session)| {
                let prefix = if index == self.logs_state.session_index {
                    " > "
                } else {
                    "   "
                };
                ListItem::new(format!("{}{}", prefix, session.started_at))
            })
            .collect();

        let session_block = Block::default().borders(Borders::ALL).title(" Sessions ");
        let session_list = List::new(session_items).block(session_block);
        frame.render_widget(session_list, chunks[0]);

        let log_block = Block::default().borders(Borders::ALL).title(" Logs ");
        let log_inner = log_block.inner(chunks[1]);
        frame.render_widget(log_block, chunks[1]);

        if let Some(session) = sessions.get(self.logs_state.session_index) {
            let logs = self
                .db
                .list_logs_for_session(&session.id, 200)
                .unwrap_or_default();
            let lines: Vec<Line> = logs
                .iter()
                .map(|log| {
                    Line::from(format!(
                        "[{}] {} - {}",
                        log.level, log.created_at, log.message
                    ))
                })
                .collect();
            let paragraph = Paragraph::new(lines).scroll((self.logs_state.scroll as u16, 0));
            frame.render_widget(paragraph, log_inner);
        } else {
            let empty = Paragraph::new("No log sessions available.");
            frame.render_widget(empty, log_inner);
        }
    }

    fn handle_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if matches!(key.code, KeyCode::Char('q') | KeyCode::Char('Q')) {
                        self.running = false;
                        return Ok(());
                    }
                    match self.screen {
                        Screen::Setup => self.handle_setup_input(key.code),
                        Screen::Dashboard => self.handle_dashboard_input(key.code),
                        Screen::SingboxStatus => self.handle_singbox_status_input(key.code),
                        Screen::Users => self.handle_users_input(key.code),
                        Screen::UserCreate => self.handle_user_create_input(key.code),
                        Screen::UserDelete => self.handle_user_delete_input(key.code),
                        Screen::Configs => self.handle_generic_input(key.code),
                        Screen::ConfigPlatform => self.handle_config_platform_input(key.code),
                        Screen::ConfigOutput => self.handle_config_output_input(key.code),
                        Screen::Settings => self.handle_settings_input(key.code),
                        Screen::Logs => self.handle_logs_input(key.code),
                    }
                }
            }
        }
        self.drain_singbox_task_events();
        Ok(())
    }

    fn handle_dashboard_input(&mut self, key: KeyCode) {
        let menu_len = MenuItem::all().len();

        match key {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.running = false;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.menu_index > 0 {
                    self.menu_index -= 1;
                } else {
                    self.menu_index = menu_len - 1; // Wrap to bottom
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.menu_index < menu_len - 1 {
                    self.menu_index += 1;
                } else {
                    self.menu_index = 0; // Wrap to top
                }
            }
            KeyCode::Enter => {
                let selected = MenuItem::all()[self.menu_index];
                if let Some(screen) = selected.screen() {
                    self.push_screen(screen);
                } else {
                    // Quit selected
                    self.running = false;
                }
            }
            _ => {}
        }
    }

    fn handle_setup_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Char('Q') => self.running = false,
            KeyCode::Esc => {
                self.status_message = None;
                self.setup_state.step = match self.setup_state.step {
                    SetupStep::SingboxCheck => SetupStep::SingboxCheck,
                    SetupStep::PortConfig => SetupStep::SingboxCheck,
                    SetupStep::ModeSelect => SetupStep::PortConfig,
                    SetupStep::Confirm => SetupStep::ModeSelect,
                };
            }
            _ => {}
        }

        match self.setup_state.step {
            SetupStep::SingboxCheck => match key {
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    self.status_message = None;
                }
                KeyCode::Enter => {
                    if let Ok(info) = singbox::detect_singbox() {
                        if info.installed {
                            self.setup_state.step = SetupStep::PortConfig;
                            self.status_message = None;
                        } else {
                            self.status_message =
                                Some("sing-box is not installed yet.".to_string());
                        }
                    }
                }
                _ => {}
            },
            SetupStep::PortConfig => match key {
                KeyCode::Up | KeyCode::Down => {
                    self.setup_state.existing_port = !self.setup_state.existing_port;
                }
                KeyCode::Backspace => {
                    if self.setup_state.existing_port {
                        self.setup_state.port_input.backspace();
                    }
                }
                KeyCode::Char(ch) if ch.is_ascii_digit() => {
                    if self.setup_state.existing_port {
                        self.setup_state.port_input.insert_char(ch);
                    }
                }
                KeyCode::Enter => {
                    if !self.setup_state.existing_port {
                        if let Some(port) = crate::utils::system::find_available_port(1024, 60000) {
                            self.setup_state.port_input.set(&port.to_string());
                        }
                    }
                    self.setup_state.focus = SetupFocus::ModeChoice;
                    self.autodetect_public_ip();
                    self.setup_state.step = SetupStep::ModeSelect;
                    self.status_message = None;
                }
                _ => {}
            },
            SetupStep::ModeSelect => match key {
                KeyCode::Up | KeyCode::Down => {
                    if self.setup_state.focus == SetupFocus::ModeChoice {
                        self.setup_state.mode = match self.setup_state.mode {
                            ServerMode::Production => ServerMode::Development,
                            ServerMode::Development => ServerMode::Production,
                        };
                    } else {
                        self.setup_state.focus = match self.setup_state.focus {
                            SetupFocus::DomainInput => SetupFocus::IpInput,
                            SetupFocus::IpInput => SetupFocus::DomainInput,
                            SetupFocus::ModeChoice | SetupFocus::PortChoice => {
                                SetupFocus::DomainInput
                            }
                        };
                    }
                }
                KeyCode::Left | KeyCode::Right => {
                    if self.setup_state.focus == SetupFocus::ModeChoice {
                        self.setup_state.mode = match self.setup_state.mode {
                            ServerMode::Production => ServerMode::Development,
                            ServerMode::Development => ServerMode::Production,
                        };
                    }
                }
                KeyCode::Char('a') | KeyCode::Char('A') => {
                    self.autodetect_public_ip();
                }
                KeyCode::Char(ch) => {
                    if self.setup_state.focus == SetupFocus::DomainInput
                        && (ch.is_ascii_graphic() || ch == '.' || ch == '-')
                    {
                        self.setup_state.domain_input.insert_char(ch);
                    } else if self.setup_state.focus == SetupFocus::IpInput
                        && (ch.is_ascii_digit() || ch == '.')
                    {
                        self.setup_state.ip_input.insert_char(ch);
                    }
                }
                KeyCode::Backspace => {
                    if self.setup_state.focus == SetupFocus::DomainInput {
                        self.setup_state.domain_input.backspace();
                    } else if self.setup_state.focus == SetupFocus::IpInput {
                        self.setup_state.ip_input.backspace();
                    }
                }
                KeyCode::Enter => {
                    self.setup_state.step = SetupStep::Confirm;
                }
                _ => {}
            },
            SetupStep::Confirm => {
                if key == KeyCode::Enter {
                    if let Err(err) = self.save_setup_settings() {
                        self.status_message = Some(err.to_string());
                    } else {
                        self.status_message = None;
                        self.screen = Screen::Dashboard;
                    }
                }
            }
        }
    }

    fn handle_singbox_status_input(&mut self, key: KeyCode) {
        if let Some(task) = &self.singbox_task {
            if task.running {
                return;
            }
            if matches!(key, KeyCode::Esc | KeyCode::Char('b') | KeyCode::Char('B')) {
                self.singbox_task = None;
                return;
            }
        }
        let info = singbox::detect_singbox().unwrap_or(singbox::SingBoxInfo {
            installed: false,
            path: None,
            version: None,
        });
        let status = singbox::get_service_status().unwrap_or(ServiceStatus::Unknown);
        let actions = self.singbox_actions(&info, status);
        let action_count = actions.len();
        if action_count == 0 {
            return;
        }
        if self.singbox_action_index >= action_count {
            self.singbox_action_index = 0;
        }
        if !actions[self.singbox_action_index].1 {
            if let Some(next) =
                self.next_enabled_action_index(&actions, self.singbox_action_index, 1)
            {
                self.singbox_action_index = next;
            }
        }
        match key {
            KeyCode::Esc | KeyCode::Char('b') | KeyCode::Char('B') => self.pop_screen(),
            KeyCode::Up => {
                if let Some(prev) =
                    self.next_enabled_action_index(&actions, self.singbox_action_index, -1)
                {
                    self.singbox_action_index = prev;
                }
            }
            KeyCode::Down => {
                if let Some(next) =
                    self.next_enabled_action_index(&actions, self.singbox_action_index, 1)
                {
                    self.singbox_action_index = next;
                }
            }
            KeyCode::Enter => {
                let (action, enabled) = actions[self.singbox_action_index];
                if enabled {
                    self.run_singbox_action(action);
                }
            }
            KeyCode::Char(ch) if ch.is_ascii_digit() => {
                let index = (ch as usize).saturating_sub('1' as usize);
                if index < actions.len() {
                    let (action, enabled) = actions[index];
                    if enabled {
                        self.run_singbox_action(action);
                    }
                }
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.run_singbox_action(SingboxAction::Refresh);
            }
            _ => {}
        }
    }

    fn handle_users_input(&mut self, key: KeyCode) {
        let user_count = self.db.list_users().map(|u| u.len()).unwrap_or(0);

        match key {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.running = false;
            }
            KeyCode::Esc | KeyCode::Char('b') | KeyCode::Char('B') => {
                self.pop_screen();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if user_count > 0 {
                    if self.user_list_index > 0 {
                        self.user_list_index -= 1;
                    } else {
                        self.user_list_index = user_count - 1;
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if user_count > 0 {
                    if self.user_list_index < user_count - 1 {
                        self.user_list_index += 1;
                    } else {
                        self.user_list_index = 0;
                    }
                }
            }
            KeyCode::Enter => {
                // Toggle active status
                if let Ok(users) = self.db.list_users() {
                    if let Some(user) = users.get(self.user_list_index) {
                        let _ = self.db.toggle_user_active(user.id);
                    }
                }
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                self.user_create_state = UserCreateState {
                    username_input: InputField::new("user_"),
                    key_type: KeyType::Ed25519,
                    focus: UserCreateFocus::Username,
                    error: None,
                };
                self.push_screen(Screen::UserCreate);
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                if user_count > 0 {
                    self.user_delete_state = UserDeleteState { error: None };
                    self.push_screen(Screen::UserDelete);
                }
            }
            KeyCode::Char('g') | KeyCode::Char('G') => {
                if let Ok(users) = self.db.list_users() {
                    if let Some(user) = users.get(self.user_list_index) {
                        self.config_state.user_id = Some(user.id);
                        self.config_state.output_json = None;
                        self.push_screen(Screen::ConfigPlatform);
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_user_create_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.pop_screen();
            }
            KeyCode::Up | KeyCode::Down => {
                self.user_create_state.focus = match self.user_create_state.focus {
                    UserCreateFocus::Username => UserCreateFocus::KeyType,
                    UserCreateFocus::KeyType => UserCreateFocus::Username,
                };
            }
            KeyCode::Left | KeyCode::Right => {
                if self.user_create_state.focus == UserCreateFocus::KeyType {
                    self.user_create_state.key_type = match self.user_create_state.key_type {
                        KeyType::Ed25519 => KeyType::Rsa,
                        KeyType::Rsa => KeyType::Ed25519,
                    };
                }
            }
            KeyCode::Char(ch) => {
                if self.user_create_state.focus == UserCreateFocus::Username
                    && (ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '-')
                {
                    self.user_create_state.username_input.insert_char(ch);
                }
            }
            KeyCode::Backspace => {
                if self.user_create_state.focus == UserCreateFocus::Username {
                    self.user_create_state.username_input.backspace();
                }
            }
            KeyCode::Enter => {
                if let Err(err) = self.create_user_from_form() {
                    self.user_create_state.error = Some(err.to_string());
                } else {
                    self.pop_screen();
                }
            }
            _ => {}
        }
    }

    fn handle_user_delete_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
                self.pop_screen();
            }
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Err(err) = self.delete_selected_user() {
                    self.user_delete_state.error = Some(err.to_string());
                } else {
                    self.pop_screen();
                }
            }
            _ => {}
        }
    }

    fn handle_config_platform_input(&mut self, key: KeyCode) {
        let max = ConfigPlatform::all().len();
        match key {
            KeyCode::Esc => self.pop_screen(),
            KeyCode::Up | KeyCode::Char('k') => {
                if self.config_state.platform_index == 0 {
                    self.config_state.platform_index = max - 1;
                } else {
                    self.config_state.platform_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.config_state.platform_index + 1 >= max {
                    self.config_state.platform_index = 0;
                } else {
                    self.config_state.platform_index += 1;
                }
            }
            KeyCode::Enter => {
                if let Err(err) = self.generate_config_for_selected_user() {
                    self.config_state.error = Some(err.to_string());
                } else {
                    self.push_screen(Screen::ConfigOutput);
                }
            }
            _ => {}
        }
    }

    fn handle_config_output_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc | KeyCode::Char('b') | KeyCode::Char('B') => self.pop_screen(),
            KeyCode::Down | KeyCode::Char('j') => {
                self.config_state.scroll = self.config_state.scroll.saturating_add(1)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.config_state.scroll = self.config_state.scroll.saturating_sub(1)
            }
            _ => {}
        }
    }

    fn handle_settings_input(&mut self, key: KeyCode) {
        if self.settings_modal != SettingsModal::None {
            match self.settings_modal {
                SettingsModal::ServerAddress => match key {
                    KeyCode::Esc => {
                        self.settings_modal = SettingsModal::None;
                        self.settings_modal_error = None;
                    }
                    KeyCode::Enter => {
                        let value = self.settings_edit_state.server_input.value.trim();
                        if value.is_empty() {
                            self.settings_modal_error =
                                Some("Server address cannot be empty".to_string());
                            return;
                        }
                        if let Err(err) = self.db.set_setting("server_address", value) {
                            self.settings_modal_error = Some(err.to_string());
                            return;
                        }
                        self.settings_modal = SettingsModal::None;
                        self.settings_modal_error = None;
                    }
                    KeyCode::Backspace => {
                        self.settings_edit_state.server_input.backspace();
                    }
                    KeyCode::Char(ch) => {
                        if ch.is_ascii_graphic() || ch == '.' || ch == '-' {
                            self.settings_edit_state.server_input.insert_char(ch);
                        }
                    }
                    _ => {}
                },
                SettingsModal::SshPort => match key {
                    KeyCode::Esc => {
                        self.settings_modal = SettingsModal::None;
                        self.settings_modal_error = None;
                    }
                    KeyCode::Enter => {
                        let port_text = self.settings_edit_state.ssh_port_input.value.trim();
                        let port = match port_text.parse::<u16>() {
                            Ok(port) => port,
                            Err(_) => {
                                self.settings_modal_error =
                                    Some("SSH port must be numeric".to_string());
                                return;
                            }
                        };

                        let path = ssh::sshd_config_path();
                        let existing_ports = match ssh::list_ports(&path) {
                            Ok(ports) => ports,
                            Err(err) => {
                                self.settings_modal_error = Some(err.to_string());
                                return;
                            }
                        };
                        let old_port = self
                            .db
                            .get_setting("ssh_port")
                            .ok()
                            .flatten()
                            .and_then(|value| value.parse::<u16>().ok());

                        if port == 22 {
                            self.settings_modal_error =
                                Some("Custom SSH port must not be 22".to_string());
                            return;
                        }
                        if !(1024..=60000).contains(&port) {
                            self.settings_modal_error =
                                Some("Custom SSH port must be between 1024 and 60000".to_string());
                            return;
                        }
                        if !existing_ports.contains(&port)
                            && !crate::utils::system::is_port_available(port)
                        {
                            self.settings_modal_error =
                                Some("Custom SSH port is already in use".to_string());
                            return;
                        }
                        if existing_ports.contains(&port) && old_port.is_some_and(|old| old != port)
                        {
                            self.settings_modal_error = Some(
                                "SSH port is already configured; choose a new port".to_string(),
                            );
                            return;
                        }

                        if let Some(old_port) = old_port {
                            if old_port != port && existing_ports.contains(&old_port) {
                                if let Err(err) = ssh::remove_port(&path, old_port) {
                                    self.settings_modal_error = Some(err.to_string());
                                    return;
                                }
                            }
                        }
                        if let Err(err) = ssh::add_port(&path, port) {
                            self.settings_modal_error = Some(err.to_string());
                            return;
                        }

                        if let Err(err) = ssh::restart_sshd() {
                            self.settings_modal_error = Some(err.to_string());
                            return;
                        }

                        if let Err(err) = self.db.set_setting("ssh_port", port_text) {
                            self.settings_modal_error = Some(err.to_string());
                            return;
                        }

                        self.settings_modal = SettingsModal::None;
                        self.settings_modal_error = None;
                    }
                    KeyCode::Backspace => {
                        self.settings_edit_state.ssh_port_input.backspace();
                    }
                    KeyCode::Char(ch) => {
                        if ch.is_ascii_digit() {
                            self.settings_edit_state.ssh_port_input.insert_char(ch);
                        }
                    }
                    _ => {}
                },
                SettingsModal::Mode => match key {
                    KeyCode::Left | KeyCode::Right => {
                        self.settings_edit_state.mode = match self.settings_edit_state.mode {
                            ServerMode::Production => ServerMode::Development,
                            ServerMode::Development => ServerMode::Production,
                        };
                    }
                    KeyCode::Enter => {
                        let mode = match self.settings_edit_state.mode {
                            ServerMode::Production => "production",
                            ServerMode::Development => "development",
                        };
                        if let Err(err) = self.db.set_setting("mode", mode) {
                            self.settings_modal_error = Some(err.to_string());
                            return;
                        }
                        self.settings_modal = SettingsModal::None;
                        self.settings_modal_error = None;
                    }
                    KeyCode::Esc => {
                        self.settings_modal = SettingsModal::None;
                        self.settings_modal_error = None;
                    }
                    _ => {}
                },
                SettingsModal::RemoveSshPort => match key {
                    KeyCode::Esc => {
                        self.settings_modal = SettingsModal::None;
                        self.settings_modal_error = None;
                        self.settings_modal_scroll = 0;
                    }
                    KeyCode::Enter => {
                        let port_text = self
                            .db
                            .get_setting("ssh_port")
                            .ok()
                            .flatten()
                            .unwrap_or_default();
                        let port = match port_text.parse::<u16>() {
                            Ok(port) => port,
                            Err(_) => {
                                self.settings_modal_error =
                                    Some("No valid SSH port configured".to_string());
                                return;
                            }
                        };

                        let path = ssh::sshd_config_path();
                        let changed = match ssh::remove_port(&path, port) {
                            Ok(changed) => changed,
                            Err(err) => {
                                self.settings_modal_error = Some(err.to_string());
                                return;
                            }
                        };
                        if changed {
                            if let Err(err) = ssh::restart_sshd() {
                                self.settings_modal_error = Some(err.to_string());
                                return;
                            }
                        }
                        if let Err(err) = self.db.set_setting("ssh_port", "") {
                            self.settings_modal_error = Some(err.to_string());
                            return;
                        }

                        if let Some(port) = crate::utils::system::find_available_port(1024, 60000) {
                            self.settings_edit_state
                                .ssh_port_input
                                .set(&port.to_string());
                        } else {
                            self.settings_edit_state.ssh_port_input.set("");
                        }
                        self.settings_modal = SettingsModal::SshPort;
                        self.settings_modal_error = Some(
                            "Custom SSH port required for configs. Add a port or press Esc to skip."
                                .to_string(),
                        );
                        self.settings_modal_scroll = 0;
                    }
                    _ => {}
                },
                SettingsModal::ViewSshConfig => match key {
                    KeyCode::Esc | KeyCode::Enter => {
                        self.settings_modal = SettingsModal::None;
                        self.settings_modal_error = None;
                        self.settings_modal_scroll = 0;
                    }
                    KeyCode::Up => {
                        self.settings_modal_scroll = self.settings_modal_scroll.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        self.settings_modal_scroll = self.settings_modal_scroll.saturating_add(1);
                    }
                    KeyCode::PageUp => {
                        self.settings_modal_scroll = self.settings_modal_scroll.saturating_sub(5);
                    }
                    KeyCode::PageDown => {
                        self.settings_modal_scroll = self.settings_modal_scroll.saturating_add(5);
                    }
                    KeyCode::Home => {
                        self.settings_modal_scroll = 0;
                    }
                    _ => {}
                },
                SettingsModal::None => {}
            }
            return;
        }

        match key {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.running = false;
            }
            KeyCode::Esc | KeyCode::Char('b') | KeyCode::Char('B') => self.pop_screen(),
            KeyCode::Up => {
                self.settings_edit_state.focus = match self.settings_edit_state.focus {
                    SettingsFocus::ChangeServerAddress => SettingsFocus::ViewSshConfig,
                    SettingsFocus::UpdateSshPort => SettingsFocus::ChangeServerAddress,
                    SettingsFocus::ChangeMode => SettingsFocus::UpdateSshPort,
                    SettingsFocus::RemoveSshPort => SettingsFocus::ChangeMode,
                    SettingsFocus::ViewSshConfig => SettingsFocus::RemoveSshPort,
                };
            }
            KeyCode::Down => {
                self.settings_edit_state.focus = match self.settings_edit_state.focus {
                    SettingsFocus::ChangeServerAddress => SettingsFocus::UpdateSshPort,
                    SettingsFocus::UpdateSshPort => SettingsFocus::ChangeMode,
                    SettingsFocus::ChangeMode => SettingsFocus::RemoveSshPort,
                    SettingsFocus::RemoveSshPort => SettingsFocus::ViewSshConfig,
                    SettingsFocus::ViewSshConfig => SettingsFocus::ChangeServerAddress,
                };
            }
            KeyCode::Enter => {
                self.load_settings_edit();
                self.settings_modal_error = None;
                self.settings_modal_scroll = 0;
                self.settings_modal = match self.settings_edit_state.focus {
                    SettingsFocus::ChangeServerAddress => SettingsModal::ServerAddress,
                    SettingsFocus::UpdateSshPort => SettingsModal::SshPort,
                    SettingsFocus::ChangeMode => SettingsModal::Mode,
                    SettingsFocus::RemoveSshPort => SettingsModal::RemoveSshPort,
                    SettingsFocus::ViewSshConfig => SettingsModal::ViewSshConfig,
                };
            }
            _ => {}
        }
    }

    fn handle_logs_input(&mut self, key: KeyCode) {
        let sessions = self.db.list_sessions(20).unwrap_or_default();
        match key {
            KeyCode::Esc | KeyCode::Char('b') | KeyCode::Char('B') => self.pop_screen(),
            KeyCode::Up | KeyCode::Char('k') => {
                if self.logs_state.session_index > 0 {
                    self.logs_state.session_index -= 1;
                    self.logs_state.scroll = 0;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.logs_state.session_index + 1 < sessions.len() {
                    self.logs_state.session_index += 1;
                    self.logs_state.scroll = 0;
                }
            }
            KeyCode::PageDown => {
                self.logs_state.scroll = self.logs_state.scroll.saturating_add(5);
            }
            KeyCode::PageUp => {
                self.logs_state.scroll = self.logs_state.scroll.saturating_sub(5);
            }
            _ => {}
        }
    }

    fn handle_generic_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.running = false;
            }
            KeyCode::Esc | KeyCode::Char('b') | KeyCode::Char('B') => {
                self.pop_screen();
            }
            _ => {}
        }
    }

    fn push_screen(&mut self, next: Screen) {
        if self.screen != next {
            self.screen_stack.push(self.screen);
            self.screen = next;
            if self.screen == Screen::SingboxStatus {
                self.singbox_message = None;
                self.refresh_singbox_status_output();
            }
        }
    }

    fn pop_screen(&mut self) {
        if let Some(prev) = self.screen_stack.pop() {
            self.screen = prev;
        } else {
            self.screen = Screen::Dashboard;
        }
    }

    fn load_settings_edit(&mut self) {
        let server = self
            .db
            .get_setting("server_address")
            .ok()
            .flatten()
            .unwrap_or_default();
        let ssh_port = self
            .db
            .get_setting("ssh_port")
            .ok()
            .flatten()
            .unwrap_or_else(|| "7344".to_string());
        let mode = self
            .db
            .get_setting("mode")
            .ok()
            .flatten()
            .unwrap_or_else(|| "production".to_string());

        self.settings_edit_state.server_input.set(&server);
        self.settings_edit_state.ssh_port_input.set(&ssh_port);
        self.settings_edit_state.mode = if mode == "development" {
            ServerMode::Development
        } else {
            ServerMode::Production
        };
        self.settings_edit_state.error = None;
    }

    fn save_setup_settings(&mut self) -> Result<()> {
        let port = self.setup_state.port_input.value.trim();
        let port_num = port
            .parse::<u16>()
            .map_err(|_| crate::error::AppError::Config("SSH port must be numeric".to_string()))?;
        if port_num == 22 {
            return Err(crate::error::AppError::Config(
                "Custom SSH port must not be 22".to_string(),
            ));
        }

        let mode = match self.setup_state.mode {
            ServerMode::Production => "production",
            ServerMode::Development => "development",
        };

        let server = if self.setup_state.mode == ServerMode::Development {
            "localhost".to_string()
        } else if !self.setup_state.domain_input.value.trim().is_empty() {
            self.setup_state.domain_input.value.trim().to_string()
        } else if !self.setup_state.ip_input.value.trim().is_empty() {
            self.setup_state.ip_input.value.trim().to_string()
        } else {
            return Err(crate::error::AppError::Config(
                "Server address is required".to_string(),
            ));
        };

        self.db.set_setting("ssh_port", port)?;
        self.db.set_setting("mode", mode)?;
        self.db.set_setting("server_address", &server)?;
        self.db.set_setting("setup_complete", "true")?;
        self.get_or_create_encryption_key()?;
        Ok(())
    }

    fn autodetect_public_ip(&mut self) {
        if !self.setup_state.ip_input.value.trim().is_empty() {
            return;
        }
        match crate::utils::system::get_public_ip() {
            Ok(ip) => {
                self.setup_state.ip_input.set(&ip);
                self.status_message = None;
            }
            Err(err) => {
                self.status_message = Some(err.to_string());
            }
        }
    }

    fn run_singbox_action(&mut self, action: SingboxAction) {
        self.singbox_progress = None;
        let result = match action {
            SingboxAction::Start => singbox::start_service().map(|_| "Service started"),
            SingboxAction::Stop => singbox::stop_service().map(|_| "Service stopped"),
            SingboxAction::Restart => singbox::restart_service().map(|_| "Service restarted"),
            SingboxAction::Enable => singbox::enable_service().map(|_| "Service enabled"),
            SingboxAction::Disable => singbox::disable_service().map(|_| "Service disabled"),
            SingboxAction::Refresh => Ok("Status refreshed"),
            SingboxAction::Install => {
                self.start_singbox_task("Install sing-box", singbox::install_steps);
                return;
            }
            SingboxAction::Reinstall => {
                self.start_singbox_task("Reinstall sing-box", singbox::reinstall_steps);
                return;
            }
            SingboxAction::Uninstall => {
                self.start_singbox_task("Uninstall sing-box", singbox::uninstall_steps);
                return;
            }
        };

        match result {
            Ok(message) => self.singbox_message = Some(message.to_string()),
            Err(err) => self.singbox_message = Some(err.to_string()),
        }

        self.refresh_singbox_status_output();
    }

    fn refresh_singbox_status_output(&mut self) {
        match singbox::get_service_status_output(8) {
            Ok(output) => {
                if output.is_empty() {
                    self.singbox_status_output = None;
                } else {
                    self.singbox_status_output = Some(output);
                }
            }
            Err(err) => {
                self.singbox_status_output = Some(err.to_string());
            }
        }
    }

    fn start_singbox_task<F>(&mut self, title: &str, steps_fn: F)
    where
        F: FnOnce() -> Result<Vec<singbox::CommandSpec>> + Send + 'static,
    {
        if self.singbox_task.as_ref().is_some_and(|t| t.running) {
            return;
        }

        let steps = match steps_fn() {
            Ok(steps) => steps,
            Err(err) => {
                self.singbox_task = Some(SingboxTaskState {
                    title: title.to_string(),
                    running: false,
                    logs: Vec::new(),
                    result: None,
                    error: Some(err.to_string()),
                    step: 0,
                    total_steps: 0,
                });
                return;
            }
        };

        let total_steps = steps.len();
        let (tx, rx) = mpsc::channel::<SingboxTaskEvent>();
        self.singbox_task = Some(SingboxTaskState {
            title: title.to_string(),
            running: true,
            logs: Vec::new(),
            result: None,
            error: None,
            step: 0,
            total_steps,
        });
        self.singbox_task_rx = Some(rx);

        thread::spawn(move || {
            for (index, step) in steps.into_iter().enumerate() {
                let _ = tx.send(SingboxTaskEvent::Progress {
                    step: index + 1,
                    total: total_steps,
                });
                let _ = tx.send(SingboxTaskEvent::Log(format!(
                    "[{}/{}] $ {}",
                    index + 1,
                    total_steps,
                    step.display
                )));
                match singbox::run_command_spec(&step) {
                    Ok(output) => {
                        if !output.is_empty() {
                            for line in output.lines() {
                                let _ = tx.send(SingboxTaskEvent::Log(line.to_string()));
                            }
                        }
                    }
                    Err(err) => {
                        let _ = tx.send(SingboxTaskEvent::Error(err.to_string()));
                        return;
                    }
                }
            }
            let _ = tx.send(SingboxTaskEvent::Done("Completed".to_string()));
        });
    }

    fn drain_singbox_task_events(&mut self) {
        let rx = match &self.singbox_task_rx {
            Some(rx) => rx,
            None => return,
        };
        let mut done = false;
        while let Ok(event) = rx.try_recv() {
            if let Some(task) = &mut self.singbox_task {
                match event {
                    SingboxTaskEvent::Log(line) => {
                        let cleaned = line.replace('\r', "");
                        task.logs.push(cleaned);
                        if task.logs.len() > 200 {
                            let excess = task.logs.len() - 200;
                            task.logs.drain(0..excess);
                        }
                    }
                    SingboxTaskEvent::Progress { step, total } => {
                        task.step = step;
                        task.total_steps = total;
                    }
                    SingboxTaskEvent::Done(message) => {
                        task.running = false;
                        task.result = Some(message);
                        done = true;
                    }
                    SingboxTaskEvent::Error(err) => {
                        task.running = false;
                        task.error = Some(err);
                        done = true;
                    }
                }
            }
        }

        if done {
            self.singbox_task_rx = None;
            self.refresh_singbox_status_output();
        }
    }

    fn singbox_actions(
        &self,
        info: &singbox::SingBoxInfo,
        status: ServiceStatus,
    ) -> Vec<(SingboxAction, bool)> {
        let installed = info.installed;
        let service_available = status != ServiceStatus::NotFound;

        vec![
            (SingboxAction::Start, installed && service_available),
            (SingboxAction::Stop, installed && service_available),
            (SingboxAction::Restart, installed && service_available),
            (SingboxAction::Enable, installed && service_available),
            (SingboxAction::Disable, installed && service_available),
            (SingboxAction::Refresh, true),
            (SingboxAction::Install, !installed),
            (SingboxAction::Reinstall, installed),
            (SingboxAction::Uninstall, installed),
        ]
    }

    fn next_enabled_action_index(
        &self,
        actions: &[(SingboxAction, bool)],
        current: usize,
        step: isize,
    ) -> Option<usize> {
        if actions.is_empty() {
            return None;
        }
        let len = actions.len() as isize;
        let mut offset = 0;
        while offset < len {
            let idx = ((current as isize + step * (offset + 1)).rem_euclid(len)) as usize;
            if actions[idx].1 {
                return Some(idx);
            }
            offset += 1;
        }
        None
    }

    fn get_or_create_encryption_key(&self) -> Result<String> {
        if let Some(existing) = self.db.get_setting("encryption_key")? {
            return Ok(existing);
        }

        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        let encoded = general_purpose::STANDARD.encode(bytes);
        self.db.set_setting("encryption_key", &encoded)?;
        Ok(encoded)
    }

    fn create_user_from_form(&mut self) -> Result<()> {
        let username = self.user_create_state.username_input.value.trim();
        if username.is_empty() {
            return Err(crate::error::AppError::User(
                "Username is required".to_string(),
            ));
        }

        ssh::validate_username(username)?;
        let keypair = ssh::generate_keypair(self.user_create_state.key_type, username)?;

        let encryption_key = self.get_or_create_encryption_key()?;
        let encrypted = crate::db::encrypt(&keypair.private_key, &encryption_key)?;
        self.db.create_user(
            username,
            &keypair.public_key,
            &encrypted,
            keypair.key_type.as_str(),
        )?;

        ssh::create_system_user(username)?;
        ssh::setup_authorized_keys(username, &keypair.public_key)?;

        Ok(())
    }

    fn delete_selected_user(&mut self) -> Result<()> {
        let users = self.db.list_users()?;
        if let Some(user) = users.get(self.user_list_index) {
            let _ = ssh::delete_system_user(&user.username);
            self.db.delete_user(user.id)?;
        }
        Ok(())
    }

    fn generate_config_for_selected_user(&mut self) -> Result<()> {
        let user_id = self
            .config_state
            .user_id
            .ok_or_else(|| crate::error::AppError::User("No user selected".to_string()))?;
        let user = self
            .db
            .get_user(user_id)?
            .ok_or_else(|| crate::error::AppError::User("User not found".to_string()))?;

        let encryption_key = self.get_or_create_encryption_key()?;
        let private_key = decrypt(&user.private_key_encrypted, &encryption_key)?;
        let server = self
            .db
            .get_setting("server_address")?
            .unwrap_or_else(|| "localhost".to_string());
        let port = self
            .db
            .get_setting("ssh_port")?
            .unwrap_or_else(|| "7344".to_string())
            .parse::<u16>()
            .map_err(|_| crate::error::AppError::Config("Invalid SSH port".to_string()))?;

        let json = generate_config_json(
            &server,
            port,
            &user.username,
            &private_key,
            RoutingPreset::Default,
        )?;
        self.config_state.output_json = Some(json);
        self.config_state.scroll = 0;
        Ok(())
    }
}
