use crossterm::event::{KeyCode, KeyEvent};
use std::collections::HashMap;

use crate::config::{InstallConfig, UserConfig, UserType};
use crate::infra::install::run_phase3_install;
use crate::infra::password_hasher::hash_password;
use crate::logic::setup::{
    CPU_OPTIONS, GPU_OPTIONS, HardwareInfo, KEYBOARD_OPTIONS, LOCALE_OPTIONS, TIMEZONE_OPTIONS,
    apply_detected_config, collect_hardware,
};

pub const GUI_PROGRAM_OPTIONS: &[(&str, &str)] = &[
    ("browser", "Chromium web browser"),
    ("gaming", "Steam + Lutris + Wine"),
    ("media", "Spotify + mpv"),
    ("sns", "Discord"),
    ("kicad", "KiCad"),
    ("freecad", "FreeCAD + MeshLab"),
    ("zed", "Zed editor"),
];

pub const DEV_PROGRAM_OPTIONS: &[(&str, &str)] = &[
    ("programming", "Shell setup"),
    ("lang", "Language toolchains"),
    ("nix-tools", "Nix ecosystem"),
    ("cli-tools", "General CLI tools"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Screen {
    Welcome,
    HardwareDetect,
    DeviceSelect,
    PartitionConfig,
    HostnameInput,
    KeyboardSelect,
    LocaleSelect,
    TimezoneSelect,
    SshToggle,
    StorageToggle,
    GpuSelect,
    CpuSelect,
    UserMenu,
    PresetUserPassword,
    CustomUserBasic,
    CustomUserType,
    CustomUserPrograms,
    CustomUserPassword,
    UserAddResult,
    Summary,
    Installing,
    Done,
}

impl Screen {
    pub const ORDER: [Self; 22] = [
        Self::Welcome,
        Self::HardwareDetect,
        Self::DeviceSelect,
        Self::PartitionConfig,
        Self::HostnameInput,
        Self::KeyboardSelect,
        Self::LocaleSelect,
        Self::TimezoneSelect,
        Self::SshToggle,
        Self::StorageToggle,
        Self::GpuSelect,
        Self::CpuSelect,
        Self::UserMenu,
        Self::PresetUserPassword,
        Self::CustomUserBasic,
        Self::CustomUserType,
        Self::CustomUserPrograms,
        Self::CustomUserPassword,
        Self::UserAddResult,
        Self::Summary,
        Self::Installing,
        Self::Done,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Welcome => "Welcome",
            Self::HardwareDetect => "Hardware Detect",
            Self::DeviceSelect => "Device Select",
            Self::PartitionConfig => "Partition Config",
            Self::HostnameInput => "Hostname",
            Self::KeyboardSelect => "Keyboard",
            Self::LocaleSelect => "Locale",
            Self::TimezoneSelect => "Timezone",
            Self::SshToggle => "SSH",
            Self::StorageToggle => "Storage",
            Self::GpuSelect => "GPU",
            Self::CpuSelect => "CPU",
            Self::UserMenu => "Users",
            Self::PresetUserPassword => "Preset Password",
            Self::CustomUserBasic => "Custom User",
            Self::CustomUserType => "User Type",
            Self::CustomUserPrograms => "Programs",
            Self::CustomUserPassword => "Custom Password",
            Self::UserAddResult => "User Added",
            Self::Summary => "Summary",
            Self::Installing => "Installing",
            Self::Done => "Done",
        }
    }

    pub fn phase(self) -> &'static str {
        match self {
            Self::Welcome
            | Self::HardwareDetect
            | Self::DeviceSelect
            | Self::PartitionConfig
            | Self::HostnameInput
            | Self::KeyboardSelect
            | Self::LocaleSelect
            | Self::TimezoneSelect
            | Self::SshToggle
            | Self::StorageToggle
            | Self::GpuSelect
            | Self::CpuSelect => "PC Config",
            Self::UserMenu
            | Self::PresetUserPassword
            | Self::CustomUserBasic
            | Self::CustomUserType
            | Self::CustomUserPrograms
            | Self::CustomUserPassword
            | Self::UserAddResult => "Users",
            Self::Summary | Self::Installing | Self::Done => "Install",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserMenuChoice {
    Jade,
    Admin,
    Custom,
    Finish,
}

impl UserMenuChoice {
    fn from_index(index: usize) -> Self {
        match index {
            0 => Self::Jade,
            1 => Self::Admin,
            2 => Self::Custom,
            _ => Self::Finish,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PendingUser {
    pub username: String,
    pub display_name: String,
    pub user_type: UserType,
    pub programs: Vec<String>,
    pub password: String,
    pub password_confirm: String,
    pub is_preset: bool,
}

impl PendingUser {
    fn preset(username: &str, display_name: &str, user_type: UserType, programs: &[&str]) -> Self {
        Self {
            username: username.to_string(),
            display_name: display_name.to_string(),
            user_type,
            programs: programs.iter().map(|program| program.to_string()).collect(),
            password: String::new(),
            password_confirm: String::new(),
            is_preset: true,
        }
    }

    fn custom() -> Self {
        Self {
            username: String::new(),
            display_name: String::new(),
            user_type: UserType::Gui,
            programs: vec!["desktop".to_string()],
            password: String::new(),
            password_confirm: String::new(),
            is_preset: false,
        }
    }
}

#[derive(Debug)]
pub struct App {
    pub screen: Screen,
    pub config: InstallConfig,
    pub hardware: HardwareInfo,
    pub install_log: Vec<String>,
    pub should_quit: bool,
    pub input_state: HashMap<Screen, ScreenInputState>,
    pub pending_user: Option<PendingUser>,
    pub user_message: Option<String>,
    pub install_confirmation: String,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ScreenInputState {
    pub active_field: usize,
    pub custom_selected: bool,
}

impl Default for App {
    fn default() -> Self {
        let mut input_state = HashMap::new();
        for screen in Screen::ORDER {
            input_state.insert(screen, ScreenInputState::default());
        }

        let mut config = InstallConfig::default();
        let hardware = collect_hardware();
        apply_detected_config(&mut config, &hardware);

        Self {
            screen: Screen::Welcome,
            config,
            hardware,
            install_log: vec!["installer bootstrap ready".to_string()],
            should_quit: false,
            input_state,
            pending_user: None,
            user_message: None,
            install_confirmation: String::new(),
        }
    }
}

impl App {
    pub fn next_screen(&mut self) {
        let current = Self::screen_index(self.screen);
        if let Some(next) = Screen::ORDER.get(current + 1).copied() {
            self.screen = next;
        }
    }

    pub fn previous_screen(&mut self) {
        let current = Self::screen_index(self.screen);
        if current > 0 {
            self.screen = Screen::ORDER[current - 1];
        }
    }

    pub fn active_field_for_current_screen(&self) -> usize {
        self.input_state
            .get(&self.screen)
            .map(|state| {
                let max_index = self.editable_field_count(self.screen).saturating_sub(1);
                state.active_field.min(max_index)
            })
            .unwrap_or(0)
    }

    pub fn custom_selected_for_current_screen(&self) -> bool {
        self.input_state
            .get(&self.screen)
            .map(|state| state.custom_selected)
            .unwrap_or(false)
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') if !self.active_field_accepts_text() => self.should_quit = true,
            KeyCode::Right | KeyCode::Enter => self.confirm_current_screen(),
            KeyCode::Left => self.previous_screen_for_current_flow(),
            KeyCode::Up => self.move_active_field(-1),
            KeyCode::Down | KeyCode::Tab => self.move_active_field(1),
            KeyCode::Backspace => self.backspace_active_field(),
            KeyCode::Char(' ') => {
                if self.active_field_accepts_text() {
                    self.insert_active_field_char(' ');
                } else {
                    self.toggle_active_field();
                }
            }
            KeyCode::Char(c) => self.insert_active_field_char(c),
            KeyCode::Esc => {
                if self.screen == Screen::Welcome {
                    self.should_quit = true;
                } else {
                    self.previous_screen_for_current_flow();
                }
            }
            _ => {}
        }
    }

    fn confirm_current_screen(&mut self) {
        self.user_message = None;
        match self.screen {
            Screen::UserMenu => self.confirm_user_menu(),
            Screen::PresetUserPassword | Screen::CustomUserPassword => self.confirm_user_password(),
            Screen::CustomUserBasic => self.confirm_custom_basic(),
            Screen::CustomUserType => {
                self.sync_custom_user_type();
                self.screen = Screen::CustomUserPrograms;
            }
            Screen::CustomUserPrograms => self.screen = Screen::CustomUserPassword,
            Screen::UserAddResult => self.screen = Screen::UserMenu,
            Screen::Summary => self.confirm_installation(),
            _ => self.next_screen(),
        }
    }

    fn previous_screen_for_current_flow(&mut self) {
        match self.screen {
            Screen::PresetUserPassword
            | Screen::CustomUserBasic
            | Screen::UserAddResult
            | Screen::Summary => self.screen = Screen::UserMenu,
            Screen::Done => self.screen = Screen::Summary,
            Screen::CustomUserType => self.screen = Screen::CustomUserBasic,
            Screen::CustomUserPrograms => self.screen = Screen::CustomUserType,
            Screen::CustomUserPassword => self.screen = Screen::CustomUserPrograms,
            _ => self.previous_screen(),
        }
    }

    fn move_active_field(&mut self, direction: isize) {
        let count = self.editable_field_count(self.screen);
        if count <= 1 {
            return;
        }

        let current = self.active_field_for_current_screen() as isize;
        let mut next = (current + direction).rem_euclid(count as isize) as usize;

        if self.requires_custom_mode(self.screen, next)
            && !self.custom_selected_for_current_screen()
        {
            next = 0;
        }

        if let Some(state) = self.input_state.get_mut(&self.screen) {
            state.active_field = next;
        }

        if self.screen == Screen::CustomUserType {
            self.sync_custom_user_type();
        }
    }

    fn insert_active_field_char(&mut self, c: char) {
        if c.is_control() {
            return;
        }

        let screen = self.screen;
        let active = self.active_field_for_current_screen();
        if self.requires_custom_mode(screen, active) && !self.custom_selected_for_current_screen() {
            return;
        }
        if let Some(text) = self.text_field_mut(screen, active) {
            text.push(c);
        }
    }

    fn backspace_active_field(&mut self) {
        let screen = self.screen;
        let active = self.active_field_for_current_screen();
        if self.requires_custom_mode(screen, active) && !self.custom_selected_for_current_screen() {
            return;
        }
        if let Some(text) = self.text_field_mut(screen, active) {
            text.pop();
        }
    }

    fn toggle_active_field(&mut self) {
        let active = self.active_field_for_current_screen();
        match (self.screen, active) {
            (Screen::KeyboardSelect, 0) => self.toggle_preset_or_custom(KEYBOARD_OPTIONS),
            (Screen::LocaleSelect, 0) => self.toggle_preset_or_custom(LOCALE_OPTIONS),
            (Screen::TimezoneSelect, 0) => self.toggle_preset_or_custom(TIMEZONE_OPTIONS),
            (Screen::GpuSelect, 0) => self.toggle_preset_or_custom(GPU_OPTIONS),
            (Screen::CpuSelect, 0) => self.toggle_preset_or_custom(CPU_OPTIONS),
            (Screen::SshToggle, 0) => self.config.ssh_enabled = !self.config.ssh_enabled,
            (Screen::StorageToggle, 0) => {
                self.config.storage_enabled = !self.config.storage_enabled
            }
            (Screen::UserMenu, _) => self.move_active_field(1),
            (Screen::CustomUserType, _) => {
                self.move_active_field(1);
                self.sync_custom_user_type();
            }
            (Screen::CustomUserPrograms, _) => self.toggle_program(active),
            _ => {}
        }
    }

    fn editable_field_count(&self, screen: Screen) -> usize {
        match screen {
            Screen::DeviceSelect => 1,
            Screen::PartitionConfig => 2,
            Screen::HostnameInput => 1,
            Screen::KeyboardSelect => 2,
            Screen::LocaleSelect => 2,
            Screen::TimezoneSelect => 2,
            Screen::SshToggle => 1,
            Screen::StorageToggle => 1,
            Screen::GpuSelect => 2,
            Screen::CpuSelect => 2,
            Screen::UserMenu => 4,
            Screen::PresetUserPassword => 2,
            Screen::CustomUserBasic => 2,
            Screen::CustomUserType => 2,
            Screen::CustomUserPrograms => self.program_option_count(),
            Screen::CustomUserPassword => 2,
            Screen::Summary => 3,
            _ => 0,
        }
    }

    fn text_field_mut(&mut self, screen: Screen, active_field: usize) -> Option<&mut String> {
        match (screen, active_field) {
            (Screen::DeviceSelect, 0) => Some(&mut self.config.device),
            (Screen::PartitionConfig, 0) => Some(&mut self.config.boot_end),
            (Screen::PartitionConfig, 1) => Some(&mut self.config.root_end),
            (Screen::HostnameInput, 0) => Some(&mut self.config.hostname),
            (Screen::KeyboardSelect, 1) => Some(&mut self.config.keyboard),
            (Screen::LocaleSelect, 1) => Some(&mut self.config.locale),
            (Screen::TimezoneSelect, 1) => Some(&mut self.config.timezone),
            (Screen::GpuSelect, 1) => Some(&mut self.config.gpu_custom),
            (Screen::CpuSelect, 1) => Some(&mut self.config.cpu_custom),
            (Screen::PresetUserPassword, 0) | (Screen::CustomUserPassword, 0) => {
                self.pending_user.as_mut().map(|user| &mut user.password)
            }
            (Screen::PresetUserPassword, 1) | (Screen::CustomUserPassword, 1) => self
                .pending_user
                .as_mut()
                .map(|user| &mut user.password_confirm),
            (Screen::CustomUserBasic, 0) => {
                self.pending_user.as_mut().map(|user| &mut user.username)
            }
            (Screen::CustomUserBasic, 1) => self
                .pending_user
                .as_mut()
                .map(|user| &mut user.display_name),
            (Screen::Summary, 2) => Some(&mut self.install_confirmation),
            _ => None,
        }
    }

    fn active_field_accepts_text(&self) -> bool {
        matches!(
            (self.screen, self.active_field_for_current_screen()),
            (Screen::DeviceSelect, 0)
                | (Screen::PartitionConfig, 0)
                | (Screen::PartitionConfig, 1)
                | (Screen::HostnameInput, 0)
                | (Screen::KeyboardSelect, 1)
                | (Screen::LocaleSelect, 1)
                | (Screen::TimezoneSelect, 1)
                | (Screen::GpuSelect, 1)
                | (Screen::CpuSelect, 1)
                | (Screen::PresetUserPassword, 0)
                | (Screen::PresetUserPassword, 1)
                | (Screen::CustomUserBasic, 0)
                | (Screen::CustomUserBasic, 1)
                | (Screen::CustomUserPassword, 0)
                | (Screen::CustomUserPassword, 1)
                | (Screen::Summary, 2)
        )
    }

    fn toggle_preset_or_custom(&mut self, values: &[&str]) {
        let is_custom = self.custom_selected_for_current_screen();
        if is_custom {
            self.set_custom_selected(false);
            self.apply_preset_for_current_screen(values.first().copied().unwrap_or_default());
            return;
        }

        let current = self.current_text_value_for_screen();
        if let Some(index) = values.iter().position(|candidate| *candidate == current) {
            if index + 1 < values.len() {
                self.apply_preset_for_current_screen(values[index + 1]);
            } else {
                self.set_custom_selected(true);
                if let Some(state) = self.input_state.get_mut(&self.screen) {
                    state.active_field = 1;
                }
            }
        } else if let Some(first) = values.first() {
            self.apply_preset_for_current_screen(first);
        }
    }

    fn set_custom_selected(&mut self, selected: bool) {
        if let Some(state) = self.input_state.get_mut(&self.screen) {
            state.custom_selected = selected;
            if !selected && state.active_field > 0 {
                state.active_field = 0;
            }
        }
    }

    fn requires_custom_mode(&self, screen: Screen, active_field: usize) -> bool {
        matches!(
            (screen, active_field),
            (Screen::KeyboardSelect, 1)
                | (Screen::LocaleSelect, 1)
                | (Screen::TimezoneSelect, 1)
                | (Screen::GpuSelect, 1)
                | (Screen::CpuSelect, 1)
        )
    }

    fn current_text_value_for_screen(&self) -> &str {
        match self.screen {
            Screen::KeyboardSelect => &self.config.keyboard,
            Screen::LocaleSelect => &self.config.locale,
            Screen::TimezoneSelect => &self.config.timezone,
            Screen::GpuSelect => self.config.gpu_type.label(),
            Screen::CpuSelect => self.config.cpu_type.label(),
            _ => "",
        }
    }

    fn apply_preset_for_current_screen(&mut self, value: &str) {
        if value.is_empty() {
            return;
        }

        match self.screen {
            Screen::KeyboardSelect => self.config.keyboard = value.to_string(),
            Screen::LocaleSelect => self.config.locale = value.to_string(),
            Screen::TimezoneSelect => self.config.timezone = value.to_string(),
            Screen::GpuSelect => {
                self.config.gpu_type = match value {
                    "none" => crate::config::GpuType::None,
                    "nvidia" => crate::config::GpuType::Nvidia,
                    "amd" => crate::config::GpuType::Amd,
                    "intel" => crate::config::GpuType::Intel,
                    _ => self.config.gpu_type,
                };
            }
            Screen::CpuSelect => {
                self.config.cpu_type = match value {
                    "amd" => crate::config::CpuType::Amd,
                    "intel" => crate::config::CpuType::Intel,
                    "aarch64" => crate::config::CpuType::Aarch64,
                    _ => self.config.cpu_type,
                };
            }
            _ => {}
        }
        self.set_custom_selected(false);
    }

    fn confirm_user_menu(&mut self) {
        match UserMenuChoice::from_index(self.active_field_for_current_screen()) {
            UserMenuChoice::Jade => self.start_preset_user("jade"),
            UserMenuChoice::Admin => self.start_preset_user("admin"),
            UserMenuChoice::Custom => {
                self.pending_user = Some(PendingUser::custom());
                self.set_active_field(Screen::CustomUserBasic, 0);
                self.screen = Screen::CustomUserBasic;
            }
            UserMenuChoice::Finish => {
                if self.config.users.is_empty() {
                    self.user_message = Some("Please add at least one user.".to_string());
                } else {
                    self.user_message = None;
                    self.screen = Screen::Summary;
                }
            }
        }
    }

    fn start_preset_user(&mut self, username: &str) {
        if self.user_exists(username) {
            self.user_message = Some(format!("{username} is already added."));
            return;
        }

        self.pending_user = match username {
            "jade" => Some(PendingUser::preset(
                "jade",
                "Jade",
                UserType::Gui,
                &["desktop"],
            )),
            "admin" => Some(PendingUser::preset(
                "admin",
                "Administrator",
                UserType::Cui,
                &[],
            )),
            _ => None,
        };
        self.set_active_field(Screen::PresetUserPassword, 0);
        self.screen = Screen::PresetUserPassword;
    }

    fn confirm_custom_basic(&mut self) {
        let Some(user) = self.pending_user.as_ref() else {
            self.pending_user = Some(PendingUser::custom());
            return;
        };

        let username = user.username.trim().to_string();
        let display_name = user.display_name.trim().to_string();
        if username.is_empty() {
            self.user_message = Some("Username cannot be empty.".to_string());
            return;
        }
        if !is_valid_username(&username) {
            self.user_message = Some(
                "Use lowercase letters, digits, '_' or '-' and start with a letter or '_'."
                    .to_string(),
            );
            return;
        }
        if self.is_reserved_username(&username) || self.user_exists(&username) {
            self.user_message = Some(format!("{username} is reserved or already added."));
            return;
        }

        let Some(user) = self.pending_user.as_mut() else {
            return;
        };
        user.username = username;
        if display_name.is_empty() {
            user.display_name = default_display_name(&user.username);
        } else {
            user.display_name = display_name;
        }
        self.set_active_field(Screen::CustomUserType, 0);
        self.sync_custom_user_type();
        self.screen = Screen::CustomUserType;
    }

    fn confirm_user_password(&mut self) {
        let Some(user) = self.pending_user.as_mut() else {
            self.user_message = Some("No pending user to add.".to_string());
            self.screen = Screen::UserMenu;
            return;
        };

        if user.password.is_empty() {
            self.user_message = Some("Password cannot be empty.".to_string());
            return;
        }
        if user.password != user.password_confirm {
            self.user_message = Some("Passwords do not match.".to_string());
            return;
        }

        let hash = match hash_password(&user.password) {
            Ok(hash) => hash,
            Err(error) => {
                self.user_message = Some(format!("Password hash failed: {error}"));
                return;
            }
        };

        let user = self.pending_user.take().expect("pending user exists");
        self.config.users.push(UserConfig {
            username: user.username,
            display_name: user.display_name,
            user_type: user.user_type,
            programs: normalize_programs(user.user_type, user.programs),
            password_hash: hash,
            is_preset: user.is_preset,
        });
        self.user_message = Some("User added. Press Enter to return to user menu.".to_string());
        self.screen = Screen::UserAddResult;
    }

    fn sync_custom_user_type(&mut self) {
        let active = self.active_field_for_current_screen();
        if let Some(user) = self.pending_user.as_mut() {
            user.user_type = if active == 0 {
                UserType::Gui
            } else {
                UserType::Cui
            };
            user.programs = normalize_programs(user.user_type, user.programs.clone());
        }
    }

    fn toggle_program(&mut self, active: usize) {
        let Some(user) = self.pending_user.as_mut() else {
            return;
        };
        let options = program_options_for(user.user_type);
        let Some(program) = options.get(active).map(|(name, _)| *name) else {
            return;
        };
        if program == "desktop" {
            return;
        }
        if user.programs.iter().any(|candidate| candidate == program) {
            user.programs.retain(|candidate| candidate != program);
        } else {
            user.programs.push(program.to_string());
        }
        user.programs = normalize_programs(user.user_type, user.programs.clone());
    }

    fn program_option_count(&self) -> usize {
        self.pending_user
            .as_ref()
            .map(|user| program_options_for(user.user_type).len())
            .unwrap_or(0)
    }

    fn set_active_field(&mut self, screen: Screen, active_field: usize) {
        if let Some(state) = self.input_state.get_mut(&screen) {
            state.active_field = active_field;
        }
    }

    pub fn user_exists(&self, username: &str) -> bool {
        self.config
            .users
            .iter()
            .any(|user| user.username == username)
    }

    fn is_reserved_username(&self, username: &str) -> bool {
        matches!(username, "greeter" | "jade" | "admin")
    }

    fn confirm_installation(&mut self) {
        if self.install_confirmation.trim() != "yes" {
            self.user_message =
                Some("Type 'yes' in confirm field before starting install.".to_string());
            return;
        }
        self.screen = Screen::Installing;
        self.install_log.clear();
        match run_phase3_install(&self.config, &self.config.users, &mut self.install_log) {
            Ok(()) => {
                self.user_message = Some("Installation complete. Review logs below.".to_string());
                self.screen = Screen::Done;
            }
            Err(error) => {
                self.install_log.push(format!("install failed: {error}"));
                self.user_message = Some(format!("Installation failed: {error}"));
                self.screen = Screen::Done;
            }
        }
    }

    fn screen_index(screen: Screen) -> usize {
        Screen::ORDER
            .iter()
            .position(|candidate| *candidate == screen)
            .unwrap_or(0)
    }
}

pub fn program_options_for(user_type: UserType) -> Vec<(&'static str, &'static str)> {
    match user_type {
        UserType::Gui => std::iter::once(("desktop", "Niri desktop environment"))
            .chain(GUI_PROGRAM_OPTIONS.iter().copied())
            .chain(DEV_PROGRAM_OPTIONS.iter().copied())
            .collect(),
        UserType::Cui => DEV_PROGRAM_OPTIONS.to_vec(),
    }
}

fn normalize_programs(user_type: UserType, programs: Vec<String>) -> Vec<String> {
    let options = program_options_for(user_type);
    let mut normalized = Vec::new();
    for (program, _) in options {
        if (program == "desktop" && user_type == UserType::Gui)
            || programs.iter().any(|candidate| candidate == program)
        {
            normalized.push(program.to_string());
        }
    }
    normalized
}

fn is_valid_username(username: &str) -> bool {
    let mut chars = username.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_lowercase() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

fn default_display_name(username: &str) -> String {
    let mut chars = username.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    format!("{}{}", first.to_ascii_uppercase(), chars.as_str())
}
