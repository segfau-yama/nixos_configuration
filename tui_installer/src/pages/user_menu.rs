use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::{
    action::{Action, PendingUserChange},
    app::{AppSnapshot, PendingUser, Screen},
    component::Component,
    config::UserType,
    pages::InstallerPage,
    terminal::Frame,
};

const USER_MENU_OPTIONS: &[&str] = &[
    "jade-core",
    "jade-office",
    "jade-gaming",
    "jade-develop",
    "jade-full",
    "custom user",
    "finish",
];

#[derive(Default)]
pub struct UserMenuPage {
    selected: usize,
    status_message: Option<String>,
}

#[derive(Default)]
pub struct PresetPasswordPage {
    active_field: usize,
    editing: bool,
    password: String,
    password_confirm: String,
    status_message: Option<String>,
}

#[derive(Default)]
pub struct CustomBasicPage {
    active_field: usize,
    editing: bool,
    username: String,
    display_name: String,
    status_message: Option<String>,
}

#[derive(Default)]
pub struct CustomTypePage {
    selected: usize,
    status_message: Option<String>,
}

#[derive(Default)]
pub struct CustomProgramsPage {
    selected: usize,
    user_type: Option<UserType>,
    selected_programs: Vec<String>,
    status_message: Option<String>,
}

#[derive(Default)]
pub struct CustomPasswordPage {
    active_field: usize,
    editing: bool,
    password: String,
    password_confirm: String,
    status_message: Option<String>,
}

#[derive(Default)]
pub struct UserAddResultPage {
    last_user: Option<PendingUser>,
    status_message: Option<String>,
}

pub fn menu_page() -> Box<dyn InstallerPage> {
    Box::new(UserMenuPage::default())
}

pub fn preset_password_page() -> Box<dyn InstallerPage> {
    Box::new(PresetPasswordPage::default())
}

pub fn custom_basic_page() -> Box<dyn InstallerPage> {
    Box::new(CustomBasicPage::default())
}

pub fn custom_type_page() -> Box<dyn InstallerPage> {
    Box::new(CustomTypePage::default())
}

pub fn custom_programs_page() -> Box<dyn InstallerPage> {
    Box::new(CustomProgramsPage::default())
}

pub fn custom_password_page() -> Box<dyn InstallerPage> {
    Box::new(CustomPasswordPage::default())
}

pub fn result_page() -> Box<dyn InstallerPage> {
    Box::new(UserAddResultPage::default())
}

impl InstallerPage for UserMenuPage {
    fn screen(&self) -> Screen {
        Screen::UserMenu
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.status_message = snapshot.status_message.clone();
    }
}

impl Component for UserMenuPage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
                Action::Noop
            }
            KeyCode::Down | KeyCode::Tab => {
                self.selected = (self.selected + 1).min(USER_MENU_OPTIONS.len() - 1);
                Action::Noop
            }
            KeyCode::Left => Action::Navigate(Screen::SshToggle),
            KeyCode::Right | KeyCode::Enter => match self.selected {
                0 => Action::StartPresetUser(PendingUser::preset(
                    "jade-core",
                    "Jade Core",
                    UserType::Cui,
                    &["base", "media"],
                )),
                1 => Action::StartPresetUser(PendingUser::preset(
                    "jade-office",
                    "Jade Office",
                    UserType::Gui,
                    &["office", "browser", "sns"],
                )),
                2 => Action::StartPresetUser(PendingUser::preset(
                    "jade-gaming",
                    "Jade Gaming",
                    UserType::Gui,
                    &["gaming", "browser", "media"],
                )),
                3 => Action::StartPresetUser(PendingUser::preset(
                    "jade-develop",
                    "Jade Develop",
                    UserType::Gui,
                    &["programming", "browser", "office"],
                )),
                4 => Action::StartPresetUser(PendingUser::preset(
                    "jade-full",
                    "Jade Full",
                    UserType::Gui,
                    &["browser", "office", "media", "programming", "gaming"],
                )),
                5 => Action::Batch(vec![
                    Action::PendingUserChanged(PendingUserChange::Replace(PendingUser::custom())),
                    Action::Navigate(Screen::CustomUserBasic),
                ]),
                _ => Action::Navigate(Screen::Summary),
            },
            _ => Action::Noop,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Users ")
            .border_style(Style::default().fg(Color::Cyan));
        let inner = block.inner(rect);
        f.render_widget(block, rect);

        let mut lines = vec![
            Line::from("Add preset or custom users before starting the install."),
            Line::default(),
        ];
        for (index, option) in USER_MENU_OPTIONS.iter().enumerate() {
            lines.push(Line::from(vec![
                Span::styled(
                    if index == self.selected { "> " } else { "  " },
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    option.to_string(),
                    if index == self.selected {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    },
                ),
            ]));
        }
        if let Some(message) = self.status_message.as_ref() {
            lines.push(Line::default());
            lines.push(Line::from(vec![
                Span::styled("status: ", Style::default().fg(Color::Yellow)),
                Span::raw(message.clone()),
            ]));
        }

        f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
    }
}

