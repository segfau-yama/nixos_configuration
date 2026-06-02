use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
};

use crate::{
    action::Action,
    app::{AppSnapshot, Screen},
    component::Component,
    components::form::{FormFieldRole, FormSection, render_form_section},
    pages::{InstallerPage, form_field, status_field},
    terminal::Frame,
};

#[derive(Default)]
pub struct DonePage {
    install_log: Vec<String>,
    status_message: Option<String>,
}

pub fn page() -> Box<dyn InstallerPage> {
    Box::new(DonePage::default())
}

impl InstallerPage for DonePage {
    fn screen(&self) -> Screen {
        Screen::Done
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.install_log = snapshot.install_log.clone();
        self.status_message = snapshot.status_message.clone();
    }
}

impl Component for DonePage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Left | KeyCode::Right | KeyCode::Enter => Action::Navigate(Screen::Summary),
            _ => Action::Noop,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let start = self.install_log.len().saturating_sub(12);
        let log = if self.install_log.is_empty() {
            "No install log entries".to_string()
        } else {
            self.install_log[start..].join("\n")
        };
        let section = FormSection::new(
            "done",
            vec![
                form_field(
                    "result",
                    "install flow complete",
                    Some("Enter returns to Summary, q quits".to_string()),
                    FormFieldRole::ReadOnly,
                ),
                form_field("install log", log, None, FormFieldRole::Log),
                status_field(self.status_message.as_deref()),
            ],
            None,
            false,
        );
        render_form_section(f, rect, &section);
    }
}
