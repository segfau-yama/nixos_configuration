use std::fs;

use crate::config::{BootType, CpuType, GpuType, InstallConfig};
use crate::infra::command_runner::{CommandRunner, SystemCommandRunner};

pub const KEYBOARD_OPTIONS: &[&str] = &["jp106", "us", "de", "fr"];
pub const GPU_OPTIONS: &[&str] = &["none", "nvidia", "amd", "intel"];
pub const CPU_OPTIONS: &[&str] = &["amd", "intel", "aarch64"];
pub const LOCALE_OPTIONS: &[&str] = &["ja_JP.UTF-8", "en_US.UTF-8", "zh_CN.UTF-8", "ko_KR.UTF-8"];
pub const TIMEZONE_OPTIONS: &[&str] = &[
    "Asia/Tokyo",
    "UTC",
    "America/New_York",
    "America/Los_Angeles",
    "Europe/London",
    "Europe/Berlin",
];

#[derive(Debug, Clone)]
pub struct HardwareInfo {
    pub cpu_brand: String,
    pub gpu_brand: String,
    pub cpu_type: CpuType,
    pub gpu_type: GpuType,
    pub boot_type: BootType,
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
