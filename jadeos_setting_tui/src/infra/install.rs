use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    config::{BootType, DEFAULT_REPOSITORY_URL, InstallConfig, UserConfig},
    infra::command_runner::{CommandRunner, SystemCommandRunner},
};

const MOUNT_ROOT: &str = "/mnt";

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
    ensure_running_as_root(runner)?;
    ensure_target_disk_is_not_mounted(config, runner)?;

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
    prepare_configuration_repository(config, logs, runner)?;
    generate_hardware_configuration(config, logs, runner)?;
    track_repository(config, logs, runner)?;
    run_nixos_install(config, logs, runner)?;

    logs.push("install complete: phase3 finished".to_string());
    Ok(())
}

fn ensure_running_as_root<R: CommandRunner>(runner: &R) -> Result<(), String> {
    let output = runner
        .run("id", &["-u"])
        .map_err(|error| format!("preflight: failed to check current user id: {error}"))?;
    if output.exit_code != 0 {
        return Err(format!(
            "preflight: `id -u` failed: {}",
            output.stderr.trim()
        ));
    }
    if output.stdout.trim() != "0" {
        return Err("preflight: installer must be run as root. Start it with sudo.".to_string());
    }
    Ok(())
}

fn ensure_target_disk_is_not_mounted<R: CommandRunner>(
    config: &InstallConfig,
    runner: &R,
) -> Result<(), String> {
    let output = runner
        .run("lsblk", &["-nr", "-o", "PKNAME,MOUNTPOINT", &config.device])
        .map_err(|error| format!("preflight: failed to inspect target disk mounts: {error}"))?;
    if output.exit_code != 0 {
        return Err(format!(
            "preflight: lsblk mount inspection failed: {}",
            output.stderr.trim()
        ));
    }

    let mounted = mounted_partitions(&output.stdout);
    if !mounted.is_empty() {
        return Err(format!(
            "preflight: target disk {} has mounted partitions: {}. Boot from installer media or unmount them first.",
            config.device,
            mounted.join(", ")
        ));
    }

    Ok(())
}

fn mounted_partitions(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
            let mut parts = trimmed.splitn(2, char::is_whitespace);
            let name = parts.next()?.trim();
            let mountpoint = parts.next().unwrap_or_default().trim();
            if mountpoint.is_empty() {
                None
            } else {
                Some(format!("{name}:{mountpoint}"))
            }
        })
        .collect()
}

fn create_partitions<R: CommandRunner>(
    config: &InstallConfig,
    logs: &mut Vec<String>,
    runner: &R,
) -> Result<(), String> {
    let root_end = root_partition_end(config);
    let swap_start = format!("-{}", config.swap_size);

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
                    &config.boot_size,
                ],
            )?;
            run_step(
                logs,
                runner,
                "partition: esp flag",
                "parted",
                &["-s", &config.device, "set", "1", "esp", "on"],
            )?;
            if config.has_swap_partition() {
                run_step(
                    logs,
                    runner,
                    "partition: root",
                    "parted",
                    &[
                        "-s",
                        &config.device,
                        "--",
                        "mkpart",
                        "nixos",
                        "ext4",
                        &config.boot_size,
                        &root_end,
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
                        "--",
                        "mkpart",
                        "swap",
                        "linux-swap",
                        &swap_start,
                        "100%",
                    ],
                )?;
            } else {
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
                        &config.boot_size,
                        &root_end,
                    ],
                )?;
            }
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
            if config.has_swap_partition() {
                run_step(
                    logs,
                    runner,
                    "partition: root",
                    "parted",
                    &[
                        "-s",
                        &config.device,
                        "--",
                        "mkpart",
                        "nixos",
                        "ext4",
                        "2MiB",
                        &root_end,
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
                        "--",
                        "mkpart",
                        "swap",
                        "linux-swap",
                        &swap_start,
                        "100%",
                    ],
                )?;
            } else {
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
                        &root_end,
                    ],
                )?;
            }
        }
    }

    Ok(())
}

fn root_partition_end(config: &InstallConfig) -> String {
    if config.has_swap_partition() {
        format!("-{}", config.swap_size)
    } else {
        "100%".to_string()
    }
}

fn format_filesystems<R: CommandRunner>(
    config: &InstallConfig,
    logs: &mut Vec<String>,
    runner: &R,
) -> Result<(), String> {
    let part_boot = partition_path(&config.device, 1);
    let part_root = partition_path(&config.device, 2);

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
    if config.has_swap_partition() {
        let part_swap = partition_path(&config.device, 3);
        run_step(
            logs,
            runner,
            "format: swap",
            "mkswap",
            &["-L", "swap", &part_swap],
        )?;
    } else {
        logs.push("format: swap skipped".to_string());
    }
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
    if config.has_swap_partition() {
        run_step(
            logs,
            runner,
            "mount: swap",
            "swapon",
            &["/dev/disk/by-label/swap"],
        )?;
    } else {
        logs.push("mount: swap skipped".to_string());
    }
    Ok(())
}

