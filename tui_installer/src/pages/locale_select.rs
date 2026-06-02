use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
};

use crate::{
    action::{Action, ConfigChange},
    app::{AppSnapshot, Screen},
    component::Component,
    components::form::{FormFieldRole, FormSection, render_form_section},
    config::{KEYBOARD_OPTIONS, LOCALE_OPTIONS, TIMEZONE_OPTIONS},
    pages::{InstallerPage, form_field, status_field},
    terminal::Frame,
};

#[derive(Default)]
pub struct KeyboardPage {
    selected: usize,
    status_message: Option<String>,
}

#[derive(Default)]
pub struct LocalePage {
    selected: usize,
    status_message: Option<String>,
}

#[derive(Default)]
pub struct TimezonePage {
    selected: usize,
    status_message: Option<String>,
}

#[derive(Default)]
pub struct SshPage {
    enabled: bool,
    status_message: Option<String>,
}

#[derive(Default)]
pub struct StoragePage {
    enabled: bool,
    status_message: Option<String>,
}

pub fn keyboard_page() -> Box<dyn InstallerPage> {
    Box::new(KeyboardPage::default())
}

pub fn locale_page() -> Box<dyn InstallerPage> {
    Box::new(LocalePage::default())
}

pub fn timezone_page() -> Box<dyn InstallerPage> {
    Box::new(TimezonePage::default())
}

pub fn ssh_page() -> Box<dyn InstallerPage> {
    Box::new(SshPage::default())
}

pub fn storage_page() -> Box<dyn InstallerPage> {
    Box::new(StoragePage::default())
}

impl InstallerPage for KeyboardPage {
    fn screen(&self) -> Screen {
        Screen::KeyboardSelect
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.status_message = snapshot.status_message.clone();
        self.selected = KEYBOARD_OPTIONS
            .iter()
            .position(|value| *value == snapshot.config.keyboard)
            .unwrap_or(0);
    }
}

impl Component for KeyboardPage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        select_list_key(
            key,
            &mut self.selected,
            KEYBOARD_OPTIONS.len(),
            Screen::BootTypeSelect,
            Screen::LocaleSelect,
            |index| ConfigChange::Keyboard(KEYBOARD_OPTIONS[index].to_string()),
        )
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        render_select_page(
            f,
            rect,
            "keyboard",
            "selection",
            KEYBOARD_OPTIONS,
            self.selected,
            self.status_message.as_deref(),
        );
    }
}

impl InstallerPage for LocalePage {
    fn screen(&self) -> Screen {
        Screen::LocaleSelect
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.status_message = snapshot.status_message.clone();
        self.selected = LOCALE_OPTIONS
            .iter()
            .position(|value| *value == snapshot.config.locale)
            .unwrap_or(0);
    }
}

impl Component for LocalePage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        select_list_key(
            key,
            &mut self.selected,
            LOCALE_OPTIONS.len(),
            Screen::KeyboardSelect,
            Screen::TimezoneSelect,
            |index| ConfigChange::Locale(LOCALE_OPTIONS[index].to_string()),
        )
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        render_select_page(
            f,
            rect,
            "locale",
            "selection",
            LOCALE_OPTIONS,
            self.selected,
            self.status_message.as_deref(),
        );
    }
}

impl InstallerPage for TimezonePage {
    fn screen(&self) -> Screen {
        Screen::TimezoneSelect
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.status_message = snapshot.status_message.clone();
        self.selected = TIMEZONE_OPTIONS
            .iter()
            .position(|value| *value == snapshot.config.timezone)
            .unwrap_or(0);
    }
}

impl Component for TimezonePage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        select_list_key(
            key,
            &mut self.selected,
            TIMEZONE_OPTIONS.len(),
            Screen::LocaleSelect,
            Screen::SshToggle,
            |index| ConfigChange::Timezone(TIMEZONE_OPTIONS[index].to_string()),
        )
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        render_select_page(
            f,
            rect,
            "timezone",
            "selection",
            TIMEZONE_OPTIONS,
            self.selected,
            self.status_message.as_deref(),
        );
    }
}

impl InstallerPage for SshPage {
    fn screen(&self) -> Screen {
        Screen::SshToggle
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.enabled = snapshot.config.ssh_enabled;
        self.status_message = snapshot.status_message.clone();
    }
}

impl Component for SshPage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Left => Action::Navigate(Screen::TimezoneSelect),
            KeyCode::Right => Action::Navigate(Screen::StorageToggle),
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.enabled = !self.enabled;
                Action::ConfigChanged(ConfigChange::SshEnabled(self.enabled))
            }
            _ => Action::Noop,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        render_toggle_page(
            f,
            rect,
            "ssh",
            "enabled",
            self.enabled,
            "Space or Enter toggles OpenSSH access",
            self.status_message.as_deref(),
        );
    }
}

impl InstallerPage for StoragePage {
    fn screen(&self) -> Screen {
        Screen::StorageToggle
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.enabled = snapshot.config.storage_enabled;
        self.status_message = snapshot.status_message.clone();
    }
}

impl Component for StoragePage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Left => Action::Navigate(Screen::SshToggle),
            KeyCode::Right => Action::Navigate(Screen::UserMenu),
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.enabled = !self.enabled;
                Action::ConfigChanged(ConfigChange::StorageEnabled(self.enabled))
            }
            _ => Action::Noop,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        render_toggle_page(
            f,
            rect,
            "storage",
            "enabled",
            self.enabled,
            "Space or Enter toggles the optional storage module",
            self.status_message.as_deref(),
        );
    }
}

fn select_list_key<F>(
    key: KeyEvent,
    selected: &mut usize,
    len: usize,
    previous: Screen,
    next: Screen,
    change: F,
) -> Action
where
    F: Fn(usize) -> ConfigChange,
{
    match key.code {
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Up => {
            *selected = (*selected as isize - 1).rem_euclid(len as isize) as usize;
            Action::ConfigChanged(change(*selected))
        }
        KeyCode::Down | KeyCode::Tab => {
            *selected = (*selected as isize + 1).rem_euclid(len as isize) as usize;
            Action::ConfigChanged(change(*selected))
        }
        KeyCode::Left => Action::Navigate(previous),
        KeyCode::Right | KeyCode::Enter => Action::Navigate(next),
        _ => Action::Noop,
    }
}

fn render_select_page(
    f: &mut Frame,
    rect: Rect,
    title: &str,
    label: &str,
    options: &[&str],
    selected: usize,
    status_message: Option<&str>,
) {
    let section = FormSection::new(
        title,
        vec![
            form_field(
                label,
                options[selected],
                Some(format!("Space/Up/Down: cycle | {}", options.join(" / "))),
                FormFieldRole::Choice,
            ),
            status_field(status_message),
        ],
        Some(0),
        false,
    );
    render_form_section(f, rect, &section);
}

fn render_toggle_page(
    f: &mut Frame,
    rect: Rect,
    title: &str,
    label: &str,
    value: bool,
    hint: &str,
    status_message: Option<&str>,
) {
    let section = FormSection::new(
        title,
        vec![
            form_field(
                label,
                value.to_string(),
                Some(hint.to_string()),
                FormFieldRole::Toggle,
            ),
            status_field(status_message),
        ],
        Some(0),
        false,
    );
    render_form_section(f, rect, &section);
}
