use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    config::{BootType, InstallConfig, UserConfig},
    infra::command_runner::{CommandRunner, SystemCommandRunner},
};

const MOUNT_ROOT: &str = "/mnt";
const REPO_URL: &str = "https://github.com/segfau-yama/nixos_configuration.git";

pub fn run_phase3_install(
    config: &InstallConfig,
    users: &[UserConfig],
    logs: &mut Vec<String>,
) -> Result<(), String> {
    let runner = SystemCommandRunner;
    run_phase3_install_with_runner(config, users, logs, &runner)
}

fn run_phase3_install_with_runner<R: CommandRunner>(
    config: &InstallConfig,
    users: &[UserConfig],
    logs: &mut Vec<String>,
    runner: &R,
) -> Result<(), String> {
    if config.device.trim().is_empty() {
        return Err("Install target device is empty.".to_string());
    }
    if users.is_empty() {
        return Err("No interactive users configured.".to_string());
    }

    logs.push(format!(
        "install start: host={}, target={}",
        config.hostname, config.device
    ));
    logs.push(format!(
        "summary: gpu={}, cpu={}, locale={}, timezone={}",
        config.gpu_type.label(),
        config.cpu_type.label(),
        config.locale,
        config.timezone
    ));
    logs.push(format!(
        "summary: users={}",
        users
            .iter()
            .map(|user| format!("{}({})", user.username, user.user_type.label()))
            .collect::<Vec<_>>()
            .join(", ")
    ));

    create_partitions(config, logs, runner)?;
    format_filesystems(config, logs, runner)?;
    mount_filesystems(config, logs, runner)?;
    prepare_configuration_repository(logs, runner)?;
    generate_hardware_configuration(config, logs, runner)?;
    track_repository(logs, runner)?;
    run_nixos_install(config, logs, runner)?;

    logs.push("install complete: phase3 finished".to_string());
    Ok(())
}

fn create_partitions<R: CommandRunner>(
    config: &InstallConfig,
    logs: &mut Vec<String>,
    runner: &R,
) -> Result<(), String> {
    run_step(
        logs,
        runner,
        "partition: mklabel",
        "parted",
        &["-s", &config.device, "mklabel", "gpt"],
    )?;

    match config.boot_type {
        BootType::SystemdBoot => {
            run_step(
                logs,
                runner,
                "partition: esp",
                "parted",
                &[
                    "-s",
                    &config.device,
                    "mkpart",
                    "ESP",
                    "fat32",
                    "1MiB",
                    &config.boot_end,
                ],
            )?;
            run_step(
                logs,
                runner,
                "partition: esp flag",
                "parted",
                &["-s", &config.device, "set", "1", "esp", "on"],
            )?;
            run_step(
                logs,
                runner,
                "partition: root",
                "parted",
                &[
                    "-s",
                    &config.device,
                    "mkpart",
                    "nixos",
                    "ext4",
                    &config.boot_end,
                    &config.root_end,
                ],
            )?;
            run_step(
                logs,
                runner,
                "partition: swap",
                "parted",
                &[
                    "-s",
                    &config.device,
                    "mkpart",
                    "swap",
                    "linux-swap",
                    &config.root_end,
                    "100%",
                ],
            )?;
        }
        BootType::Grub => {
            run_step(
                logs,
                runner,
                "partition: bios grub",
                "parted",
                &["-s", &config.device, "mkpart", "grub", "1MiB", "2MiB"],
            )?;
            run_step(
                logs,
                runner,
                "partition: bios flag",
                "parted",
                &["-s", &config.device, "set", "1", "bios_grub", "on"],
            )?;
            run_step(
                logs,
                runner,
                "partition: root",
                "parted",
                &[
                    "-s",
                    &config.device,
                    "mkpart",
                    "nixos",
                    "ext4",
                    "2MiB",
                    &config.root_end,
                ],
            )?;
            run_step(
                logs,
                runner,
                "partition: swap",
                "parted",
                &[
                    "-s",
                    &config.device,
                    "mkpart",
                    "swap",
                    "linux-swap",
                    &config.root_end,
                    "100%",
                ],
            )?;
        }
    }

    Ok(())
}

fn format_filesystems<R: CommandRunner>(
    config: &InstallConfig,
    logs: &mut Vec<String>,
    runner: &R,
) -> Result<(), String> {
    let part_boot = partition_path(&config.device, 1);
    let part_root = partition_path(&config.device, 2);
    let part_swap = partition_path(&config.device, 3);

    if config.boot_type == BootType::SystemdBoot {
        run_step(
            logs,
            runner,
            "format: boot",
            "mkfs.fat",
            &["-F", "32", "-n", "boot", &part_boot],
        )?;
    }
    run_step(
        logs,
        runner,
        "format: root",
        "mkfs.ext4",
        &["-L", "nixos", "-F", &part_root],
    )?;
    run_step(
        logs,
        runner,
        "format: swap",
        "mkswap",
        &["-L", "swap", &part_swap],
    )?;
    run_step(
        logs,
        runner,
        "udev: trigger",
        "udevadm",
        &["trigger", "--action=add"],
    )?;
    run_step(logs, runner, "udev: settle", "udevadm", &["settle"])?;
    Ok(())
}

