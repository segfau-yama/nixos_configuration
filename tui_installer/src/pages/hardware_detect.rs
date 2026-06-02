use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
};

use crate::{
    action::{Action, ConfigChange},
    app::{AppSnapshot, Screen},
    component::Component,
    components::form::{FormFieldRole, FormSection, render_form_section},
    config::{BootType, CpuType, GpuType, HardwareInfo},
    pages::{InstallerPage, form_field, status_field},
    terminal::Frame,
};

const GPU_OPTIONS: &[&str] = &["none", "nvidia", "amd", "intel", "custom"];
const CPU_OPTIONS: &[&str] = &["amd", "intel", "aarch64", "custom"];
const BOOT_OPTIONS: &[&str] = &["systemd-boot", "grub"];

#[derive(Default)]
pub struct HardwareDetectPage {
    hardware: Option<HardwareInfo>,
    status_message: Option<String>,
}

#[derive(Default)]
pub struct GpuSelectPage {
    selected: usize,
    custom_value: String,
    editing_custom: bool,
    status_message: Option<String>,
}

#[derive(Default)]
pub struct CpuSelectPage {
    selected: usize,
    custom_value: String,
    editing_custom: bool,
    status_message: Option<String>,
}

#[derive(Default)]
pub struct BootTypeSelectPage {
    selected: usize,
    status_message: Option<String>,
}

pub fn detect_page() -> Box<dyn InstallerPage> {
    Box::new(HardwareDetectPage::default())
}

pub fn gpu_page() -> Box<dyn InstallerPage> {
    Box::new(GpuSelectPage::default())
}

pub fn cpu_page() -> Box<dyn InstallerPage> {
    Box::new(CpuSelectPage::default())
}

pub fn boot_page() -> Box<dyn InstallerPage> {
    Box::new(BootTypeSelectPage::default())
}

impl InstallerPage for HardwareDetectPage {
    fn screen(&self) -> Screen {
        Screen::HardwareDetect
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.hardware = Some(snapshot.hardware.clone());
        self.status_message = snapshot.status_message.clone();
    }
}

impl Component for HardwareDetectPage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Left => Action::Navigate(Screen::HostnameInput),
            KeyCode::Right | KeyCode::Enter => Action::Navigate(Screen::GpuSelect),
            _ => Action::Noop,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let hardware = self.hardware.clone().unwrap_or(HardwareInfo {
            cpu_brand: "unknown".to_string(),
            gpu_brand: "unknown".to_string(),
            cpu_type: CpuType::Amd,
            gpu_type: GpuType::None,
            boot_type: BootType::SystemdBoot,
        });
        let section = FormSection::new(
            "hardware",
            vec![
                form_field(
                    "detected cpu",
                    hardware.cpu_brand,
                    Some(format!("cpu type: {}", hardware.cpu_type.label())),
                    FormFieldRole::ReadOnly,
                ),
                form_field(
                    "detected gpu",
                    hardware.gpu_brand,
                    Some(format!("gpu type: {}", hardware.gpu_type.label())),
                    FormFieldRole::ReadOnly,
                ),
                form_field(
                    "boot type",
                    hardware.boot_type.label(),
                    Some("detected from firmware".to_string()),
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

impl InstallerPage for GpuSelectPage {
    fn screen(&self) -> Screen {
        Screen::GpuSelect
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.status_message = snapshot.status_message.clone();
        self.custom_value = snapshot.config.gpu_custom.clone();
        self.selected = if !self.custom_value.trim().is_empty() {
            4
        } else {
            match snapshot.config.gpu_type {
                GpuType::None => 0,
                GpuType::Nvidia => 1,
                GpuType::Amd => 2,
                GpuType::Intel => 3,
            }
        };
    }
}

impl Component for GpuSelectPage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        if self.editing_custom {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.editing_custom = false;
                    Action::Noop
                }
                KeyCode::Backspace => {
                    self.custom_value.pop();
                    Action::ConfigChanged(ConfigChange::GpuCustom(self.custom_value.clone()))
                }
                KeyCode::Char(c) => {
                    self.custom_value.push(c);
                    Action::ConfigChanged(ConfigChange::GpuCustom(self.custom_value.clone()))
                }
                _ => Action::Noop,
            }
        } else {
            match key.code {
                KeyCode::Char('q') => Action::Quit,
                KeyCode::Up => self.move_selection(-1),
                KeyCode::Down | KeyCode::Tab => self.move_selection(1),
                KeyCode::Enter if self.selected == 4 => {
                    self.editing_custom = true;
                    Action::Noop
                }
                KeyCode::Left => Action::Navigate(Screen::HardwareDetect),
                KeyCode::Right => {
                    if self.selected == 4 && self.custom_value.trim().is_empty() {
                        Action::SetStatus(Some("Custom GPU label is required".to_string()))
                    } else {
                        Action::Navigate(Screen::CpuSelect)
                    }
                }
                _ => Action::Noop,
            }
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        render_choice_form(
            f,
            rect,
            ChoiceForm {
                title: "gpu",
                label: "selection",
                value: GPU_OPTIONS[self.selected],
                hint: Some(format!(
                    "Space/Up/Down: cycle -> custom | {}",
                    GPU_OPTIONS.join(" / ")
                )),
                custom: Some((&self.custom_value, self.editing_custom)),
                active_field: if self.editing_custom {
                    Some(1)
                } else {
                    Some(0)
                },
                status_message: self.status_message.as_deref(),
            },
        );
    }
}