fn prepare_configuration_repository<R: CommandRunner>(
    config: &InstallConfig,
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

    let configured_root = PathBuf::from(config.repository_path.trim());
    if source_tree_available(&configured_root) {
        copy_source_tree(&configured_root, Path::new("/mnt/etc/nixos"))
            .map_err(|error| format!("copy source tree failed: {error}"))?;
        logs.push(format!(
            "repo: copied selected repository from {}",
            configured_root.display()
        ));
        return Ok(());
    }

    let source_root = source_tree_root();
    if source_tree_available(&source_root) {
        copy_source_tree(&source_root, Path::new("/mnt/etc/nixos"))
            .map_err(|error| format!("copy source tree failed: {error}"))?;
        logs.push(format!(
            "repo: copied bundled source tree from {}",
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
        &[
            "-C",
            "/mnt/etc/nixos",
            "remote",
            "add",
            "origin",
            &repository_remote_url(config),
        ],
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

fn track_repository<R: CommandRunner>(
    config: &InstallConfig,
    logs: &mut Vec<String>,
    runner: &R,
) -> Result<(), String> {
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
            &[
                "-C",
                "/mnt/etc/nixos",
                "remote",
                "add",
                "origin",
                &repository_remote_url(config),
            ],
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
                &repository_remote_url(config),
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

fn repository_remote_url(config: &InstallConfig) -> String {
    if config.repository_url.trim().is_empty() {
        DEFAULT_REPOSITORY_URL.to_string()
    } else {
        config.repository_url.trim().to_string()
    }
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

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, io};

    use super::*;
    use crate::infra::command_runner::CommandOutput;

    #[derive(Default)]
    struct RecordingRunner {
        calls: RefCell<Vec<(String, Vec<String>)>>,
    }

    impl RecordingRunner {
        fn calls(&self) -> Vec<String> {
            self.calls
                .borrow()
                .iter()
                .map(|(program, args)| format!("{program} {}", args.join(" ")))
                .collect()
        }
    }

    impl CommandRunner for RecordingRunner {
        fn run(&self, program: &str, args: &[&str]) -> io::Result<CommandOutput> {
            self.calls.borrow_mut().push((
                program.to_string(),
                args.iter().map(|arg| arg.to_string()).collect(),
            ));
            Ok(CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            })
        }
    }

    fn install_config() -> InstallConfig {
        InstallConfig {
            device: "/dev/vda".to_string(),
            ..InstallConfig::default()
        }
    }

    #[test]
    fn mounted_partitions_returns_parent_and_mountpoint_pairs() {
        let output = "\n nvme1n1 /\nnvme1n1 /boot\nnvme1n1 \n";

        assert_eq!(
            mounted_partitions(output),
            vec!["nvme1n1:/".to_string(), "nvme1n1:/boot".to_string()]
        );
    }

    #[test]
    fn swap_partition_steps_are_skipped_when_swap_is_disabled() {
        let config = install_config();
        let runner = RecordingRunner::default();
        let mut logs = Vec::new();

        create_partitions(&config, &mut logs, &runner).unwrap();
        format_filesystems(&config, &mut logs, &runner).unwrap();
        mount_filesystems(&config, &mut logs, &runner).unwrap();

        let calls = runner.calls();
        assert!(
            calls
                .iter()
                .any(|call| { call == "parted -s /dev/vda mkpart nixos ext4 512MiB 100%" })
        );
        assert!(!calls.iter().any(|call| call.contains("linux-swap")));
        assert!(!calls.iter().any(|call| call.starts_with("mkswap ")));
        assert!(!calls.iter().any(|call| call.starts_with("swapon ")));
        assert!(logs.iter().any(|line| line == "format: swap skipped"));
        assert!(logs.iter().any(|line| line == "mount: swap skipped"));
    }

    #[test]
    fn swap_partition_steps_use_tail_size_when_swap_is_enabled() {
        let mut config = install_config();
        config.swap_size = "2GiB".to_string();
        let runner = RecordingRunner::default();
        let mut logs = Vec::new();

        create_partitions(&config, &mut logs, &runner).unwrap();
        format_filesystems(&config, &mut logs, &runner).unwrap();
        mount_filesystems(&config, &mut logs, &runner).unwrap();

        let calls = runner.calls();
        assert!(
            calls
                .iter()
                .any(|call| { call == "parted -s /dev/vda -- mkpart nixos ext4 512MiB -2GiB" })
        );
        assert!(
            calls
                .iter()
                .any(|call| { call == "parted -s /dev/vda -- mkpart swap linux-swap -2GiB 100%" })
        );
        assert!(calls.iter().any(|call| call == "mkswap -L swap /dev/vda3"));
        assert!(
            calls
                .iter()
                .any(|call| call == "swapon /dev/disk/by-label/swap")
        );
    }
}
