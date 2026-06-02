use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::{
    action::Action,
    app::{AppSnapshot, Screen},
    component::Component,
    config::InstallConfig,
    pages::InstallerPage,
    terminal::Frame,
};

#[derive(Default)]
pub struct SummaryPage {
    config: InstallConfig,
    status_message: Option<String>,
}

pub fn page() -> Box<dyn InstallerPage> {
    Box::new(SummaryPage::default())
}

impl InstallerPage for SummaryPage {
    fn screen(&self) -> Screen {
        Screen::Summary
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.config = snapshot.config.clone();
        self.status_message = snapshot.status_message.clone();
    }
}

impl Component for SummaryPage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Left => Action::Navigate(Screen::UserMenu),
            KeyCode::Right | KeyCode::Enter => Action::StartInstall,
            _ => Action::Noop,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Summary ")
            .border_style(Style::default().fg(Color::Magenta));
        let inner = block.inner(rect);
        f.render_widget(block, rect);

        let mut lines = vec![
            Line::from("Review the selected values and then start the install flow."),
            Line::default(),
            Line::from(format!("github user : {}", self.config.github_username)),
            Line::from(format!("repository  : {}", self.config.repository)),
            Line::from(format!("device      : {}", self.config.device)),
            Line::from(format!("hostname    : {}", self.config.hostname)),
            Line::from(format!("keyboard    : {}", self.config.keyboard)),
            Line::from(format!("locale      : {}", self.config.locale)),
            Line::from(format!("timezone    : {}", self.config.timezone)),
            Line::from(format!("gpu         : {}", gpu_display(&self.config))),
            Line::from(format!("cpu         : {}", cpu_display(&self.config))),
            Line::from(format!("boot        : {}", self.config.boot_type.label())),
            Line::from(format!("users       : {}", self.config.users.len())),
            Line::default(),
            Line::from("Press Right to switch to the simulated install log."),
        ];
        if let Some(message) = self.status_message.as_ref() {
            lines.push(Line::from(vec![
                Span::styled("status: ", Style::default().fg(Color::Yellow)),
                Span::raw(message.clone()),
            ]));
        }

        f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
    }
}

fn gpu_display(config: &InstallConfig) -> String {
    if config.gpu_custom.trim().is_empty() {
        config.gpu_type.label().to_string()
    } else {
        config.gpu_custom.clone()
    }
}

fn cpu_display(config: &InstallConfig) -> String {
    if config.cpu_custom.trim().is_empty() {
        config.cpu_type.label().to_string()
    } else {
        config.cpu_custom.clone()
    }
}