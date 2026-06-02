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
pub struct WelcomePage {
    status_message: Option<String>,
}

pub fn page() -> Box<dyn InstallerPage> {
    Box::new(WelcomePage::default())
}

impl InstallerPage for WelcomePage {
    fn screen(&self) -> Screen {
        Screen::Welcome
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.status_message = snapshot.status_message.clone();
    }
}

impl Component for WelcomePage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Right | KeyCode::Enter => Action::CheckNetwork,
            _ => Action::Noop,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let section = FormSection::new(
            "welcome",
            vec![
                status_field(self.status_message.as_deref()),
                form_field(
                    "mode",
                    "interactive installer",
                    Some("Press Enter or Right to run network check".to_string()),
                    FormFieldRole::ReadOnly,
                ),
                form_field(
                    "architecture",
                    "component pages + shared form renderer",
                    Some("Each page owns event handling, state, update, and render".to_string()),
                    FormFieldRole::ReadOnly,
                ),
            ],
            None,
            false,
        );
        render_form_section(f, rect, &section);
    }
}
