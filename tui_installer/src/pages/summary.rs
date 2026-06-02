use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
};

use crate::{
    action::Action,
    app::{AppSnapshot, Screen},
    component::Component,
    components::{
        Popup,
        form::{FormFieldRole, FormSection, render_form_section},
    },
    config::InstallConfig,
    pages::{InstallerPage, form_field, status_field},
    terminal::Frame,
};

#[derive(Default)]
pub struct SummaryPage {
    config: InstallConfig,
    confirmation: String,
    editing: bool,
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

    fn popup(&self) -> Option<Popup> {
        Some(Popup::new(
            "Summary",
            72,
            36,
            FormSection::new(
                "install confirmation",
                vec![form_field(
                    "confirmation",
                    self.confirmation.clone(),
                    Some("Type yes before starting install".to_string()),
                    FormFieldRole::Text,
                )],
                Some(0),
                self.editing,
            ),
        ))
    }
}

impl Component for SummaryPage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        if self.editing {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.editing = false;
                    Action::Noop
                }
                KeyCode::Backspace => {
                    self.confirmation.pop();
                    Action::Noop
                }
                KeyCode::Char(c) => {
                    self.confirmation.push(c);
                    Action::Noop
                }
                _ => Action::Noop,
            }
        } else {
            match key.code {
                KeyCode::Char('q') => Action::Quit,
                KeyCode::Left => Action::Navigate(Screen::UserMenu),
                KeyCode::Enter => {
                    self.editing = true;
                    Action::Noop
                }
                KeyCode::Right => {
                    if self.confirmation.trim() == "yes" {
                        Action::StartInstall
                    } else {
                        Action::SetStatus(Some(
                            "Type 'yes' in confirm field before starting install".to_string(),
                        ))
                    }
                }
                _ => Action::Noop,
            }
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let section = FormSection::new(
            "summary",
            vec![
                form_field(
                    "repository",
                    self.config.repository.clone(),
                    None,
                    FormFieldRole::ReadOnly,
                ),
                form_field(
                    "device",
                    self.config.device.clone(),
                    None,
                    FormFieldRole::ReadOnly,
                ),
                form_field(
                    "hostname",
                    self.config.hostname.clone(),
                    None,
                    FormFieldRole::ReadOnly,
                ),
                form_field(
                    "keyboard",
                    self.config.keyboard.clone(),
                    None,
                    FormFieldRole::ReadOnly,
                ),
                form_field(
                    "locale",
                    self.config.locale.clone(),
                    None,
                    FormFieldRole::ReadOnly,
                ),
                form_field(
                    "timezone",
                    self.config.timezone.clone(),
                    None,
                    FormFieldRole::ReadOnly,
                ),
                form_field(
                    "storage",
                    self.config.storage_enabled.to_string(),
                    None,
                    FormFieldRole::Toggle,
                ),
                form_field(
                    "gpu",
                    gpu_display(&self.config),
                    None,
                    FormFieldRole::ReadOnly,
                ),
                form_field(
                    "cpu",
                    cpu_display(&self.config),
                    None,
                    FormFieldRole::ReadOnly,
                ),
                form_field(
                    "boot",
                    self.config.boot_type.label(),
                    None,
                    FormFieldRole::ReadOnly,
                ),
                form_field(
                    "users",
                    self.config.users.len().to_string(),
                    None,
                    FormFieldRole::ReadOnly,
                ),
                status_field(self.status_message.as_deref()),
            ],
            None,
            false,
        );
        render_form_section(f, rect, &section);
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
