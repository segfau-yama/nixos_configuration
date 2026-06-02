use std::fs;

use serde::{Deserialize, Serialize};

pub const DEFAULT_REPOSITORY_PATH: &str = "/tmp/nixos_config";
pub const DEFAULT_REPOSITORY_URL: &str = "https://github.com/segfau-yama/nixos_configuration.git";

pub const KEYBOARD_OPTIONS: &[&str] = &["jp106", "us", "de", "fr"];
pub const LOCALE_OPTIONS: &[&str] = &["ja_JP.UTF-8", "en_US.UTF-8", "zh_CN.UTF-8", "ko_KR.UTF-8"];
pub const TIMEZONE_OPTIONS: &[&str] = &[
    "Asia/Tokyo",
    "UTC",
    "America/New_York",
    "America/Los_Angeles",
    "Europe/London",
    "Europe/Berlin",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserType {
    Gui,
    Tui,
    Cui,
}

impl UserType {
    pub fn label(self) -> &'static str {
        match self {
            Self::Gui => "gui",
            Self::Tui => "tui",
            Self::Cui => "cui",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserConfig {
    pub username: String,
    pub display_name: String,
    pub user_type: UserType,
    pub programs: Vec<String>,
    pub password_hash: String,
    pub is_preset: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallConfig {
    pub github_username: String,
    pub repository: String,
    pub repository_url: String,
    pub repository_path: String,
    pub device: String,
    pub boot_size: String,
    pub swap_size: String,
    pub hostname: String,
    pub keyboard: String,
    pub locale: String,
    pub timezone: String,
    pub ssh_enabled: bool,
    pub gpu_type: GpuType,
    pub gpu_custom: String,
    pub cpu_type: CpuType,
    pub cpu_custom: String,
    pub boot_type: BootType,
    pub users: Vec<UserConfig>,
}

impl Default for InstallConfig {
    fn default() -> Self {
        Self {
            github_username: String::new(),
            repository: "segfau-yama/nixos_configuration".to_string(),
            repository_url: DEFAULT_REPOSITORY_URL.to_string(),
            repository_path: DEFAULT_REPOSITORY_PATH.to_string(),
            device: String::new(),
            boot_size: "512MiB".to_string(),
            swap_size: "0".to_string(),
            hostname: "jadeos".to_string(),
            keyboard: "jp106".to_string(),
            locale: "ja_JP.UTF-8".to_string(),
            timezone: "Asia/Tokyo".to_string(),
            ssh_enabled: false,
            gpu_type: GpuType::None,
            gpu_custom: String::new(),
            cpu_type: CpuType::Amd,
            cpu_custom: String::new(),
            boot_type: BootType::SystemdBoot,
            users: Vec::new(),
        }
    }
}

impl InstallConfig {
    pub fn has_gui_user(&self) -> bool {
        self.users
            .iter()
            .any(|user| user.user_type == UserType::Gui)
    }

    pub fn has_swap_partition(&self) -> bool {
        !matches!(
            self.swap_size.trim(),
            "" | "0" | "0B" | "0K" | "0M" | "0G" | "0MiB" | "0GiB" | "none"
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceOption {
    pub path: String,
    pub size: String,
    pub model: String,
}

impl DeviceOption {
    pub fn label(&self) -> String {
        if self.model.trim().is_empty() {
            format!("{} ({})", self.path, self.size)
        } else {
            format!("{} ({}, {})", self.path, self.size, self.model)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HardwareInfo {
    pub cpu_brand: String,
    pub gpu_brand: String,
    pub cpu_type: CpuType,
    pub gpu_type: GpuType,
    pub boot_type: BootType,
}

pub fn sample_devices() -> Vec<DeviceOption> {
    vec![
        DeviceOption {
            path: "/dev/nvme0n1".to_string(),
            size: "1.0T".to_string(),
            model: "NVMe SSD".to_string(),
        },
        DeviceOption {
            path: "/dev/sda".to_string(),
            size: "512G".to_string(),
            model: "SATA SSD".to_string(),
        },
    ]
}

pub fn sample_hardware() -> HardwareInfo {
    let boot_type = if fs::metadata("/sys/firmware/efi/efivars").is_ok() {
        BootType::SystemdBoot
    } else {
        BootType::Grub
    };

    HardwareInfo {
        cpu_brand: "Detected CPU (sample)".to_string(),
        gpu_brand: "Detected GPU (sample)".to_string(),
        cpu_type: CpuType::Amd,
        gpu_type: GpuType::Intel,
        boot_type,
    }
}