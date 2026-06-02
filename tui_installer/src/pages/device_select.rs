use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
};

use crate::{
    action::{Action, ConfigChange},
    app::{AppSnapshot, Screen},
    component::Component,
    components::form::{FormFieldRole, FormSection, render_form_section},
    config::DeviceOption,
    pages::{InstallerPage, form_field, status_field},
    terminal::Frame,
};

#[derive(Default)]
pub struct DeviceSelectPage {
    devices: Vec<DeviceOption>,
    selected: usize,
    current_device: String,
    status_message: Option<String>,
}

pub fn page() -> Box<dyn InstallerPage> {
    Box::new(DeviceSelectPage::default())
}

impl InstallerPage for DeviceSelectPage {
    fn screen(&self) -> Screen {
        Screen::DeviceSelect
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.devices = snapshot.devices.clone();
        self.current_device = snapshot.config.device.clone();
        self.status_message = snapshot.status_message.clone();
        if let Some(index) = self
            .devices
            .iter()
            .position(|device| device.path == self.current_device)
        {
            self.selected = index;
        }
    }
}

impl Component for DeviceSelectPage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Up => self.move_selection(-1),
            KeyCode::Down | KeyCode::Tab | KeyCode::Char(' ') => self.move_selection(1),
            KeyCode::Left => Action::Navigate(Screen::GitHubLogin),
            KeyCode::Right | KeyCode::Enter => {
                if let Some(device) = self.devices.get(self.selected) {
                    Action::Batch(vec![
                        Action::ConfigChanged(ConfigChange::Device(device.path.clone())),
                        Action::Navigate(Screen::PartitionConfig),
                    ])
                } else {
                    Action::SetStatus(Some("No block devices available".to_string()))
                }
            }
            _ => Action::Noop,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let mut fields = Vec::new();
        if self.devices.is_empty() {
            fields.push(form_field(
                "target device",
                "no install target disks detected",
                Some("lsblk did not return any TYPE=disk entries".to_string()),
                FormFieldRole::ReadOnly,
            ));
        } else if let Some(device) = self.devices.get(self.selected) {
            fields.push(form_field(
                "target device",
                device.label(),
                Some(format!(
                    "{} / {} - Tab/Up/Down/Space: change, Enter: confirm",
                    self.selected + 1,
                    self.devices.len()
                )),
                FormFieldRole::Choice,
            ));
        }
        if self.status_message.is_some() {
            fields.push(status_field(self.status_message.as_deref()));
        }

        let section = FormSection::new(
            "device",
            fields,
            if self.devices.is_empty() {
                None
            } else {
                Some(0)
            },
            false,
        );
        render_form_section(f, rect, &section);
    }
}

impl DeviceSelectPage {
    fn move_selection(&mut self, delta: isize) -> Action {
        if self.devices.is_empty() {
            return Action::Noop;
        }
        let len = self.devices.len() as isize;
        self.selected = (self.selected as isize + delta).rem_euclid(len) as usize;
        Action::ConfigChanged(ConfigChange::Device(
            self.devices[self.selected].path.clone(),
        ))
    }
}
