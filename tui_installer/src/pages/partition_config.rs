use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
};

use crate::{
    action::{Action, ConfigChange},
    app::{AppSnapshot, Screen},
    component::Component,
    components::{
        Popup,
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
    confirmation: String,
    editing: bool,
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
            36,
            FormSection::new(
                "partition confirmation",
                vec![form_field(
                    "confirmation",
                    self.confirmation.clone(),
                    Some("Type yes to confirm destructive partitioning".to_string()),
                    FormFieldRole::Text,
                )],
                Some(0),
                self.editing,
            ),
        ))
    }
}

impl Component for PartitionConfirmPage {
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
                KeyCode::Left | KeyCode::Esc => Action::Navigate(Screen::PartitionConfig),
                KeyCode::Enter => {
                    self.editing = true;
                    Action::Noop
                }
                KeyCode::Right => {
                    if self.confirmation.trim() == "yes" {
                        Action::Navigate(Screen::HostSelect)
                    } else {
                        Action::SetStatus(Some(
                            "Type 'yes' in confirm field before continuing".to_string(),
                        ))
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
