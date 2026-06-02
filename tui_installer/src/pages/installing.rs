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
pub struct InstallingPage {
    config: InstallConfig,
    install_log: Vec<String>,
    running: bool,
    plan: Vec<String>,
    step: usize,
}

pub fn page() -> Box<dyn InstallerPage> {
    Box::new(InstallingPage::default())
}

impl InstallerPage for InstallingPage {
    fn screen(&self) -> Screen {
        Screen::Installing
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.config = snapshot.config.clone();
        self.install_log = snapshot.install_log.clone();
    }
}

impl Component for InstallingPage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') => Action::Quit,
            _ => Action::Noop,
        }
    }

    fn update(&mut self, action: Action) -> Action {
        match action {
            Action::StartInstall => {
                self.running = true;
                self.plan = build_plan(&self.config);
                self.step = 0;
                Action::AppendInstallLog("Preparing installation plan".to_string())
            }
            Action::Tick if self.running => {
                if let Some(line) = self.plan.get(self.step).cloned() {
                    self.step += 1;
                    Action::AppendInstallLog(line)
                } else {
                    self.running = false;
                    Action::InstallComplete
                }
            }
            _ => Action::Noop,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Installing ")
            .border_style(Style::default().fg(Color::Yellow));
        let inner = block.inner(rect);
        f.render_widget(block, rect);

        let mut lines = vec![
            Line::from("This is a simulated async install log driven by Tick actions."),
            Line::default(),
        ];
        if self.install_log.is_empty() {
            lines.push(Line::from("Waiting for the first install step..."));
        } else {
            for entry in &self.install_log {
                lines.push(Line::from(Span::raw(entry.clone())));
            }
        }

        f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
    }
}

fn build_plan(config: &InstallConfig) -> Vec<String> {
    vec![
        format!("Cloning configuration from {}", config.repository),
        format!("Selecting target device {}", config.device),
        format!("Creating EFI partition with size {}", config.boot_size),
        format!("Creating swap partition with size {}", config.swap_size),
        format!("Writing hostname {}", config.hostname),
        format!("Applying locale {} and timezone {}", config.locale, config.timezone),
        format!("Configuring {} users", config.users.len()),
        "Finalizing boot loader configuration".to_string(),
    ]
}