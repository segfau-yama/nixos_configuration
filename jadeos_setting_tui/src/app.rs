use crossterm::event::KeyEvent;
use std::collections::HashMap;

use crate::config::{InstallConfig, UserType};
use crate::infra::github::prepare_github_repository;
use crate::infra::install::run_phase3_install;
use crate::infra::network::check_network_connectivity;
use crate::logic::setup::{
    DeviceOption, HardwareInfo, apply_detected_config, collect_block_devices, collect_hardware,
};
use crate::update;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Screen {
    Welcome,
    GitHubLogin,
    HardwareDetect,
    DeviceSelect,
    PartitionConfig,
    PartitionConfirm,
    HostnameInput,
    KeyboardSelect,
    LocaleSelect,
    TimezoneSelect,
    SshToggle,
    StorageToggle,
    GpuSelect,
    CpuSelect,
    BootTypeSelect,
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
    pub const ORDER: [Self; 25] = [
        Self::Welcome,
        Self::GitHubLogin,
        Self::HardwareDetect,
        Self::DeviceSelect,
        Self::PartitionConfig,
        Self::PartitionConfirm,
        Self::HostnameInput,
        Self::KeyboardSelect,
        Self::LocaleSelect,
        Self::TimezoneSelect,
        Self::SshToggle,
        Self::StorageToggle,
        Self::GpuSelect,
        Self::CpuSelect,
        Self::BootTypeSelect,
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
            Self::GitHubLogin => "GitHub Login",
            Self::HardwareDetect => "Hardware Detect",
            Self::DeviceSelect => "Device Select",
            Self::PartitionConfig => "Partition Config",
            Self::PartitionConfirm => "Partition Confirm",
            Self::HostnameInput => "Hostname",
            Self::KeyboardSelect => "Keyboard",
            Self::LocaleSelect => "Locale",
            Self::TimezoneSelect => "Timezone",
            Self::SshToggle => "SSH",
            Self::StorageToggle => "Storage",
            Self::GpuSelect => "GPU",
            Self::CpuSelect => "CPU",
            Self::BootTypeSelect => "Boot Type",
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
            | Self::GitHubLogin
            | Self::HardwareDetect
            | Self::DeviceSelect
            | Self::PartitionConfig
            | Self::PartitionConfirm
            | Self::HostnameInput
            | Self::KeyboardSelect
            | Self::LocaleSelect
            | Self::TimezoneSelect
            | Self::SshToggle
            | Self::StorageToggle
            | Self::GpuSelect
            | Self::CpuSelect
            | Self::BootTypeSelect => "PC Config",
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
    pub device_options: Vec<DeviceOption>,
    pub install_log: Vec<String>,
    pub should_quit: bool,
    pub input_state: HashMap<Screen, ScreenInputState>,
    pub pending_user: Option<PendingUser>,
    pub user_message: Option<String>,
    pub partition_confirmation: String,
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
        let device_options = collect_block_devices();
        apply_detected_config(&mut config, &hardware);
        if let Some(device) = device_options.first() {
            config.device = device.path.clone();
        }

        Self {
            screen: Screen::Welcome,
            config,
            hardware,
            device_options,
            install_log: vec!["installer bootstrap ready".to_string()],
            should_quit: false,
            input_state,
            pending_user: None,
            user_message: None,
            partition_confirmation: String::new(),
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
            Screen::Welcome => self.confirm_welcome(),
            Screen::GitHubLogin => self.confirm_github_login(),
            Screen::DeviceSelect => self.confirm_device_select(),
            Screen::PartitionConfig => self.confirm_partition_config(),
            Screen::PartitionConfirm => self.confirm_partition_plan(),
            Screen::HostnameInput => self.confirm_hostname(),
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
            Screen::PartitionConfirm => self.screen = Screen::PartitionConfig,
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

    fn confirm_welcome(&mut self) {
        match check_network_connectivity() {
            Ok(()) => {
                self.user_message = Some("Network check passed.".to_string());
                self.next_screen();
            }
            Err(error) => {
                self.user_message = Some(error);
            }
        }
    }

    fn confirm_github_login(&mut self) {
        let mut logs = Vec::new();
        match prepare_github_repository(&mut self.config, &mut logs) {
            Ok(()) => {
                self.install_log.extend(logs);
                self.user_message =
                    Some(format!("Repository ready: {}", self.config.repository_path));
                self.next_screen();
            }
            Err(error) => {
                self.install_log.extend(logs);
                self.user_message = Some(error);
            }
        }
    }

    fn confirm_device_select(&mut self) {
        let Some(device) = self
            .device_options
            .get(self.active_field_for_current_screen())
            .map(|option| option.path.clone())
        else {
            self.user_message = Some("No install target disks were detected by lsblk.".to_string());
            return;
        };
        let device = device.trim();
        if device.is_empty() {
            self.user_message = Some("Target device cannot be empty.".to_string());
            return;
        }
        if !device.starts_with("/dev/") {
            self.user_message = Some("Target device must be under /dev/.".to_string());
            return;
        }
        self.config.device = device.to_string();
        self.next_screen();
    }

    fn confirm_partition_config(&mut self) {
        let boot_size = self.config.boot_size.trim().to_string();
        let swap_size = self.config.swap_size.trim().to_string();
        if !looks_like_parted_size(&boot_size) {
            self.user_message = Some("Boot size must be a parted size like 512MiB.".to_string());
            return;
        }
        if self.config.has_swap_partition() && !looks_like_parted_size(&swap_size) {
            self.user_message = Some("Swap size must be 0 or a parted size like 2GiB.".to_string());
            return;
        }
        self.config.boot_size = boot_size;
        self.config.swap_size = if self.config.has_swap_partition() {
            swap_size
        } else {
            "0".to_string()
        };
        self.next_screen();
    }

    fn confirm_hostname(&mut self) {
        let hostname = self.config.hostname.trim();
        if !is_valid_hostname(hostname) {
            self.user_message =
                Some("Hostname must use lowercase letters, digits, and hyphen.".to_string());
            return;
        }
        self.config.hostname = hostname.to_string();
        self.next_screen();
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

    fn confirm_partition_plan(&mut self) {
        if self.partition_confirmation.trim() != "yes" {
            self.user_message = Some("Type 'yes' in confirm field before continuing.".to_string());
            return;
        }
        self.user_message = None;
        self.next_screen();
    }

    fn screen_index(screen: Screen) -> usize {
        Screen::ORDER
            .iter()
            .position(|candidate| *candidate == screen)
            .unwrap_or(0)
    }
}

fn looks_like_parted_size(value: &str) -> bool {
    let mut chars = value.chars().peekable();
    let mut has_digit = false;
    while matches!(chars.peek(), Some(candidate) if candidate.is_ascii_digit()) {
        chars.next();
        has_digit = true;
    }
    let mut has_unit = false;
    for c in chars {
        if !c.is_ascii_alphabetic() {
            return false;
        }
        has_unit = true;
    }
    has_digit && has_unit
}

fn is_valid_hostname(value: &str) -> bool {
    if value.is_empty() || value.starts_with('-') || value.ends_with('-') {
        return false;
    }
    value
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

#[cfg(test)]
mod tests {
    use super::{is_valid_hostname, looks_like_parted_size};

    #[test]
    fn parted_size_requires_digits_and_unit() {
        assert!(looks_like_parted_size("512MiB"));
        assert!(looks_like_parted_size("2GiB"));
        assert!(!looks_like_parted_size(""));
        assert!(!looks_like_parted_size("512"));
        assert!(!looks_like_parted_size("512 MiB"));
        assert!(!looks_like_parted_size("half"));
    }

    #[test]
    fn hostname_allows_lowercase_digits_and_internal_hyphen() {
        assert!(is_valid_hostname("jade-develop1"));
        assert!(!is_valid_hostname(""));
        assert!(!is_valid_hostname("-jade"));
        assert!(!is_valid_hostname("jade-"));
        assert!(!is_valid_hostname("Jade"));
        assert!(!is_valid_hostname("jade_dev"));
    }
}
