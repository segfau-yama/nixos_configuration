use crossterm::event::{KeyCode, KeyEvent};

use crate::config::InstallConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    Summary,
    Installing,
    Done,
}

impl Screen {
    pub const ORDER: [Self; 16] = [
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
            Self::UserMenu => "Users",
            Self::Summary | Self::Installing | Self::Done => "Install",
        }
    }
}

#[derive(Debug)]
pub struct App {
    pub screen: Screen,
    pub config: InstallConfig,
    pub install_log: Vec<String>,
    pub should_quit: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            screen: Screen::Welcome,
            config: InstallConfig::default(),
            install_log: vec!["installer bootstrap ready".to_string()],
            should_quit: false,
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

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Right | KeyCode::Enter | KeyCode::Char('l') => self.next_screen(),
            KeyCode::Left | KeyCode::Backspace | KeyCode::Char('h') => self.previous_screen(),
            KeyCode::Esc => {
                if self.screen == Screen::Welcome {
                    self.should_quit = true;
                } else {
                    self.previous_screen();
                }
            }
            _ => {}
        }
    }

    fn screen_index(screen: Screen) -> usize {
        Screen::ORDER
            .iter()
            .position(|candidate| *candidate == screen)
            .unwrap_or(0)
    }
}