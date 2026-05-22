#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuType {
    None,
    Nvidia,
    Amd,
    Intel,
}

impl GpuType {
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Nvidia => "nvidia",
            Self::Amd => "amd",
            Self::Intel => "intel",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuType {
    Amd,
    Intel,
    Aarch64,
}

impl CpuType {
    pub fn label(self) -> &'static str {
        match self {
            Self::Amd => "amd",
            Self::Intel => "intel",
            Self::Aarch64 => "aarch64",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootType {
    SystemdBoot,
    Grub,
}

impl BootType {
    pub fn label(self) -> &'static str {
        match self {
            Self::SystemdBoot => "systemd-boot",
            Self::Grub => "grub",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserType {
    Gui,
    Cui,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserConfig {
    pub username: String,
    pub display_name: String,
    pub user_type: UserType,
    pub programs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallConfig {
    pub device: String,
    pub boot_end: String,
    pub root_end: String,
    pub hostname: String,
    pub keyboard: String,
    pub locale: String,
    pub timezone: String,
    pub ssh_enabled: bool,
    pub storage_enabled: bool,
    pub gpu_type: GpuType,
    pub cpu_type: CpuType,
    pub boot_type: BootType,
    pub users: Vec<UserConfig>,
}

impl Default for InstallConfig {
    fn default() -> Self {
        Self {
            device: String::new(),
            boot_end: "512MiB".to_string(),
            root_end: "100GiB".to_string(),
            hostname: "nixos".to_string(),
            keyboard: "jp106".to_string(),
            locale: "ja_JP.UTF-8".to_string(),
            timezone: "Asia/Tokyo".to_string(),
            ssh_enabled: false,
            storage_enabled: false,
            gpu_type: GpuType::None,
            cpu_type: CpuType::Amd,
            boot_type: BootType::SystemdBoot,
            users: Vec::new(),
        }
    }
}