impl GpuSelectPage {
    fn move_selection(&mut self, delta: isize) -> Action {
        let len = GPU_OPTIONS.len() as isize;
        self.selected = (self.selected as isize + delta).rem_euclid(len) as usize;
        match self.selected {
            0 => Action::Batch(vec![
                Action::ConfigChanged(ConfigChange::GpuType(GpuType::None)),
                Action::ConfigChanged(ConfigChange::GpuCustom(String::new())),
            ]),
            1 => Action::Batch(vec![
                Action::ConfigChanged(ConfigChange::GpuType(GpuType::Nvidia)),
                Action::ConfigChanged(ConfigChange::GpuCustom(String::new())),
            ]),
            2 => Action::Batch(vec![
                Action::ConfigChanged(ConfigChange::GpuType(GpuType::Amd)),
                Action::ConfigChanged(ConfigChange::GpuCustom(String::new())),
            ]),
            3 => Action::Batch(vec![
                Action::ConfigChanged(ConfigChange::GpuType(GpuType::Intel)),
                Action::ConfigChanged(ConfigChange::GpuCustom(String::new())),
            ]),
            _ => Action::Noop,
        }
    }
}

impl InstallerPage for CpuSelectPage {
    fn screen(&self) -> Screen {
        Screen::CpuSelect
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.status_message = snapshot.status_message.clone();
        self.custom_value = snapshot.config.cpu_custom.clone();
        self.selected = if !self.custom_value.trim().is_empty() {
            3
        } else {
            match snapshot.config.cpu_type {
                CpuType::Amd => 0,
                CpuType::Intel => 1,
                CpuType::Aarch64 => 2,
            }
        };
    }
}

impl Component for CpuSelectPage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        if self.editing_custom {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.editing_custom = false;
                    Action::Noop
                }
                KeyCode::Backspace => {
                    self.custom_value.pop();
                    Action::ConfigChanged(ConfigChange::CpuCustom(self.custom_value.clone()))
                }
                KeyCode::Char(c) => {
                    self.custom_value.push(c);
                    Action::ConfigChanged(ConfigChange::CpuCustom(self.custom_value.clone()))
                }
                _ => Action::Noop,
            }
        } else {
            match key.code {
                KeyCode::Char('q') => Action::Quit,
                KeyCode::Up => self.move_selection(-1),
                KeyCode::Down | KeyCode::Tab => self.move_selection(1),
                KeyCode::Enter if self.selected == 3 => {
                    self.editing_custom = true;
                    Action::Noop
                }
                KeyCode::Left => Action::Navigate(Screen::GpuSelect),
                KeyCode::Right => {
                    if self.selected == 3 && self.custom_value.trim().is_empty() {
                        Action::SetStatus(Some("Custom CPU label is required".to_string()))
                    } else {
                        Action::Navigate(Screen::BootTypeSelect)
                    }
                }
                _ => Action::Noop,
            }
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        render_choice_form(
            f,
            rect,
            ChoiceForm {
                title: "cpu",
                label: "selection",
                value: CPU_OPTIONS[self.selected],
                hint: Some(format!(
                    "Space/Up/Down: cycle -> custom | {}",
                    CPU_OPTIONS.join(" / ")
                )),
                custom: Some((&self.custom_value, self.editing_custom)),
                active_field: if self.editing_custom {
                    Some(1)
                } else {
                    Some(0)
                },
                status_message: self.status_message.as_deref(),
            },
        );
    }
}

