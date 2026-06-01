use crossterm::event::KeyEvent;
use std::collections::HashMap;

use crate::config::{InstallConfig, UserType};
use crate::infra::install::run_phase3_install;
use crate::logic::setup::{HardwareInfo, apply_detected_config, collect_hardware};
use crate::update;

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
    JadeCore,
    JadeOffice,
    JadeGaming,
    JadeDevelop,
    JadeFull,
    Custom,
    Finish,
}

impl UserMenuChoice {
    pub(crate) fn from_index(index: usize) -> Self {
        match index {
            0 => Self::JadeCore,
            1 => Self::JadeOffice,
            2 => Self::JadeGaming,
            3 => Self::JadeDevelop,
            4 => Self::JadeFull,
            5 => Self::Custom,
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
    pub(crate) fn preset(
        username: &str,
        display_name: &str,
        user_type: UserType,
        programs: &[&str],
    ) -> Self {
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

    pub(crate) fn custom() -> Self {
        Self {
            username: String::new(),
            display_name: String::new(),
            user_type: UserType::Gui,
            programs: Vec::new(),
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
    pub input_mode: InputMode,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ScreenInputState {
    pub active_field: usize,
    pub custom_selected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    Normal,
    Editing,
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
            input_mode: InputMode::Normal,
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

    fn editable_field_count(&self, screen: Screen) -> usize {
        update::pc_config::editable_field_count(self, screen)
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        update::handle_key(self, key);
    }

    pub fn is_editing(&self) -> bool {
        self.input_mode == InputMode::Editing
    }

    pub(crate) fn set_input_mode(&mut self, mode: InputMode) {
        self.input_mode = mode;
    }

    pub(crate) fn confirm_current_screen(&mut self) {
        self.input_mode = InputMode::Normal;
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

    pub(crate) fn previous_screen_for_current_flow(&mut self) {
        self.input_mode = InputMode::Normal;
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

    pub(crate) fn move_active_field(&mut self, direction: isize) {
        update::pc_config::move_active_field(self, direction);
    }

    pub(crate) fn insert_active_field_char(&mut self, c: char) {
        update::pc_config::insert_active_field_char(self, c);
    }

    pub(crate) fn backspace_active_field(&mut self) {
        update::pc_config::backspace_active_field(self);
    }

    pub(crate) fn toggle_active_field(&mut self) {
        update::pc_config::toggle_active_field(self);
    }

    pub(crate) fn active_field_accepts_text(&self) -> bool {
        update::pc_config::active_field_accepts_text(self)
    }

    fn confirm_user_menu(&mut self) {
        update::user_flow::confirm_user_menu(self);
    }

    fn confirm_custom_basic(&mut self) {
        update::user_flow::confirm_custom_basic(self);
    }

    fn confirm_user_password(&mut self) {
        update::user_flow::confirm_user_password(self);
    }

    fn sync_custom_user_type(&mut self) {
        update::user_flow::sync_custom_user_type(self);
    }

    pub fn user_exists(&self, username: &str) -> bool {
        self.config
            .users
            .iter()
            .any(|user| user.username == username)
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
