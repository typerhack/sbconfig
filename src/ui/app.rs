// src/ui/app.rs
// Main application state and event loop

use crate::db::Database;
use crate::error::Result;
use crate::singbox;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::time::Duration;

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
    pub status_message: Option<String>,
}

impl App {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            running: true,
            screen: Screen::Dashboard,
            status_message: None,
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
        let header = Block::default()
            .borders(Borders::ALL)
            .title(" sbconfig - sing-box SSH Proxy Manager ");
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
            Screen::Dashboard => " [1] Users  [2] Configs  [3] Settings  [4] Logs  [q] Quit ",
            _ => " [Esc] Back  [q] Quit ",
        };
        let footer = Paragraph::new(footer_text).block(Block::default().borders(Borders::ALL));
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
                    Span::styled("Installed", Style::default().fg(Color::Green))
                } else {
                    Span::styled("Not Found", Style::default().fg(Color::Red))
                },
            ]),
            Line::from(format!(
                "  Version:  {}",
                singbox_info.version.unwrap_or_else(|| "-".to_string())
            )),
            Line::from(""),
            Line::from(format!("  Total Users:  {}", user_count)),
            Line::from(format!("  Active Users: {}", active_users)),
        ];

        let status = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL).title(" Status "));
        frame.render_widget(status, chunks[0]);

        // Menu panel
        let menu_items = vec![
            Line::from(""),
            Line::from("  [1] Manage Users"),
            Line::from("  [2] Generate Configs"),
            Line::from("  [3] Settings"),
            Line::from("  [4] View Logs"),
            Line::from(""),
            Line::from("  [q] Quit"),
        ];

        let menu = Paragraph::new(menu_items)
            .block(Block::default().borders(Borders::ALL).title(" Menu "));
        frame.render_widget(menu, chunks[1]);
    }

    fn draw_users(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" User Management ");

        let users = self.db.list_users().unwrap_or_default();
        let items: Vec<ListItem> = users
            .iter()
            .map(|u| {
                let status = if u.is_active {
                    "[Active]"
                } else {
                    "[Disabled]"
                };
                ListItem::new(format!("  {} {} - {}", status, u.username, u.key_type))
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        frame.render_widget(list, area);
    }

    fn draw_configs(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Config Generation ");
        let text = Paragraph::new("Select a user to generate configuration...").block(block);
        frame.render_widget(text, area);
    }

    fn draw_settings(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default().borders(Borders::ALL).title(" Settings ");
        let text = Paragraph::new("Settings page...").block(block);
        frame.render_widget(text, area);
    }

    fn draw_logs(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default().borders(Borders::ALL).title(" Logs ");
        let text = Paragraph::new("Log viewer...").block(block);
        frame.render_widget(text, area);
    }

    fn handle_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => self.running = false,
                        KeyCode::Esc => {
                            if self.screen != Screen::Dashboard {
                                self.screen = Screen::Dashboard;
                            }
                        }
                        KeyCode::Char('1') if self.screen == Screen::Dashboard => {
                            self.screen = Screen::Users;
                        }
                        KeyCode::Char('2') if self.screen == Screen::Dashboard => {
                            self.screen = Screen::Configs;
                        }
                        KeyCode::Char('3') if self.screen == Screen::Dashboard => {
                            self.screen = Screen::Settings;
                        }
                        KeyCode::Char('4') if self.screen == Screen::Dashboard => {
                            self.screen = Screen::Logs;
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }
}
