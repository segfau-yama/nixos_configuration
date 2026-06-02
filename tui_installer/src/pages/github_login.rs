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
    pages::InstallerPage,
    terminal::Frame,
};

#[derive(Default)]
pub struct GitHubLoginPage {
    active_field: usize,
    editing: bool,
    github_username: String,
    repository: String,
    repository_path: String,
    status_message: Option<String>,
}

pub fn page() -> Box<dyn InstallerPage> {
    Box::new(GitHubLoginPage::default())
}

impl InstallerPage for GitHubLoginPage {
    fn screen(&self) -> Screen {
        Screen::GitHubLogin
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.github_username = snapshot.config.github_username.clone();
        self.repository = snapshot.config.repository.clone();
        self.repository_path = snapshot.config.repository_path.clone();
        self.status_message = snapshot.status_message.clone();
    }
}

impl Component for GitHubLoginPage {
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
                    self.active_field = (self.active_field + 1).min(2);
                    Action::Noop
                }
                KeyCode::Enter => {
                    self.editing = true;
                    Action::Noop
                }
                KeyCode::Left => Action::Navigate(Screen::Welcome),
                KeyCode::Right => {
                    if self.github_username.trim().is_empty() {
                        Action::SetStatus(Some("GitHub username is required".to_string()))
                    } else {
                        Action::Navigate(Screen::DeviceSelect)
                    }
                }
                _ => Action::Noop,
            }
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" GitHub Login ")
            .border_style(Style::default().fg(Color::Blue));
        let inner = block.inner(rect);
        f.render_widget(block, rect);

        let lines = vec![
            Line::from("Configure the repository source before installation."),
            Line::default(),
            field_line(
                self.active_field == 0,
                self.editing && self.active_field == 0,
                "GitHub user",
                &self.github_username,
            ),
            field_line(
                self.active_field == 1,
                self.editing && self.active_field == 1,
                "Repository",
                &self.repository,
            ),
            field_line(
                self.active_field == 2,
                self.editing && self.active_field == 2,
                "Clone path",
                &self.repository_path,
            ),
            Line::default(),
            Line::from("Enter edits the selected field. Right continues to disk selection."),
            status_line(self.status_message.as_deref()),
        ];

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
        f.render_widget(paragraph, inner);
    }
}

impl GitHubLoginPage {
    fn current_field_mut(&mut self) -> &mut String {
        match self.active_field {
            0 => &mut self.github_username,
            1 => &mut self.repository,
            _ => &mut self.repository_path,
        }
    }

    fn current_change(&self) -> ConfigChange {
        match self.active_field {
            0 => ConfigChange::GitHubUsername(self.github_username.clone()),
            1 => ConfigChange::Repository(self.repository.clone()),
            _ => ConfigChange::RepositoryPath(self.repository_path.clone()),
        }
    }
}

fn field_line(active: bool, editing: bool, label: &str, value: &str) -> Line<'static> {
    let marker = if active { ">" } else { " " };
    let value = if value.is_empty() { "<empty>" } else { value };
    let mut spans = vec![
        Span::styled(marker, Style::default().fg(Color::Yellow)),
        Span::raw(" "),
        Span::styled(label.to_string(), Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": "),
        Span::raw(value.to_string()),
    ];
    if editing {
        spans.push(Span::styled("  [editing]", Style::default().fg(Color::Green)));
    }
    Line::from(spans)
}

fn status_line(message: Option<&str>) -> Line<'static> {
    match message {
        Some(message) => Line::from(vec![
            Span::styled("status: ", Style::default().fg(Color::Yellow)),
            Span::raw(message.to_string()),
        ]),
        None => Line::default(),
    }
}