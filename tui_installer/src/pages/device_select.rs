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
    config::DeviceOption,
    pages::InstallerPage,
    terminal::Frame,
};

#[derive(Default)]
pub struct DeviceSelectPage {
    devices: Vec<DeviceOption>,
    selected: usize,
    current_device: String,
    status_message: Option<String>,
}

pub fn page() -> Box<dyn InstallerPage> {
    Box::new(DeviceSelectPage::default())
}

impl InstallerPage for DeviceSelectPage {
    fn screen(&self) -> Screen {
        Screen::DeviceSelect
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.devices = snapshot.devices.clone();
        self.current_device = snapshot.config.device.clone();
        self.status_message = snapshot.status_message.clone();
        if let Some(index) = self
            .devices
            .iter()
            .position(|device| device.path == self.current_device)
        {
            self.selected = index;
        }
    }
}

impl Component for DeviceSelectPage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Up => self.move_selection(-1),
            KeyCode::Down | KeyCode::Tab => self.move_selection(1),
            KeyCode::Left => Action::Navigate(Screen::GitHubLogin),
            KeyCode::Right | KeyCode::Enter => {
                if let Some(device) = self.devices.get(self.selected) {
                    Action::Batch(vec![
                        Action::ConfigChanged(ConfigChange::Device(device.path.clone())),
                        Action::Navigate(Screen::PartitionConfig),
                    ])
                } else {
                    Action::SetStatus(Some("No block devices available".to_string()))
                }
            }
            _ => Action::Noop,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Device Select ")
            .border_style(Style::default().fg(Color::Blue));
        let inner = block.inner(rect);
        f.render_widget(block, rect);

        let mut lines = vec![
            Line::from("Select the disk that will receive the installation."),
            Line::default(),
        ];

        if self.devices.is_empty() {
            lines.push(Line::from("No device entries are available in this prototype."));
        } else {
            for (index, device) in self.devices.iter().enumerate() {
                lines.push(Line::from(vec![
                    Span::styled(
                        if index == self.selected { "> " } else { "  " },
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::styled(
                        device.label(),
                        if index == self.selected {
                            Style::default().add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                        },
                    ),
                ]));
            }
        }

        lines.push(Line::default());
        lines.push(Line::from("Up/Down changes the selection. Right confirms it."));
        if let Some(message) = self.status_message.as_ref() {
            lines.push(Line::from(vec![
                Span::styled("status: ", Style::default().fg(Color::Yellow)),
                Span::raw(message.clone()),
            ]));
        }

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
        f.render_widget(paragraph, inner);
    }
}

impl DeviceSelectPage {
    fn move_selection(&mut self, delta: isize) -> Action {
        if self.devices.is_empty() {
            return Action::Noop;
        }
        let len = self.devices.len() as isize;
        self.selected = (self.selected as isize + delta).rem_euclid(len) as usize;
        Action::ConfigChanged(ConfigChange::Device(
            self.devices[self.selected].path.clone(),
        ))
    }
}