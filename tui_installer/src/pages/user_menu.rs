use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
};

use crate::{
    action::{Action, PendingUserChange},
    app::{AppSnapshot, PendingUser, Screen},
    component::Component,
    components::{
        Popup,
        form::{FormFieldRole, FormSection, render_form_section},
    },
    config::UserType,
    pages::{InstallerPage, form_field, status_field},
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
    username: String,
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
    username: String,
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
            KeyCode::Left => Action::Navigate(Screen::StorageToggle),
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
        let mut fields = USER_MENU_OPTIONS
            .iter()
            .enumerate()
            .map(|(index, option)| {
                form_field(
                    *option,
                    "default config",
                    Some(user_menu_hint(index).to_string()),
                    FormFieldRole::Choice,
                )
            })
            .collect::<Vec<_>>();
        fields.push(status_field(self.status_message.as_deref()));

        let section = FormSection::new("users", fields, Some(self.selected), false);
        render_form_section(f, rect, &section);
    }
}

impl InstallerPage for PresetPasswordPage {
    fn screen(&self) -> Screen {
        Screen::PresetUserPassword
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.status_message = snapshot.status_message.clone();
        if let Some(user) = snapshot.pending_user.as_ref() {
            self.username = user.username.clone();
            self.password = user.password.clone();
            self.password_confirm = user.password_confirm.clone();
        }
    }

    fn popup(&self) -> Option<Popup> {
        Some(password_popup(
            "Preset Password",
            "preset password",
            &self.username,
            &self.password,
            &self.password_confirm,
            self.active_field,
            self.editing,
        ))
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
        let section = FormSection::new(
            "preset password",
            vec![
                status_field(self.status_message.as_deref()),
                form_field(
                    "flow",
                    "preset user password",
                    Some("Password input is shown in the popup".to_string()),
                    FormFieldRole::ReadOnly,
                ),
            ],
            None,
            false,
        );
        render_form_section(f, rect, &section);
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
                        Action::SetStatus(Some(
                            "Username and display name are required".to_string(),
                        ))
                    } else {
                        Action::Navigate(Screen::CustomUserType)
                    }
                }
                _ => Action::Noop,
            }
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let section = FormSection::new(
            "custom user",
            vec![
                form_field(
                    "username",
                    self.username.clone(),
                    Some("Linux account name".to_string()),
                    FormFieldRole::Text,
                ),
                form_field(
                    "display name",
                    self.display_name.clone(),
                    Some("Human-readable user name".to_string()),
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
        let fields = vec![
            form_field(
                "user type",
                user_type_choice_value(self.selected),
                Some("Select the default experience for this user".to_string()),
                FormFieldRole::Choice,
            ),
            status_field(self.status_message.as_deref()),
        ];

        let section = FormSection::new("user type", fields, Some(0), false);
        render_form_section(f, rect, &section);
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
            KeyCode::Char(' ') => Action::PendingUserChanged(PendingUserChange::ToggleProgram(
                options[self.selected].to_string(),
            )),
            KeyCode::Left => Action::Navigate(Screen::CustomUserType),
            KeyCode::Right | KeyCode::Enter => Action::Navigate(Screen::CustomUserPassword),
            _ => Action::Noop,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let options = program_options_for(self.user_type.unwrap_or(UserType::Gui));
        let mut fields = options
            .iter()
            .map(|option| {
                let enabled = self.selected_programs.iter().any(|item| item == option);
                form_field(
                    *option,
                    enabled.to_string(),
                    Some("Space toggles the highlighted program group".to_string()),
                    FormFieldRole::Toggle,
                )
            })
            .collect::<Vec<_>>();
        fields.push(status_field(self.status_message.as_deref()));

        let section = FormSection::new("programs", fields, Some(self.selected), false);
        render_form_section(f, rect, &section);
    }
}

impl InstallerPage for CustomPasswordPage {
    fn screen(&self) -> Screen {
        Screen::CustomUserPassword
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.status_message = snapshot.status_message.clone();
        if let Some(user) = snapshot.pending_user.as_ref() {
            self.username = user.username.clone();
            self.password = user.password.clone();
            self.password_confirm = user.password_confirm.clone();
        }
    }

    fn popup(&self) -> Option<Popup> {
        Some(password_popup(
            "Custom Password",
            "custom password",
            &self.username,
            &self.password,
            &self.password_confirm,
            self.active_field,
            self.editing,
        ))
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
        let section = FormSection::new(
            "custom password",
            vec![
                status_field(self.status_message.as_deref()),
                form_field(
                    "flow",
                    "custom user password",
                    Some("Password input is shown in the popup".to_string()),
                    FormFieldRole::ReadOnly,
                ),
            ],
            None,
            false,
        );
        render_form_section(f, rect, &section);
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
        let user = self.last_user.as_ref();
        let section = FormSection::new(
            "user added",
            vec![
                form_field(
                    "username",
                    user.map(|value| value.username.as_str())
                        .unwrap_or("<none>"),
                    None,
                    FormFieldRole::ReadOnly,
                ),
                form_field(
                    "display name",
                    user.map(|value| value.display_name.as_str())
                        .unwrap_or("<none>"),
                    None,
                    FormFieldRole::ReadOnly,
                ),
                form_field(
                    "type",
                    user.map(|value| value.user_type.label())
                        .unwrap_or("<none>"),
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

fn password_change(active_field: usize, password: &str, password_confirm: &str) -> Action {
    match active_field {
        0 => Action::PendingUserChanged(PendingUserChange::Password(password.to_owned())),
        _ => Action::PendingUserChanged(PendingUserChange::PasswordConfirm(
            password_confirm.to_owned(),
        )),
    }
}

fn password_popup(
    title: &str,
    section_title: &str,
    username: &str,
    password: &str,
    password_confirm: &str,
    active_field: usize,
    editing: bool,
) -> Popup {
    Popup::new(
        title,
        72,
        54,
        FormSection::new(
            section_title,
            vec![
                form_field(
                    "password",
                    password,
                    Some(format!(
                        "Password for {}",
                        if username.is_empty() {
                            "<pending>"
                        } else {
                            username
                        }
                    )),
                    FormFieldRole::Text,
                ),
                form_field(
                    "confirm",
                    password_confirm,
                    Some("Must match password".to_string()),
                    FormFieldRole::Text,
                ),
            ],
            Some(active_field),
            editing,
        ),
    )
}

fn user_menu_hint(index: usize) -> &'static str {
    match index {
        0 => "TUI core user; preset TUI user; password required",
        1 => "KDE Plasma office user; preset GUI user; password required",
        2 => "KDE Plasma gaming user; preset GUI user; password required",
        3 => "Development GUI user; programming tools; password required",
        4 => "Full GUI user; all program groups; password required",
        5 => "Create a custom user from scratch",
        _ => "Finish user setup and review install summary",
    }
}

fn user_type_choice_value(selected: usize) -> String {
    ["gui", "tui", "cui"]
        .iter()
        .enumerate()
        .map(|(index, option)| {
            if index == selected {
                format!("[ {option} ]")
            } else {
                (*option).to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" / ")
}

fn program_options_for(user_type: UserType) -> &'static [&'static str] {
    match user_type {
        UserType::Gui => &[
            "browser",
            "office",
            "media",
            "sns",
            "programming",
            "gaming",
            "electronics",
        ],
        UserType::Tui => &["cli-tools", "git", "programming", "media"],
        UserType::Cui => &["base", "monitoring", "networking"],
    }
}
