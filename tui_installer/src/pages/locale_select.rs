use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::{
    action::{Action, ConfigChange},
    app::{AppSnapshot, Screen},
    component::Component,
    config::{KEYBOARD_OPTIONS, LOCALE_OPTIONS, TIMEZONE_OPTIONS},
    pages::InstallerPage,
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
            " Keyboard ",
            "Choose the keyboard layout.",
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
            " Locale ",
            "Pick the system locale.",
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
            " Timezone ",
            "Pick the timezone used on the installed system.",
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
            KeyCode::Right => Action::Navigate(Screen::UserMenu),
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.enabled = !self.enabled;
                Action::ConfigChanged(ConfigChange::SshEnabled(self.enabled))
            }
            _ => Action::Noop,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" SSH ")
            .border_style(Style::default().fg(Color::Blue));
        let inner = block.inner(rect);
        f.render_widget(block, rect);

        let lines = vec![
            Line::from("Toggle OpenSSH access for the installed system."),
            Line::default(),
            Line::from(vec![
                Span::styled("value", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(": "),
                Span::styled(
                    self.enabled.to_string(),
                    if self.enabled {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Red)
                    },
                ),
            ]),
            Line::default(),
            Line::from("Space or Enter toggles the value. Right continues to user setup."),
            match self.status_message.as_ref() {
                Some(message) => Line::from(vec![
                    Span::styled("status: ", Style::default().fg(Color::Yellow)),
                    Span::raw(message.clone()),
                ]),
                None => Line::default(),
            },
        ];

        f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
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
    description: &str,
    options: &[&str],
    selected: usize,
    status_message: Option<&str>,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Blue));
    let inner = block.inner(rect);
    f.render_widget(block, rect);

    let mut lines = vec![Line::from(description), Line::default()];
    for (index, option) in options.iter().enumerate() {
        lines.push(Line::from(vec![
            Span::styled(
                if index == selected { "> " } else { "  " },
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                option.to_string(),
                if index == selected {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                },
            ),
        ]));
    }
    if let Some(message) = status_message {
        lines.push(Line::default());
        lines.push(Line::from(vec![
            Span::styled("status: ", Style::default().fg(Color::Yellow)),
            Span::raw(message.to_string()),
        ]));
    }

    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}