impl CpuSelectPage {
    fn move_selection(&mut self, delta: isize) -> Action {
        let len = CPU_OPTIONS.len() as isize;
        self.selected = (self.selected as isize + delta).rem_euclid(len) as usize;
        match self.selected {
            0 => Action::Batch(vec![
                Action::ConfigChanged(ConfigChange::CpuType(CpuType::Amd)),
                Action::ConfigChanged(ConfigChange::CpuCustom(String::new())),
            ]),
            1 => Action::Batch(vec![
                Action::ConfigChanged(ConfigChange::CpuType(CpuType::Intel)),
                Action::ConfigChanged(ConfigChange::CpuCustom(String::new())),
            ]),
            2 => Action::Batch(vec![
                Action::ConfigChanged(ConfigChange::CpuType(CpuType::Aarch64)),
                Action::ConfigChanged(ConfigChange::CpuCustom(String::new())),
            ]),
            _ => Action::Noop,
        }
    }
}

impl InstallerPage for BootTypeSelectPage {
    fn screen(&self) -> Screen {
        Screen::BootTypeSelect
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.status_message = snapshot.status_message.clone();
        self.selected = match snapshot.config.boot_type {
            BootType::SystemdBoot => 0,
            BootType::Grub => 1,
        };
    }
}

impl Component for BootTypeSelectPage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Up => self.move_selection(-1),
            KeyCode::Down | KeyCode::Tab => self.move_selection(1),
            KeyCode::Left => Action::Navigate(Screen::CpuSelect),
            KeyCode::Right | KeyCode::Enter => Action::Navigate(Screen::KeyboardSelect),
            _ => Action::Noop,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        render_choice_form(
            f,
            rect,
            ChoiceForm {
                title: "boot type",
                label: "selection",
                value: BOOT_OPTIONS[self.selected],
                hint: Some(format!(
                    "Space/Up/Down: cycle | {}",
                    BOOT_OPTIONS.join(" / ")
                )),
                custom: None,
                active_field: Some(0),
                status_message: self.status_message.as_deref(),
            },
        );
    }
}

impl BootTypeSelectPage {
    fn move_selection(&mut self, delta: isize) -> Action {
        let len = BOOT_OPTIONS.len() as isize;
        self.selected = (self.selected as isize + delta).rem_euclid(len) as usize;
        Action::ConfigChanged(ConfigChange::BootType(if self.selected == 0 {
            BootType::SystemdBoot
        } else {
            BootType::Grub
        }))
    }
}

struct ChoiceForm<'a> {
    title: &'a str,
    label: &'a str,
    value: &'a str,
    hint: Option<String>,
    custom: Option<(&'a str, bool)>,
    active_field: Option<usize>,
    status_message: Option<&'a str>,
}

fn render_choice_form(f: &mut Frame, rect: Rect, form: ChoiceForm<'_>) {
    let mut fields = vec![form_field(
        form.label,
        form.value,
        form.hint,
        FormFieldRole::Choice,
    )];
    if let Some((value, _editing)) = form.custom {
        fields.push(form_field(
            "custom value",
            value,
            Some("Type when selection is custom".to_string()),
            FormFieldRole::Text,
        ));
    }
    fields.push(status_field(form.status_message));

    let section = FormSection::new(
        form.title,
        fields,
        form.active_field,
        form.custom.map(|(_, editing)| editing).unwrap_or(false),
    );
    render_form_section(f, rect, &section);
}
