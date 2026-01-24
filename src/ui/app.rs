// src/ui/app.rs
// Main application state and event loop

use crate::db::Database;
use crate::error::Result;
use crate::singbox;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::time::Duration;

/// Menu items on the dashboard
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItem {
    Users,
    Configs,
    Settings,
    Logs,
    Quit,
}

impl MenuItem {
    fn all() -> &'static [MenuItem] {
        &[
            MenuItem::Users,
            MenuItem::Configs,
            MenuItem::Settings,
            MenuItem::Logs,
            MenuItem::Quit,
        ]
    }

    fn label(&self) -> &'static str {
        match self {
            MenuItem::Users => "Manage Users",
            MenuItem::Configs => "Generate Configs",
            MenuItem::Settings => "Settings",
            MenuItem::Logs => "View Logs",
            MenuItem::Quit => "Quit",
        }
    }

    fn to_screen(&self) -> Option<Screen> {
        match self {
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
    Dashboard,
    Users,
    Configs,
    Settings,
    Logs,
}

pub struct App {
    pub db: Database,
    pub running: bool,
    pub screen: Screen,
    // Dashboard state
    pub menu_index: usize,
    // Users screen state
    pub user_list_index: usize,
}

impl App {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            running: true,
            screen: Screen::Dashboard,
            menu_index: 0,
            user_list_index: 0,
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
            Screen::Dashboard => " sbconfig - sing-box SSH Proxy Manager ",
            Screen::Users => " sbconfig - User Management ",
            Screen::Configs => " sbconfig - Config Generation ",
            Screen::Settings => " sbconfig - Settings ",
            Screen::Logs => " sbconfig - Logs ",
        };
        let header = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(title);
        frame.render_widget(header, chunks[0]);

        // Content based on current screen
        match self.screen {
            Screen::Dashboard => self.draw_dashboard(frame, chunks[1]),
            Screen::Users => self.draw_users(frame, chunks[1]),
            Screen::Configs => self.draw_configs(frame, chunks[1]),
            Screen::Settings => self.draw_settings(frame, chunks[1]),
            Screen::Logs => self.draw_logs(frame, chunks[1]),
        }

        // Footer with navigation hints
        let footer_text = match self.screen {
            Screen::Dashboard => " [Up/Down] Navigate  [Enter] Select  [q] Quit ",
            Screen::Users => {
                " [Up/Down] Navigate  [a] Add  [d] Delete  [Enter] Toggle  [Esc] Back "
            }
            _ => " [Esc] Back  [q] Quit ",
        };
        let footer = Paragraph::new(footer_text)
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(footer, chunks[2]);
    }

    fn draw_dashboard(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

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
        frame.render_widget(status, chunks[0]);

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

        frame.render_widget(menu, chunks[1]);
    }

    fn draw_users(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(area);

        let users = self.db.list_users().unwrap_or_default();

        if users.is_empty() {
            // Empty state
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
            frame.render_widget(empty, chunks[0]);
        } else {
            // User list
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
                        Style::default().bg(Color::DarkGray)
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

            frame.render_widget(list, chunks[0]);
        }

        // Action hints
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
        let server_addr = self
            .db
            .get_setting("server_address")
            .ok()
            .flatten()
            .unwrap_or_else(|| "Not configured".to_string());
        let ssh_port = self
            .db
            .get_setting("ssh_port")
            .ok()
            .flatten()
            .unwrap_or_else(|| "Not configured".to_string());
        let mode = self
            .db
            .get_setting("mode")
            .ok()
            .flatten()
            .unwrap_or_else(|| "production".to_string());

        let text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "  Server Address: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(&server_addr),
            ]),
            Line::from(vec![
                Span::styled(
                    "  SSH Port:       ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(&ssh_port),
            ]),
            Line::from(vec![
                Span::styled(
                    "  Mode:           ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(&mode),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "  Press [e] to edit settings",
                Style::default().fg(Color::Cyan),
            )),
        ];
        let block = Block::default().borders(Borders::ALL).title(" Settings ");
        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
    }

    fn draw_logs(&self, frame: &mut Frame, area: Rect) {
        let text = vec![
            Line::from(""),
            Line::from("  Log viewer coming soon..."),
            Line::from(""),
            Line::from(Span::styled(
                "  Logs are stored in /var/log/sbconfig/",
                Style::default().fg(Color::DarkGray),
            )),
        ];
        let block = Block::default().borders(Borders::ALL).title(" Logs ");
        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
    }

    fn handle_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match self.screen {
                        Screen::Dashboard => self.handle_dashboard_input(key.code),
                        Screen::Users => self.handle_users_input(key.code),
                        _ => self.handle_generic_input(key.code),
                    }
                }
            }
        }
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
                if let Some(screen) = selected.to_screen() {
                    self.screen = screen;
                } else {
                    // Quit selected
                    self.running = false;
                }
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
            KeyCode::Esc => {
                self.screen = Screen::Dashboard;
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
                // TODO: Open add user dialog
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                // TODO: Delete selected user (with confirmation)
            }
            KeyCode::Char('g') | KeyCode::Char('G') => {
                // TODO: Generate config for selected user
            }
            _ => {}
        }
    }

    fn handle_generic_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.running = false;
            }
            KeyCode::Esc => {
                self.screen = Screen::Dashboard;
            }
            _ => {}
        }
    }
}
