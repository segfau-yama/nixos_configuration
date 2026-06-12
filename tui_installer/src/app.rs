use std::{
    collections::HashMap,
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
};

use color_eyre::eyre::Result;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::{
    action::{Action, ConfigChange, PendingUserChange},
    component::Component,
    components::{Footer, Header, Sidebar},
    config::{
        DeviceOption, HardwareInfo, InstallConfig, UserConfig, UserType, apply_detected_config,
        collect_block_devices, collect_hardware,
    },
    event::Event,
    infra::{
        github::prepare_github_repository, install::run_phase3_install_streaming,
        network::check_network_connectivity, password_hasher::hash_password,
    },
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
    Done,
}

impl Screen {
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
            Self::Summary | Self::Done => Phase::Install,
        }
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
    pub install_running: bool,
    pub install_finished: bool,
    pub repository_prepare_running: bool,
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
    install_running: bool,
    install_finished: bool,
    install_log_receiver: Option<Receiver<String>>,
    install_result_receiver: Option<Receiver<std::result::Result<(), String>>>,
    repository_prepare_running: bool,
    repository_prepare_receiver: Option<Receiver<RepositoryPrepareOutcome>>,
}

struct RepositoryPrepareOutcome {
    config: InstallConfig,
    logs: Vec<String>,
    result: std::result::Result<(), String>,
}

impl App {
    pub fn new() -> Result<Self> {
        let mut pages = build_pages();
        for page in pages.values_mut() {
            page.init()?;
        }

        let hardware = collect_hardware();
        let devices = collect_block_devices();
        let mut config = InstallConfig::default();
        apply_detected_config(&mut config, &hardware);
        if let Some(device) = devices.first() {
            config.device = device.path.clone();
        }

        let mut app = Self {
            config,
            current_screen: Screen::Welcome,
            should_quit: false,
            status_message: None,
            pending_user: None,
            install_log: Vec::new(),
            header: Header::new(),
            footer: Footer::new(),
            sidebar: Sidebar::new(),
            pages,
            hardware,
            devices,
            install_running: false,
            install_finished: false,
            install_log_receiver: None,
            install_result_receiver: None,
            repository_prepare_running: false,
            repository_prepare_receiver: None,
        };

        app.header.init()?;
        app.footer.init()?;
        app.sidebar.init()?;
        app.sync_components();
        Ok(app)
    }