fn mount_filesystems<R: CommandRunner>(
    config: &InstallConfig,
    logs: &mut Vec<String>,
    runner: &R,
) -> Result<(), String> {
    run_step(
        logs,
        runner,
        "mount: root",
        "mount",
        &["/dev/disk/by-label/nixos", MOUNT_ROOT],
    )?;
    if config.boot_type == BootType::SystemdBoot {
        run_step(
            logs,
            runner,
            "mount: mkdir boot",
            "mkdir",
            &["-p", "/mnt/boot"],
        )?;
        run_step(
            logs,
            runner,
            "mount: boot",
            "mount",
            &["/dev/disk/by-label/boot", "/mnt/boot"],
        )?;
    }
    run_step(
        logs,
        runner,
        "mount: swap",
        "swapon",
        &["/dev/disk/by-label/swap"],
    )?;
    Ok(())
}

fn prepare_configuration_repository<R: CommandRunner>(
    logs: &mut Vec<String>,
    runner: &R,
) -> Result<(), String> {
    run_step(
        logs,
        runner,
        "repo: mkdir",
        "mkdir",
        &["-p", "/mnt/etc/nixos"],
    )?;

    let source_root = source_tree_root();
    if source_tree_available(&source_root) {
        copy_source_tree(&source_root, Path::new("/mnt/etc/nixos"))
            .map_err(|error| format!("copy source tree failed: {error}"))?;
        logs.push(format!(
            "repo: copied source tree from {}",
            source_root.display()
        ));
        return Ok(());
    }

    run_step(
        logs,
        runner,
        "repo: git init",
        "git",
        &["-C", "/mnt/etc/nixos", "init"],
    )?;
    run_step(
        logs,
        runner,
        "repo: git remote add",
        "git",
        &["-C", "/mnt/etc/nixos", "remote", "add", "origin", REPO_URL],
    )?;
    run_step(
        logs,
        runner,
        "repo: git fetch",
        "git",
        &["-C", "/mnt/etc/nixos", "fetch", "origin"],
    )?;
    run_step(
        logs,
        runner,
        "repo: git checkout",
        "git",
        &["-C", "/mnt/etc/nixos", "checkout", "-t", "origin/main"],
    )?;
    Ok(())
}

fn generate_hardware_configuration<R: CommandRunner>(
    config: &InstallConfig,
    logs: &mut Vec<String>,
    runner: &R,
) -> Result<(), String> {
    let host_dir = format!("/mnt/etc/nixos/nixos/{}", config.hostname);
    run_step(logs, runner, "hardware: mkdir", "mkdir", &["-p", &host_dir])?;
    run_step(
        logs,
        runner,
        "hardware: nixos-generate-config",
        "nixos-generate-config",
        &["--root", MOUNT_ROOT, "--dir", &host_dir],
    )?;
    let generated_config = format!("{host_dir}/configuration.nix");
    let _ = fs::remove_file(&generated_config);
    logs.push("hardware: removed generated configuration.nix".to_string());
    logs.push("warning: host/user module generation is not ported yet".to_string());
    Ok(())
}

fn track_repository<R: CommandRunner>(logs: &mut Vec<String>, runner: &R) -> Result<(), String> {
    run_step(
        logs,
        runner,
        "git: init if missing",
        "git",
        &["-C", "/mnt/etc/nixos", "init"],
    )?;
    let remote_add = runner
        .run(
            "git",
            &["-C", "/mnt/etc/nixos", "remote", "add", "origin", REPO_URL],
        )
        .map_err(|error| format!("git: ensure remote failed: {error}"))?;
    if remote_add.exit_code != 0 {
        run_step(
            logs,
            runner,
            "git: remote set-url",
            "git",
            &[
                "-C",
                "/mnt/etc/nixos",
                "remote",
                "set-url",
                "origin",
                REPO_URL,
            ],
        )?;
    } else {
        logs.push("git: remote add done".to_string());
    }
    run_step(
        logs,
        runner,
        "git: add",
        "git",
        &["-C", "/mnt/etc/nixos", "add", "."],
    )?;
    Ok(())
}

fn run_nixos_install<R: CommandRunner>(
    config: &InstallConfig,
    logs: &mut Vec<String>,
    runner: &R,
) -> Result<(), String> {
    let flake = format!("/mnt/etc/nixos#{}", config.hostname);
    run_step(
        logs,
        runner,
        "install: nixos-install",
        "nixos-install",
        &["--flake", &flake],
    )?;
    logs.push("warning: post-install repo sync is not ported yet".to_string());
    Ok(())
}

fn run_step<R: CommandRunner>(
    logs: &mut Vec<String>,
    runner: &R,
    label: &str,
    program: &str,
    args: &[&str],
) -> Result<(), String> {
    logs.push(format!("{label}..."));
    let output = runner
        .run(program, args)
        .map_err(|error| format!("{label} failed: {error}"))?;
    if output.exit_code != 0 {
        return Err(format!(
            "{label} failed with exit code {}: {}",
            output.exit_code,
            output.stderr.trim()
        ));
    }
    if !output.stdout.trim().is_empty() {
        logs.push(format!("{label} output: {}", output.stdout.trim()));
    }
    logs.push(format!("{label} done"));
    Ok(())
}

fn partition_path(device: &str, index: u8) -> String {
    if device.starts_with("/dev/nvme") || device.starts_with("/dev/mmcblk") {
        format!("{device}p{index}")
    } else {
        format!("{device}{index}")
    }
}

fn source_tree_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

fn source_tree_available(root: &Path) -> bool {
    root.join("flake.nix").is_file() && root.join("modules").is_dir()
}

fn copy_source_tree(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        if name.to_string_lossy() == "target" && src.ends_with("jadeos_setting_tui") {
            continue;
        }

        let dest_path = dst.join(&name);
        if path.is_dir() {
            copy_source_tree(&path, &dest_path)?;
        } else {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}
