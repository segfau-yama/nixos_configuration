use std::collections::HashMap;

use color_eyre::eyre::Result;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

use crate::{
    action::{Action, ConfigChange, PendingUserChange},
    component::Component,
    components::{Footer, Header, Sidebar},
    config::{
        DeviceOption, HardwareInfo, InstallConfig, UserConfig, UserType, sample_devices,
        sample_hardware,
    },
    event::Event,
    pages::{InstallerPage, build_pages},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    PcConfig,
    Users,
    Install,
}

impl Phase {
    pub fn tab_index(self) -> usize {
        match self {
            Self::PcConfig => 0,
            Self::Users => 1,
            Self::Install => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Screen {
    Welcome,
    GitHubLogin,
    DeviceSelect,
    PartitionConfig,
    PartitionConfirm,
    HostSelect,
    HostnameInput,
    HardwareDetect,
    GpuSelect,
    CpuSelect,
    BootTypeSelect,
    KeyboardSelect,
    LocaleSelect,
    TimezoneSelect,
    SshToggle,
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
        Self::DeviceSelect,
        Self::PartitionConfig,
        Self::PartitionConfirm,
        Self::HostSelect,
        Self::HostnameInput,
        Self::HardwareDetect,
        Self::GpuSelect,
        Self::CpuSelect,
        Self::BootTypeSelect,
        Self::KeyboardSelect,
        Self::LocaleSelect,
        Self::TimezoneSelect,
        Self::SshToggle,
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
            Self::DeviceSelect => "Device Select",
            Self::PartitionConfig => "Partition Config",
            Self::PartitionConfirm => "Partition Confirm",
            Self::HostSelect => "Host Select",
            Self::HostnameInput => "Hostname",
            Self::HardwareDetect => "Hardware Detect",
            Self::GpuSelect => "GPU",
            Self::CpuSelect => "CPU",
            Self::BootTypeSelect => "Boot Type",
            Self::KeyboardSelect => "Keyboard",
            Self::LocaleSelect => "Locale",
            Self::TimezoneSelect => "Timezone",
            Self::SshToggle => "SSH",
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

    pub fn phase(self) -> Phase {
        match self {
            Self::Welcome
            | Self::GitHubLogin
            | Self::DeviceSelect
            | Self::PartitionConfig
            | Self::PartitionConfirm
            | Self::HostSelect
            | Self::HostnameInput
            | Self::HardwareDetect
            | Self::GpuSelect
            | Self::CpuSelect
            | Self::BootTypeSelect
            | Self::KeyboardSelect
            | Self::LocaleSelect
            | Self::TimezoneSelect
            | Self::SshToggle => Phase::PcConfig,
            Self::UserMenu
            | Self::PresetUserPassword
            | Self::CustomUserBasic
            | Self::CustomUserType
            | Self::CustomUserPrograms
            | Self::CustomUserPassword
            | Self::UserAddResult => Phase::Users,
            Self::Summary | Self::Installing | Self::Done => Phase::Install,
        }
    }

    pub fn next(self) -> Self {
        let index = Self::ORDER
            .iter()
            .position(|screen| *screen == self)
            .unwrap_or(0);
        Self::ORDER[(index + 1).min(Self::ORDER.len() - 1)]
    }

    pub fn previous(self) -> Self {
        let index = Self::ORDER
            .iter()
            .position(|screen| *screen == self)
            .unwrap_or(0);
        Self::ORDER[index.saturating_sub(1)]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub fn preset(
        username: &str,
        display_name: &str,
        user_type: UserType,
        programs: &[&str],
    ) -> Self {
        Self {
            username: username.to_string(),
            display_name: display_name.to_string(),
            user_type,
            programs: programs.iter().map(|value| value.to_string()).collect(),
            password: String::new(),
            password_confirm: String::new(),
            is_preset: true,
        }
    }

    pub fn custom() -> Self {
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

#[derive(Debug, Clone)]
pub struct AppSnapshot {
    pub screen: Screen,
    pub config: InstallConfig,
    pub pending_user: Option<PendingUser>,
    pub status_message: Option<String>,
    pub hardware: HardwareInfo,
    pub devices: Vec<DeviceOption>,
    pub install_log: Vec<String>,
    pub size: (u16, u16),
}

pub struct App {
    pub config: InstallConfig,
    pub current_screen: Screen,
    pub should_quit: bool,
    pub status_message: Option<String>,
    pub pending_user: Option<PendingUser>,
    pub install_log: Vec<String>,
    pub header: Header,
    pub footer: Footer,
    pub sidebar: Sidebar,
    pub pages: HashMap<Screen, Box<dyn InstallerPage>>,
    pub hardware: HardwareInfo,
    pub devices: Vec<DeviceOption>,
    pub size: (u16, u16),
}

impl App {
    pub fn new() -> Result<Self> {
        let mut pages = build_pages();
        for page in pages.values_mut() {
            page.init()?;
        }

        let mut app = Self {
            config: InstallConfig::default(),
            current_screen: Screen::Welcome,
            should_quit: false,
            status_message: None,
            pending_user: None,
            install_log: Vec::new(),
            header: Header::new(),
            footer: Footer::new(),
            sidebar: Sidebar::new(),
            pages,
            hardware: sample_hardware(),
            devices: sample_devices(),
            size: (0, 0),
        };

        app.header.init()?;
        app.footer.init()?;
        app.sidebar.init()?;
        app.sync_components();
        Ok(app)
    }

    pub fn handle_event(&mut self, event: Option<Event>) -> Action {
        self.sync_components();
        self.pages
            .get_mut(&self.current_screen)
            .map(|page| page.handle_events(event))
            .unwrap_or(Action::Noop)
    }

    pub fn update(&mut self, action: Action) {
        self.process_action(action, 0);
    }

    pub fn render(&mut self, frame: &mut Frame) {
        self.size = (frame.area().width, frame.area().height);
        self.sync_components();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(frame.area());

        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(layout[1]);

        self.header.render(frame, layout[0]);
        if let Some(page) = self.pages.get_mut(&self.current_screen) {
            page.render(frame, body[0]);
        }
        self.sidebar.render(frame, body[1]);
        self.footer.render(frame, layout[2]);
    }

    fn process_action(&mut self, action: Action, depth: usize) {
        if depth > 12 {
            return;
        }

        if matches!(action, Action::Noop) {
            return;
        }

        if let Action::Batch(actions) = action {
            for next in actions {
                self.process_action(next, depth + 1);
            }
            return;
        }

        self.apply_action(action.clone());
        self.sync_components();

        let follow_up = self
            .pages
            .get_mut(&self.current_screen)
            .map(|page| page.update(action))
            .unwrap_or(Action::Noop);

        if !matches!(follow_up, Action::Noop) {
            self.process_action(follow_up, depth + 1);
        }
    }

    fn apply_action(&mut self, action: Action) {
        match action {
            Action::Noop | Action::Batch(_) => {}
            Action::Quit => self.should_quit = true,
            Action::Tick => {}
            Action::Resize(width, height) => self.size = (width, height),
            Action::Navigate(screen) => {
                self.current_screen = screen;
                self.status_message = None;
            }
            Action::NextScreen => {
                self.current_screen = self.current_screen.next();
                self.status_message = None;
            }
            Action::PrevScreen => {
                self.current_screen = self.current_screen.previous();
                self.status_message = None;
            }
            Action::SetStatus(message) => self.status_message = message,
            Action::ConfigChanged(change) => self.apply_config_change(change),
            Action::PendingUserChanged(change) => self.apply_pending_user_change(change),
            Action::StartPresetUser(user) => {
                self.pending_user = Some(user);
                self.current_screen = Screen::PresetUserPassword;
                self.status_message = None;
            }
            Action::CommitPendingUser => self.commit_pending_user(),
            Action::ResetPendingUser => self.pending_user = None,
            Action::StartInstall => {
                self.install_log.clear();
                self.current_screen = Screen::Installing;
                self.status_message = Some("Starting install flow".to_string());
            }
            Action::AppendInstallLog(line) => self.install_log.push(line),
            Action::InstallComplete => {
                self.current_screen = Screen::Done;
                self.status_message = Some("Install flow completed".to_string());
            }
            Action::InstallFailed(message) => {
                self.current_screen = Screen::Summary;
                self.status_message = Some(message);
            }
        }
    }

    fn apply_config_change(&mut self, change: ConfigChange) {
        match change {
            ConfigChange::GitHubUsername(value) => self.config.github_username = value,
            ConfigChange::Repository(value) => self.config.repository = value,
            ConfigChange::RepositoryPath(value) => self.config.repository_path = value,
            ConfigChange::Device(value) => self.config.device = value,
            ConfigChange::BootSize(value) => self.config.boot_size = value,
            ConfigChange::SwapSize(value) => self.config.swap_size = value,
            ConfigChange::Hostname(value) => self.config.hostname = value,
            ConfigChange::Keyboard(value) => self.config.keyboard = value,
            ConfigChange::Locale(value) => self.config.locale = value,
            ConfigChange::Timezone(value) => self.config.timezone = value,
            ConfigChange::SshEnabled(value) => self.config.ssh_enabled = value,
            ConfigChange::GpuType(value) => self.config.gpu_type = value,
            ConfigChange::GpuCustom(value) => self.config.gpu_custom = value,
            ConfigChange::CpuType(value) => self.config.cpu_type = value,
            ConfigChange::CpuCustom(value) => self.config.cpu_custom = value,
            ConfigChange::BootType(value) => self.config.boot_type = value,
        }
    }

    fn apply_pending_user_change(&mut self, change: PendingUserChange) {
        let pending_user = self.pending_user.get_or_insert_with(PendingUser::custom);
        match change {
            PendingUserChange::Username(value) => pending_user.username = value,
            PendingUserChange::DisplayName(value) => pending_user.display_name = value,
            PendingUserChange::UserType(value) => pending_user.user_type = value,
            PendingUserChange::ToggleProgram(program) => {
                if let Some(index) = pending_user.programs.iter().position(|item| item == &program) {
                    pending_user.programs.remove(index);
                } else {
                    pending_user.programs.push(program);
                    pending_user.programs.sort();
                }
            }
            PendingUserChange::Password(value) => pending_user.password = value,
            PendingUserChange::PasswordConfirm(value) => pending_user.password_confirm = value,
            PendingUserChange::Replace(user) => *pending_user = user,
        }
    }

    fn commit_pending_user(&mut self) {
        let Some(user) = self.pending_user.clone() else {
            self.status_message = Some("No pending user to commit".to_string());
            return;
        };

        if user.username.trim().is_empty() || user.display_name.trim().is_empty() {
            self.status_message = Some("Username and display name are required".to_string());
            return;
        }

        if user.password.trim().is_empty() {
            self.status_message = Some("Password is required".to_string());
            return;
        }

        if user.password != user.password_confirm {
            self.status_message = Some("Passwords do not match".to_string());
            return;
        }

        self.config.users.push(UserConfig {
            username: user.username.clone(),
            display_name: user.display_name.clone(),
            user_type: user.user_type,
            programs: user.programs.clone(),
            password_hash: format!("plain-text-placeholder:{}", user.password),
            is_preset: user.is_preset,
        });
        self.pending_user = Some(user.clone());
        self.current_screen = Screen::UserAddResult;
        self.status_message = Some(format!("User '{}' added", user.username));
    }

    fn snapshot(&self) -> AppSnapshot {
        AppSnapshot {
            screen: self.current_screen,
            config: self.config.clone(),
            pending_user: self.pending_user.clone(),
            status_message: self.status_message.clone(),
            hardware: self.hardware.clone(),
            devices: self.devices.clone(),
            install_log: self.install_log.clone(),
            size: self.size,
        }
    }

    fn sync_components(&mut self) {
        let snapshot = self.snapshot();
        self.header.sync(&snapshot);
        self.footer.sync(&snapshot);
        self.sidebar.sync(&snapshot);
        if let Some(page) = self.pages.get_mut(&self.current_screen) {
            page.sync(&snapshot);
        }
    }
}