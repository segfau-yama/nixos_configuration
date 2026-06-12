use std::fs;

use serde::{Deserialize, Serialize};

use crate::infra::command_runner::{CommandOutput, CommandRunner, SystemCommandRunner};

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
    pub storage_enabled: bool,
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
            boot_size: "2GiB".to_string(),
            swap_size: "0".to_string(),
            hostname: "nixos".to_string(),
            keyboard: "jp106".to_string(),
            locale: "ja_JP.UTF-8".to_string(),
            timezone: "Asia/Tokyo".to_string(),
            ssh_enabled: false,
            storage_enabled: false,
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

    pub fn needs_programming_cli(&self) -> bool {
        self.users
            .iter()
            .any(|user| user.programs.iter().any(|program| program == "programming"))
    }

    pub fn has_swap_partition(&self) -> bool {
        !matches!(
            self.swap_size.trim(),
            "" | "0" | "0B" | "0K" | "0M" | "0G" | "0MiB" | "0GiB" | "none" | "false" | "no"
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

pub fn collect_block_devices() -> Vec<DeviceOption> {
    let runner = SystemCommandRunner;
    detect_block_devices(&runner)
}

pub fn collect_hardware() -> HardwareInfo {
    let runner = SystemCommandRunner;
    detect_hardware(&runner)
}

pub fn apply_detected_config(config: &mut InstallConfig, hardware: &HardwareInfo) {
    config.cpu_type = hardware.cpu_type;
    config.gpu_type = hardware.gpu_type;
    config.boot_type = hardware.boot_type;
}

fn detect_block_devices<R: CommandRunner>(runner: &R) -> Vec<DeviceOption> {
    runner
        .run("lsblk", &["-dP", "-o", "NAME,SIZE,TYPE,MODEL"])
        .ok()
        .map(parse_lsblk_devices)
        .unwrap_or_default()
}

fn parse_lsblk_devices(output: CommandOutput) -> Vec<DeviceOption> {
    output
        .stdout
        .lines()
        .filter_map(parse_lsblk_device_line)
        .collect()
}

fn parse_lsblk_device_line(line: &str) -> Option<DeviceOption> {
    let name = quoted_value(line, "NAME")?;
    let device_type = quoted_value(line, "TYPE")?;
    if device_type != "disk" {
        return None;
    }

    Some(DeviceOption {
        path: format!("/dev/{name}"),
        size: quoted_value(line, "SIZE").unwrap_or_else(|| "unknown".to_string()),
        model: quoted_value(line, "MODEL").unwrap_or_default(),
    })
}

fn quoted_value(line: &str, key: &str) -> Option<String> {
    let needle = format!("{key}=\"");
    let start = line.find(&needle)? + needle.len();
    let rest = &line[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn detect_hardware<R: CommandRunner>(runner: &R) -> HardwareInfo {
    let cpu_brand = detect_cpu_brand();
    let cpu_type = detect_cpu_type(&cpu_brand);

    let gpu_output = runner
        .run("lspci", &["-nn"])
        .ok()
        .map(|result| result.stdout)
        .unwrap_or_default();
    let (gpu_type, gpu_brand) = detect_gpu(&gpu_output);

    let boot_type = if fs::metadata("/sys/firmware/efi/efivars").is_ok() {
        BootType::SystemdBoot
    } else {
        BootType::Grub
    };

    HardwareInfo {
        cpu_brand,
        gpu_brand,
        cpu_type,
        gpu_type,
        boot_type,
    }
}

fn detect_cpu_brand() -> String {
    let cpuinfo = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    cpuinfo
        .lines()
        .find_map(|line| {
            let (key, value) = line.split_once(':')?;
            if key.trim() == "model name" {
                Some(value.trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn detect_cpu_type(cpu_brand: &str) -> CpuType {
    let arch = std::env::consts::ARCH;
    if arch == "aarch64" {
        return CpuType::Aarch64;
    }

    if cpu_brand.to_lowercase().contains("intel") {
        CpuType::Intel
    } else {
        CpuType::Amd
    }
}

fn detect_gpu(output: &str) -> (GpuType, String) {
    let mut lines: Vec<String> = output
        .lines()
        .filter(|line| {
            let lower = line.to_lowercase();
            lower.contains("vga") || lower.contains("3d") || lower.contains("display")
        })
        .map(|line| line.trim().to_string())
        .collect();

    if lines.is_empty() {
        return (GpuType::None, "not detected".to_string());
    }

    let first = lines.remove(0);
    let lower = first.to_lowercase();
    let vm_patterns = [
        "qxl",
        "virtio",
        "vmware",
        "virtualbox",
        "bochs",
        "red hat",
        "paravirtual",
    ];

    if vm_patterns.iter().any(|pattern| lower.contains(pattern)) {
        return (GpuType::None, first);
    }
    if lower.contains("nvidia") {
        return (GpuType::Nvidia, first);
    }
    if lower.contains("amd") || lower.contains("ati") {
        return (GpuType::Amd, first);
    }
    if lower.contains("intel") {
        return (GpuType::Intel, first);
    }

    (GpuType::None, first)
}