impl InstallerPage for PresetPasswordPage {
    fn screen(&self) -> Screen {
        Screen::PresetUserPassword
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.status_message = snapshot.status_message.clone();
        if let Some(user) = snapshot.pending_user.as_ref() {
            self.password = user.password.clone();
            self.password_confirm = user.password_confirm.clone();
        }
    }
}

impl Component for PresetPasswordPage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        password_page_key(
            key,
            &mut self.active_field,
            &mut self.editing,
            &mut self.password,
            &mut self.password_confirm,
            Screen::UserMenu,
        )
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        render_password_page(
            f,
            rect,
            " Preset Password ",
            &self.password,
            &self.password_confirm,
            self.active_field,
            self.editing,
            self.status_message.as_deref(),
        );
    }
}

impl InstallerPage for CustomBasicPage {
    fn screen(&self) -> Screen {
        Screen::CustomUserBasic
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.status_message = snapshot.status_message.clone();
        if let Some(user) = snapshot.pending_user.as_ref() {
            self.username = user.username.clone();
            self.display_name = user.display_name.clone();
        }
    }
}

impl Component for CustomBasicPage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        if self.editing {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => {
                    self.editing = false;
                    Action::Noop
                }
                KeyCode::Backspace => {
                    self.current_field_mut().pop();
                    Action::PendingUserChanged(self.current_change())
                }
                KeyCode::Char(c) => {
                    self.current_field_mut().push(c);
                    Action::PendingUserChanged(self.current_change())
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
                KeyCode::Left => Action::Navigate(Screen::UserMenu),
                KeyCode::Right => {
                    if self.username.trim().is_empty() || self.display_name.trim().is_empty() {
                        Action::SetStatus(Some("Username and display name are required".to_string()))
                    } else {
                        Action::Navigate(Screen::CustomUserType)
                    }
                }
                _ => Action::Noop,
            }
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Custom User ")
            .border_style(Style::default().fg(Color::Cyan));
        let inner = block.inner(rect);
        f.render_widget(block, rect);

        let lines = vec![
            Line::from("Fill in the basic fields for a custom user."),
            Line::default(),
            text_field_line(
                self.active_field == 0,
                self.editing && self.active_field == 0,
                "username",
                &self.username,
            ),
            text_field_line(
                self.active_field == 1,
                self.editing && self.active_field == 1,
                "display name",
                &self.display_name,
            ),
            Line::default(),
            status_line(self.status_message.as_deref()),
        ];

        f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
    }
}

impl CustomBasicPage {
    fn current_field_mut(&mut self) -> &mut String {
        match self.active_field {
            0 => &mut self.username,
            _ => &mut self.display_name,
        }
    }

    fn current_change(&self) -> PendingUserChange {
        match self.active_field {
            0 => PendingUserChange::Username(self.username.clone()),
            _ => PendingUserChange::DisplayName(self.display_name.clone()),
        }
    }
}

impl InstallerPage for CustomTypePage {
    fn screen(&self) -> Screen {
        Screen::CustomUserType
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.status_message = snapshot.status_message.clone();
        self.selected = snapshot
            .pending_user
            .as_ref()
            .map(|user| match user.user_type {
                UserType::Gui => 0,
                UserType::Tui => 1,
                UserType::Cui => 2,
            })
            .unwrap_or(0);
    }
}

