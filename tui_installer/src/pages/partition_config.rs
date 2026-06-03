use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
};

use crate::{
    action::{Action, ConfigChange},
    app::{AppSnapshot, Screen},
    component::Component,
    components::{
        ConfirmChoice, ConfirmChoiceAction, Popup,
        form::{FormFieldRole, FormSection, render_form_section},
    },
    pages::{InstallerPage, form_field, status_field},
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
    device: String,
    boot_size: String,
    swap_size: String,
    confirmation: ConfirmChoice,
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
        let section = FormSection::new(
            "partition",
            vec![
                form_field(
                    "boot size",
                    self.boot_size.clone(),
                    Some("Default: 512MiB".to_string()),
                    FormFieldRole::Text,
                ),
                form_field(
                    "swap size",
                    self.swap_size.clone(),
                    Some("Use 0 to disable swap; root uses remaining disk".to_string()),
                    FormFieldRole::Text,
                ),
                status_field(self.status_message.as_deref()),
            ],
            Some(self.active_field),
            self.editing,
        );
        render_form_section(f, rect, &section);
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
        self.device = snapshot.config.device.clone();
        self.boot_size = snapshot.config.boot_size.clone();
        self.swap_size = snapshot.config.swap_size.clone();
        self.status_message = snapshot.status_message.clone();
    }

    fn popup(&self) -> Option<Popup> {
        Some(Popup::new(
            "Partition Confirm",
            72,
            30,
            FormSection::new(
                "partition confirmation",
                vec![self.confirmation.field(
                    "repartition disk",
                    "Choose yes to continue; no cancels destructive partitioning",
                )],
                Some(0),
                false,
            ),
        ))
    }
}

impl Component for PartitionConfirmPage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        if key.code == KeyCode::Char('q') {
            return Action::Quit;
        }

        match self.confirmation.handle_key(key) {
            ConfirmChoiceAction::Submit(true) => Action::Navigate(Screen::HostSelect),
            ConfirmChoiceAction::Submit(false) | ConfirmChoiceAction::Cancel => {
                Action::Navigate(Screen::PartitionConfig)
            }
            ConfirmChoiceAction::Noop => Action::Noop,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let section = FormSection::new(
            "partition",
            vec![
                form_field(
                    "target disk",
                    self.device.clone(),
                    Some("This disk will be repartitioned and formatted".to_string()),
                    FormFieldRole::ReadOnly,
                ),
                form_field(
                    "layout",
                    format!(
                        "boot: {}, swap: {}, root: remaining disk",
                        self.boot_size, self.swap_size
                    ),
                    Some("Confirm destructive operation in popup".to_string()),
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
