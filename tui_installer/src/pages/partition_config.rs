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
pub struct PartitionConfigPage {
    active_field: usize,
    editing: bool,
    boot_size: String,
    swap_size: String,
    status_message: Option<String>,
}

#[derive(Default)]
pub struct PartitionConfirmPage {
    boot_size: String,
    swap_size: String,
    status_message: Option<String>,
}

pub fn config_page() -> Box<dyn InstallerPage> {
    Box::new(PartitionConfigPage::default())
}

pub fn confirm_page() -> Box<dyn InstallerPage> {
    Box::new(PartitionConfirmPage::default())
}

impl InstallerPage for PartitionConfigPage {
    fn screen(&self) -> Screen {
        Screen::PartitionConfig
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.boot_size = snapshot.config.boot_size.clone();
        self.swap_size = snapshot.config.swap_size.clone();
        self.status_message = snapshot.status_message.clone();
    }
}

impl Component for PartitionConfigPage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        if self.editing {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.editing = false;
                    Action::Noop
                }
                KeyCode::Backspace => {
                    self.current_field_mut().pop();
                    Action::ConfigChanged(self.current_change())
                }
                KeyCode::Char(c) => {
                    self.current_field_mut().push(c);
                    Action::ConfigChanged(self.current_change())
                }
                _ => Action::Noop,
            }
        } else {
            match key.code {
                KeyCode::Char('q') => Action::Quit,
                KeyCode::Up => {
                    self.active_field = self.active_field.saturating_sub(1);
                    Action::Noop
                }
                KeyCode::Down | KeyCode::Tab => {
                    self.active_field = (self.active_field + 1).min(1);
                    Action::Noop
                }
                KeyCode::Enter => {
                    self.editing = true;
                    Action::Noop
                }
                KeyCode::Left => Action::Navigate(Screen::DeviceSelect),
                KeyCode::Right => {
                    if self.boot_size.trim().is_empty() {
                        Action::SetStatus(Some("Boot partition size is required".to_string()))
                    } else {
                        Action::Navigate(Screen::PartitionConfirm)
                    }
                }
                _ => Action::Noop,
            }
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Partition Config ")
            .border_style(Style::default().fg(Color::Blue));
        let inner = block.inner(rect);
        f.render_widget(block, rect);

        let lines = vec![
            Line::from("Choose the EFI and swap partition sizes."),
            Line::default(),
            field_line(
                self.active_field == 0,
                self.editing && self.active_field == 0,
                "Boot size",
                &self.boot_size,
            ),
            field_line(
                self.active_field == 1,
                self.editing && self.active_field == 1,
                "Swap size",
                &self.swap_size,
            ),
            Line::default(),
            Line::from("Use values like 512MiB and 0 for disabled swap."),
            status_line(self.status_message.as_deref()),
        ];

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
        f.render_widget(paragraph, inner);
    }
}

impl PartitionConfigPage {
    fn current_field_mut(&mut self) -> &mut String {
        match self.active_field {
            0 => &mut self.boot_size,
            _ => &mut self.swap_size,
        }
    }

    fn current_change(&self) -> ConfigChange {
        match self.active_field {
            0 => ConfigChange::BootSize(self.boot_size.clone()),
            _ => ConfigChange::SwapSize(self.swap_size.clone()),
        }
    }
}

impl InstallerPage for PartitionConfirmPage {
    fn screen(&self) -> Screen {
        Screen::PartitionConfirm
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.boot_size = snapshot.config.boot_size.clone();
        self.swap_size = snapshot.config.swap_size.clone();
        self.status_message = snapshot.status_message.clone();
    }
}

impl Component for PartitionConfirmPage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Left => Action::Navigate(Screen::PartitionConfig),
            KeyCode::Right | KeyCode::Enter => Action::Navigate(Screen::HostSelect),
            _ => Action::Noop,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Partition Confirm ")
            .border_style(Style::default().fg(Color::Yellow));
        let inner = block.inner(rect);
        f.render_widget(block, rect);

        let lines = vec![
            Line::from("Review the partition layout before proceeding."),
            Line::default(),
            Line::from(format!("EFI partition : {}", self.boot_size)),
            Line::from(format!("Swap partition: {}", self.swap_size)),
            Line::from("Root partition : remaining disk"),
            Line::default(),
            Line::from("Right accepts this layout and continues to host selection."),
            status_line(self.status_message.as_deref()),
        ];

        f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
    }
}

fn field_line(active: bool, editing: bool, label: &str, value: &str) -> Line<'static> {
    let marker = if active { ">" } else { " " };
    let mut spans = vec![
        Span::styled(marker, Style::default().fg(Color::Yellow)),
        Span::raw(" "),
        Span::styled(label.to_string(), Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": "),
        Span::raw(value.to_string()),
    ];
    if editing {
        spans.push(Span::styled("  [editing]", Style::default().fg(Color::Green)));
    }
    Line::from(spans)
}

fn status_line(message: Option<&str>) -> Line<'static> {
    match message {
        Some(message) => Line::from(vec![
            Span::styled("status: ", Style::default().fg(Color::Yellow)),
            Span::raw(message.to_string()),
        ]),
        None => Line::default(),
    }
}