impl Component for CustomTypePage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Up => self.move_selection(-1),
            KeyCode::Down | KeyCode::Tab => self.move_selection(1),
            KeyCode::Left => Action::Navigate(Screen::CustomUserBasic),
            KeyCode::Right | KeyCode::Enter => Action::Navigate(Screen::CustomUserPrograms),
            _ => Action::Noop,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let options = ["gui", "tui", "cui"];
        render_options_page(
            f,
            rect,
            " User Type ",
            "Select the default experience for this user.",
            &options,
            self.selected,
            self.status_message.as_deref(),
        );
    }
}

impl CustomTypePage {
    fn move_selection(&mut self, delta: isize) -> Action {
        self.selected = (self.selected as isize + delta).rem_euclid(3) as usize;
        Action::PendingUserChanged(PendingUserChange::UserType(match self.selected {
            0 => UserType::Gui,
            1 => UserType::Tui,
            _ => UserType::Cui,
        }))
    }
}

impl InstallerPage for CustomProgramsPage {
    fn screen(&self) -> Screen {
        Screen::CustomUserPrograms
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.status_message = snapshot.status_message.clone();
        self.user_type = snapshot.pending_user.as_ref().map(|user| user.user_type);
        self.selected_programs = snapshot
            .pending_user
            .as_ref()
            .map(|user| user.programs.clone())
            .unwrap_or_default();
        let option_count = program_options_for(self.user_type.unwrap_or(UserType::Gui)).len();
        if option_count > 0 && self.selected >= option_count {
            self.selected = option_count - 1;
        }
    }
}

impl Component for CustomProgramsPage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        let options = program_options_for(self.user_type.unwrap_or(UserType::Gui));
        if options.is_empty() {
            return Action::Noop;
        }

        match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
                Action::Noop
            }
            KeyCode::Down | KeyCode::Tab => {
                self.selected = (self.selected + 1).min(options.len() - 1);
                Action::Noop
            }
            KeyCode::Char(' ') => {
                Action::PendingUserChanged(PendingUserChange::ToggleProgram(
                    options[self.selected].to_string(),
                ))
            }
            KeyCode::Left => Action::Navigate(Screen::CustomUserType),
            KeyCode::Right | KeyCode::Enter => Action::Navigate(Screen::CustomUserPassword),
            _ => Action::Noop,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Programs ")
            .border_style(Style::default().fg(Color::Cyan));
        let inner = block.inner(rect);
        f.render_widget(block, rect);

        let options = program_options_for(self.user_type.unwrap_or(UserType::Gui));
        let mut lines = vec![
            Line::from("Toggle any program groups you want for the custom user."),
            Line::default(),
        ];
        for (index, option) in options.iter().enumerate() {
            let enabled = self.selected_programs.iter().any(|item| item == option);
            lines.push(Line::from(vec![
                Span::styled(
                    if index == self.selected { "> " } else { "  " },
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw(if enabled { "[x] " } else { "[ ] " }),
                Span::styled(
                    option.to_string(),
                    if index == self.selected {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    },
                ),
            ]));
        }
        lines.push(Line::default());
        lines.push(Line::from("Space toggles the highlighted program group."));
        lines.push(status_line(self.status_message.as_deref()));

        f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
    }
}

impl InstallerPage for CustomPasswordPage {
    fn screen(&self) -> Screen {
        Screen::CustomUserPassword
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.status_message = snapshot.status_message.clone();
        if let Some(user) = snapshot.pending_user.as_ref() {
            self.password = user.password.clone();
            self.password_confirm = user.password_confirm.clone();
        }
    }
}

impl Component for CustomPasswordPage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        password_page_key(
            key,
            &mut self.active_field,
            &mut self.editing,
            &mut self.password,
            &mut self.password_confirm,
            Screen::CustomUserPrograms,
        )
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        render_password_page(
            f,
            rect,
            " Custom Password ",
            &self.password,
            &self.password_confirm,
            self.active_field,
            self.editing,
            self.status_message.as_deref(),
        );
    }
}

