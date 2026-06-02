use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::{
    action::{Action, ConfigChange},
    app::{AppSnapshot, Screen},
    component::Component,
    pages::InstallerPage,
    terminal::Frame,
};

#[derive(Default)]
pub struct HostSelectPage {
    selected: usize,
    status_message: Option<String>,
}

#[derive(Default)]
pub struct HostnameInputPage {
    hostname: String,
    status_message: Option<String>,
}

pub fn host_page() -> Box<dyn InstallerPage> {
    Box::new(HostSelectPage::default())
}

pub fn hostname_page() -> Box<dyn InstallerPage> {
    Box::new(HostnameInputPage::default())
}

impl InstallerPage for HostSelectPage {
    fn screen(&self) -> Screen {
        Screen::HostSelect
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.status_message = snapshot.status_message.clone();
        self.selected = match snapshot.config.hostname.as_str() {
            "laptop" => 0,
            "virtual-machine" => 1,
            _ => 2,
        };
    }
}

impl Component for HostSelectPage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
                Action::Noop
            }
            KeyCode::Down | KeyCode::Tab => {
                self.selected = (self.selected + 1).min(2);
                Action::Noop
            }
            KeyCode::Left => Action::Navigate(Screen::PartitionConfirm),
            KeyCode::Right | KeyCode::Enter => match self.selected {
                0 => Action::Batch(vec![
                    Action::ConfigChanged(ConfigChange::Hostname("laptop".to_string())),
                    Action::Navigate(Screen::HardwareDetect),
                ]),
                1 => Action::Batch(vec![
                    Action::ConfigChanged(ConfigChange::Hostname(
                        "virtual-machine".to_string(),
                    )),
                    Action::Navigate(Screen::HardwareDetect),
                ]),
                _ => Action::Navigate(Screen::HostnameInput),
            },
            _ => Action::Noop,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Host Select ")
            .border_style(Style::default().fg(Color::Blue));
        let inner = block.inner(rect);
        f.render_widget(block, rect);

        let options = ["laptop", "virtual-machine", "custom"];
        let mut lines = vec![
            Line::from("Pick a host preset or continue with a custom hostname."),
            Line::default(),
        ];
        for (index, option) in options.iter().enumerate() {
            lines.push(Line::from(vec![
                Span::styled(
                    if index == self.selected { "> " } else { "  " },
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    option.to_string(),
                    if index == self.selected {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    },
                ),
            ]));
        }
        if let Some(message) = self.status_message.as_ref() {
            lines.push(Line::default());
            lines.push(Line::from(vec![
                Span::styled("status: ", Style::default().fg(Color::Yellow)),
                Span::raw(message.clone()),
            ]));
        }

        f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
    }
}

impl InstallerPage for HostnameInputPage {
    fn screen(&self) -> Screen {
        Screen::HostnameInput
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.hostname = snapshot.config.hostname.clone();
        self.status_message = snapshot.status_message.clone();
    }
}

impl Component for HostnameInputPage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Backspace => {
                self.hostname.pop();
                Action::ConfigChanged(ConfigChange::Hostname(self.hostname.clone()))
            }
            KeyCode::Char(c) => {
                self.hostname.push(c);
                Action::ConfigChanged(ConfigChange::Hostname(self.hostname.clone()))
            }
            KeyCode::Left => Action::Navigate(Screen::HostSelect),
            KeyCode::Right | KeyCode::Enter => {
                if self.hostname.trim().is_empty() {
                    Action::SetStatus(Some("Hostname is required".to_string()))
                } else {
                    Action::Batch(vec![
                        Action::ConfigChanged(ConfigChange::Hostname(self.hostname.clone())),
                        Action::Navigate(Screen::HardwareDetect),
                    ])
                }
            }
            _ => Action::Noop,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Hostname ")
            .border_style(Style::default().fg(Color::Blue));
        let inner = block.inner(rect);
        f.render_widget(block, rect);

        let lines = vec![
            Line::from("Type the hostname directly. This page always accepts text input."),
            Line::default(),
            Line::from(vec![
                Span::styled("hostname", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(": "),
                Span::raw(self.hostname.clone()),
            ]),
            Line::default(),
            Line::from("Right continues to hardware detection."),
            match self.status_message.as_ref() {
                Some(message) => Line::from(vec![
                    Span::styled("status: ", Style::default().fg(Color::Yellow)),
                    Span::raw(message.clone()),
                ]),
                None => Line::default(),
            },
        ];

        f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
    }
}