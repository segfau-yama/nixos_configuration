use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
};

use crate::{
    action::{Action, ConfigChange},
    app::{AppSnapshot, Screen},
    component::Component,
    components::form::{FormFieldRole, FormSection, render_form_section},
    pages::{InstallerPage, form_field, status_field},
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
                    Action::ConfigChanged(ConfigChange::Hostname("virtual-machine".to_string())),
                    Action::Navigate(Screen::HardwareDetect),
                ]),
                _ => Action::Navigate(Screen::HostnameInput),
            },
            _ => Action::Noop,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let options = ["laptop", "virtual-machine", "custom"];
        let mut fields = options
            .iter()
            .map(|option| {
                form_field(
                    "host preset",
                    *option,
                    Some("Enter confirms preset; custom opens hostname input".to_string()),
                    FormFieldRole::Choice,
                )
            })
            .collect::<Vec<_>>();
        fields.push(status_field(self.status_message.as_deref()));

        let section = FormSection::new("host", fields, Some(self.selected), false);
        render_form_section(f, rect, &section);
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
        let section = FormSection::new(
            "hostname",
            vec![
                form_field(
                    "hostname",
                    self.hostname.clone(),
                    Some("Use lowercase letters, numbers, and hyphen".to_string()),
                    FormFieldRole::Text,
                ),
                status_field(self.status_message.as_deref()),
            ],
            Some(0),
            true,
        );
        render_form_section(f, rect, &section);
    }
}
