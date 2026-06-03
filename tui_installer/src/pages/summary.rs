use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
};

use crate::{
    action::Action,
    app::{AppSnapshot, Screen},
    component::Component,
    components::{
        ConfirmChoice, ConfirmChoiceAction, Popup,
        form::{FormFieldRole, FormSection, render_form_section},
    },
    config::InstallConfig,
    pages::{InstallerPage, form_field, status_field},
    terminal::Frame,
};

#[derive(Default)]
pub struct SummaryPage {
    config: InstallConfig,
    confirmation: ConfirmChoice,
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
            30,
            FormSection::new(
                "install confirmation",
                vec![self.confirmation.field(
                    "start install",
                    "Choose yes to open the install log and start",
                )],
                Some(0),
                false,
            ),
        ))
    }
}

impl Component for SummaryPage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        if key.code == KeyCode::Char('q') {
            return Action::Quit;
        }

        match self.confirmation.handle_key(key) {
            ConfirmChoiceAction::Submit(true) => Action::Navigate(Screen::Done),
            ConfirmChoiceAction::Submit(false) | ConfirmChoiceAction::Cancel => {
                Action::Navigate(Screen::UserMenu)
            }
            ConfirmChoiceAction::Noop => Action::Noop,
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