    pub fn handle_event(&mut self, event: Option<Event>) -> Action {
        self.poll_repository_prepare_worker();
        self.poll_install_worker();
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
        self.sync_components();
        frame.render_widget(Clear, frame.area());

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
            .constraints([Constraint::Percentage(68), Constraint::Percentage(32)])
            .split(layout[1]);

        self.header.render(frame, layout[0]);
        let body_block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", self.current_screen.title()))
            .border_style(Style::default().fg(main_border_color(self.current_screen)));
        let body_inner = body_block.inner(body[0]);
        frame.render_widget(body_block, body[0]);

        let body_inner_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(3)])
            .split(body_inner);

        let page_info = Paragraph::new(page_info_text(self.current_screen))
            .style(Style::default().fg(Color::Gray))
            .wrap(Wrap { trim: false });
        frame.render_widget(page_info, body_inner_layout[0]);

        let popup = if let Some(page) = self.pages.get_mut(&self.current_screen) {
            page.render(frame, body_inner_layout[1]);
            page.popup()
        } else {
            None
        };

        self.sidebar.render(frame, body[1]);
        self.footer.render(frame, layout[2]);

        if let Some(popup) = popup {
            popup.render(frame, frame.area());
        }
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
            Action::Navigate(screen) => {
                self.current_screen = screen;
                if screen == Screen::Done {
                    if !self.install_running && !self.install_finished {
                        self.install_log.clear();
                        self.status_message =
                            Some("Opening install log. Installation will start now.".to_string());
                    }
                } else {
                    self.status_message = None;
                }
            }
            Action::SetStatus(message) => self.status_message = message,
            Action::CheckNetwork => self.check_network(),
            Action::PrepareRepository => self.prepare_repository(),
            Action::StartInstall => self.start_install(),
            Action::ConfigChanged(change) => self.apply_config_change(change),
            Action::PendingUserChanged(change) => self.apply_pending_user_change(change),
            Action::StartPresetUser(user) => {
                self.pending_user = Some(user);
                self.current_screen = Screen::PresetUserPassword;
                self.status_message = None;
            }
            Action::CommitPendingUser => self.commit_pending_user(),
            Action::ResetPendingUser => self.pending_user = None,
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
                if let Some(index) = pending_user
                    .programs
                    .iter()
                    .position(|item| item == &program)
                {
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

        let password_hash = match hash_password(&user.password) {
            Ok(hash) => hash,
            Err(error) => {
                self.status_message = Some(format!("Password hashing failed: {error}"));
                return;
            }
        };

        self.config.users.push(UserConfig {
            username: user.username.clone(),
            display_name: user.display_name.clone(),
            user_type: user.user_type,
            programs: user.programs.clone(),
            password_hash,
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
            install_running: self.install_running,
            install_finished: self.install_finished,
            repository_prepare_running: self.repository_prepare_running,
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

    fn check_network(&mut self) {
        match check_network_connectivity() {
            Ok(()) => {
                self.status_message = Some("Network check passed".to_string());
                self.current_screen = Screen::GitHubLogin;
            }
            Err(error) => self.status_message = Some(error),
        }
    }

    fn prepare_repository(&mut self) {
        if self.repository_prepare_running {
            self.status_message = Some("Repository preparation is already running.".to_string());
            return;
        }

        let mut config = self.config.clone();
        let (sender, receiver) = mpsc::channel();
        self.repository_prepare_receiver = Some(receiver);
        self.repository_prepare_running = true;
        self.status_message = Some("Preparing repository...".to_string());

        thread::spawn(move || {
            let mut logs = Vec::new();
            let result = prepare_github_repository(&mut config, &mut logs);
            let _ = sender.send(RepositoryPrepareOutcome {
                config,
                logs,
                result,
            });
        });
    }

    fn poll_repository_prepare_worker(&mut self) {
        let Some(receiver) = self.repository_prepare_receiver.take() else {
            return;
        };

        match receiver.try_recv() {
            Ok(outcome) => {
                self.repository_prepare_running = false;
                self.install_log.extend(outcome.logs);
                match outcome.result {
                    Ok(()) => {
                        self.config = outcome.config;
                        self.status_message =
                            Some(format!("Repository ready: {}", self.config.repository_path));
                        self.current_screen = Screen::DeviceSelect;
                    }
                    Err(error) => {
                        self.status_message = Some(error);
                    }
                }
            }
            Err(TryRecvError::Empty) => {
                self.repository_prepare_receiver = Some(receiver);
            }
            Err(TryRecvError::Disconnected) => {
                self.repository_prepare_running = false;
                self.status_message =
                    Some("Repository preparation failed: worker disconnected".to_string());
            }
        }
    }

    fn start_install(&mut self) {
        if self.install_running {
            return;
        }
        if self.install_finished {
            self.status_message =
                Some("Installation already finished. Review logs below.".to_string());
            return;
        }

        let config = self.config.clone();
        let users = self.config.users.clone();
        let (log_sender, log_receiver) = mpsc::channel();
        let (result_sender, result_receiver) = mpsc::channel();
        self.install_log_receiver = Some(log_receiver);
        self.install_result_receiver = Some(result_receiver);
        self.install_running = true;
        self.install_log.clear();
        self.install_log
            .push("install: background installer started; live logs will appear below".to_string());
        self.status_message =
            Some("Installation running. Review the install log modal.".to_string());

        thread::spawn(move || {
            let result = run_phase3_install_streaming(&config, &users, log_sender);
            let _ = result_sender.send(result);
        });
    }

    fn poll_install_worker(&mut self) {
        self.poll_install_logs();
        self.poll_install_result();
    }

    fn poll_install_logs(&mut self) {
        let Some(receiver) = self.install_log_receiver.take() else {
            return;
        };

        let mut keep_receiver = true;
        loop {
            match receiver.try_recv() {
                Ok(line) => {
                    self.install_log.push(line);
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    keep_receiver = false;
                    break;
                }
            }
        }

        if keep_receiver {
            self.install_log_receiver = Some(receiver);
        }
    }

    fn poll_install_result(&mut self) {
        let Some(receiver) = self.install_result_receiver.take() else {
            return;
        };

        match receiver.try_recv() {
            Ok(result) => {
                self.install_running = false;
                self.install_finished = true;
                match result {
                    Ok(()) => {
                        self.install_log.push(
                            "install: SUCCESS - installation complete. Reboot after reviewing logs."
                                .to_string(),
                        );
                        self.status_message =
                            Some("Installation complete. Reboot after reviewing logs.".to_string());
                    }
                    Err(error) => {
                        self.install_log.push(format!("install failed: {error}"));
                        self.status_message = Some(format!("Installation failed: {error}"));
                    }
                }
            }
            Err(TryRecvError::Empty) => {
                self.install_result_receiver = Some(receiver);
            }
            Err(TryRecvError::Disconnected) => {
                self.install_running = false;
                self.install_finished = true;
                self.install_log
                    .push("install failed: installer worker disconnected".to_string());
                self.status_message =
                    Some("Installation failed: installer worker disconnected".to_string());
            }
        }
    }
}

impl Component for App {
    fn handle_events(&mut self, event: Option<Event>) -> Action {
        self.handle_event(event)
    }

    fn update(&mut self, action: Action) -> Action {
        App::update(self, action);
        Action::Noop
    }

    fn render(&mut self, frame: &mut Frame, _rect: Rect) {
        App::render(self, frame);
    }
}

fn page_info_text(screen: Screen) -> Text<'static> {
    Text::from(vec![
        Line::from(vec![
            Span::styled("phase: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                phase_label(screen.phase()),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("screen: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                screen.title(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ])
}

fn phase_label(phase: Phase) -> &'static str {
    match phase {
        Phase::PcConfig => "PC Config",
        Phase::Users => "Users",
        Phase::Install => "Install",
    }
}

fn main_border_color(screen: Screen) -> Color {
    match screen {
        Screen::Welcome | Screen::Done => Color::Green,
        Screen::Summary => Color::Magenta,
        Screen::UserMenu => Color::Cyan,
        _ => Color::Blue,
    }
}