impl InstallerPage for UserAddResultPage {
    fn screen(&self) -> Screen {
        Screen::UserAddResult
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.last_user = snapshot.pending_user.clone();
        self.status_message = snapshot.status_message.clone();
    }
}

impl Component for UserAddResultPage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Left | KeyCode::Right | KeyCode::Enter => Action::Batch(vec![
                Action::ResetPendingUser,
                Action::Navigate(Screen::UserMenu),
            ]),
            _ => Action::Noop,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" User Added ")
            .border_style(Style::default().fg(Color::Green));
        let inner = block.inner(rect);
        f.render_widget(block, rect);

        let user = self.last_user.as_ref();
        let lines = vec![
            Line::from("The pending user has been committed to the install config."),
            Line::default(),
            Line::from(format!(
                "username    : {}",
                user.map(|value| value.username.as_str()).unwrap_or("<none>")
            )),
            Line::from(format!(
                "display name: {}",
                user.map(|value| value.display_name.as_str()).unwrap_or("<none>")
            )),
            Line::from(format!(
                "type        : {}",
                user.map(|value| value.user_type.label()).unwrap_or("<none>")
            )),
            Line::default(),
            status_line(self.status_message.as_deref()),
        ];

        f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
    }
}

fn password_page_key(
    key: KeyEvent,
    active_field: &mut usize,
    editing: &mut bool,
    password: &mut String,
    password_confirm: &mut String,
    previous: Screen,
) -> Action {
    if *editing {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                *editing = false;
                Action::Noop
            }
            KeyCode::Backspace => {
                current_password_field(*active_field, password, password_confirm).pop();
                password_change(*active_field, password, password_confirm)
            }
            KeyCode::Char(c) => {
                current_password_field(*active_field, password, password_confirm).push(c);
                password_change(*active_field, password, password_confirm)
            }
            _ => Action::Noop,
        }
    } else {
        match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Up => {
                *active_field = active_field.saturating_sub(1);
                Action::Noop
            }
            KeyCode::Down | KeyCode::Tab => {
                *active_field = (*active_field + 1).min(1);
                Action::Noop
            }
            KeyCode::Enter => {
                *editing = true;
                Action::Noop
            }
            KeyCode::Left => Action::Navigate(previous),
            KeyCode::Right => Action::CommitPendingUser,
            _ => Action::Noop,
        }
    }
}

fn current_password_field<'a>(
    active_field: usize,
    password: &'a mut String,
    password_confirm: &'a mut String,
) -> &'a mut String {
    match active_field {
        0 => password,
        _ => password_confirm,
    }
}

fn password_change(
    active_field: usize,
    password: &String,
    password_confirm: &String,
) -> Action {
    match active_field {
        0 => Action::PendingUserChanged(PendingUserChange::Password(password.clone())),
        _ => Action::PendingUserChanged(PendingUserChange::PasswordConfirm(
            password_confirm.clone(),
        )),
    }
}

fn render_password_page(
    f: &mut Frame,
    rect: Rect,
    title: &str,
    password: &str,
    password_confirm: &str,
    active_field: usize,
    editing: bool,
    status_message: Option<&str>,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(rect);
    f.render_widget(block, rect);

    let lines = vec![
        Line::from("Set the password for the pending user."),
        Line::default(),
        text_field_line(active_field == 0, editing && active_field == 0, "password", password),
        text_field_line(
            active_field == 1,
            editing && active_field == 1,
            "confirm",
            password_confirm,
        ),
        Line::default(),
        Line::from("Right commits the pending user into the install config."),
        status_line(status_message),
    ];

    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn program_options_for(user_type: UserType) -> &'static [&'static str] {
    match user_type {
        UserType::Gui => &["browser", "office", "media", "sns", "programming", "gaming"],
        UserType::Tui => &["cli-tools", "git", "programming", "media"],
        UserType::Cui => &["base", "monitoring", "networking"],
    }
}

fn render_options_page(
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
        .border_style(Style::default().fg(Color::Cyan));
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
    lines.push(Line::default());
    lines.push(status_line(status_message));

    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

fn text_field_line(active: bool, editing: bool, label: &str, value: &str) -> Line<'static> {
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