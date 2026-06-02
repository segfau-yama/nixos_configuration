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
                KeyCode::Right => Action::PrepareRepository,
                _ => Action::Noop,
            }
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let section = FormSection::new(
            "github",
            vec![
                form_field(
                    "github user",
                    self.github_username.clone(),
                    Some("Used when repository is empty or repo name only".to_string()),
                    FormFieldRole::Text,
                ),
                form_field(
                    "repository",
                    self.repository.clone(),
                    Some("owner/name, repo name, URL, or empty for default".to_string()),
                    FormFieldRole::Text,
                ),
                form_field(
                    "clone path",
                    self.repository_path.clone(),
                    Some("Working clone used before copying to /mnt/etc/nixos".to_string()),
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
