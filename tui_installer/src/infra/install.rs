use std::{
    cell::RefCell,
    fs,
    path::{Path, PathBuf},
    sync::mpsc::Sender,
};

use crate::{
    config::{BootType, DEFAULT_REPOSITORY_URL, InstallConfig, UserConfig},
    infra::command_runner::{CommandRunner, SystemCommandRunner},
};

const MOUNT_ROOT: &str = "/mnt";

thread_local! {
    static LOG_SENDER: RefCell<Option<Sender<String>>> = const { RefCell::new(None) };
}

pub fn run_phase3_install(
    config: &InstallConfig,
    users: &[UserConfig],
    logs: &mut Vec<String>,
) -> Result<(), String> {
    let runner = SystemCommandRunner;
    run_phase3_install_with_runner(config, users, logs, &runner)
}

pub fn run_phase3_install_streaming(
    config: &InstallConfig,
    users: &[UserConfig],
    log_sender: Sender<String>,
) -> Result<(), String> {
    LOG_SENDER.with(|sender| {
        *sender.borrow_mut() = Some(log_sender);
    });

    let mut logs = Vec::new();
    let result = run_phase3_install(config, users, &mut logs);

    LOG_SENDER.with(|sender| {
        *sender.borrow_mut() = None;
    });

    result
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
    validate_install_inputs(config, users)?;
    emit_log(logs, format!("preflight: target={}", config.device));
    emit_log(logs, "preflight: checking root privileges");
    ensure_running_as_root(runner)?;
    emit_log(logs, "preflight: checking target disk mounts");
    ensure_target_disk_is_not_mounted(config, logs, runner)?;
    emit_log(logs, "preflight: checks passed");

    emit_log(
        logs,
        format!(
            "install start: host={}, target={}",
            config.hostname, config.device
        ),
    );
    emit_log(
        logs,
        format!(
            "summary: gpu={}, cpu={}, locale={}, timezone={}",
            config.gpu_type.label(),
            config.cpu_type.label(),
            config.locale,
            config.timezone
        ),
    );
    emit_log(
        logs,
        format!(
            "summary: users={}",
            users
                .iter()
                .map(|user| format!("{}({})", user.username, user.user_type.label()))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    );

    create_partitions(config, logs, runner)?;
    format_filesystems(config, logs, runner)?;
    mount_filesystems(config, logs, runner)?;
    prepare_configuration_repository(config, logs, runner)?;
    generate_hardware_configuration(config, users, logs, runner)?;
    track_repository(config, logs, runner)?;
    run_nixos_install(config, logs, runner)?;

    emit_log(logs, "install complete: phase3 finished");
    Ok(())
}

fn validate_install_inputs(config: &InstallConfig, users: &[UserConfig]) -> Result<(), String> {
    validate_hostname(&config.hostname)?;
    validate_hardware_value(
        "gpu",
        config.gpu_custom.trim().if_empty(config.gpu_type.label()),
        &["nvidia", "amd", "intel", "virtio", "none"],
    )?;
    validate_hardware_value(
        "cpu",
        config.cpu_custom.trim().if_empty(config.cpu_type.label()),
        &["amd", "intel", "aarch64"],
    )?;

    for user in users {
        validate_username(&user.username)?;
    }

    Ok(())
}

trait IfEmpty {
    fn if_empty<'a>(&'a self, fallback: &'a str) -> &'a str;
}

impl IfEmpty for str {
    fn if_empty<'a>(&'a self, fallback: &'a str) -> &'a str {
        if self.is_empty() { fallback } else { self }
    }
}

fn validate_hostname(hostname: &str) -> Result<(), String> {
    let hostname = hostname.trim();
    if hostname.is_empty() {
        return Err("Hostname is required.".to_string());
    }
    if hostname.len() > 63 {
        return Err("Hostname must be 63 characters or shorter.".to_string());
    }
    if hostname.starts_with('-') || hostname.ends_with('-') {
        return Err("Hostname must not start or end with '-'.".to_string());
    }
    if !hostname
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        return Err("Hostname must use lowercase letters, numbers, and '-'.".to_string());
    }
    Ok(())
}

fn validate_username(username: &str) -> Result<(), String> {
    let username = username.trim();
    let mut chars = username.chars();
    let Some(first) = chars.next() else {
        return Err("Username is required.".to_string());
    };
    if !(first.is_ascii_lowercase() || first == '_') {
        return Err(format!(
            "Username '{username}' must start with a lowercase letter or '_'."
        ));
    }
    if !chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_') {
        return Err(format!(
            "Username '{username}' must use lowercase letters, numbers, '-' or '_'."
        ));
    }
    Ok(())
}

fn validate_hardware_value(label: &str, value: &str, allowed: &[&str]) -> Result<(), String> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(format!(
            "Unsupported {label} value '{value}'. Allowed values: {}.",
            allowed.join(", ")
        ))
    }
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
    logs: &mut Vec<String>,
    runner: &R,
) -> Result<(), String> {
    let mounted = target_mounts(config, runner)?;
    if mounted.is_empty() {
        return Ok(());
    }

    if mounted
        .iter()
        .any(|partition| !is_installer_mountpoint(&partition.mountpoint))
    {
        return Err(format!(
            "preflight: target disk {} has mounted partitions: {}. Boot from installer media or unmount them first.",
            config.device,
            format_mounts(&mounted)
        ));
    }

    emit_log(
        logs,
        format!(
            "preflight: unmounting stale installer mounts: {}",
            format_mounts(&mounted)
        ),
    );
    let output = runner
        .run("umount", &["-R", MOUNT_ROOT])
        .map_err(|error| format!("preflight: failed to unmount {MOUNT_ROOT}: {error}"))?;
    if output.exit_code != 0 {
        return Err(format!(
            "preflight: failed to unmount stale installer mounts under {MOUNT_ROOT}: {}",
            output.stderr.trim()
        ));
    }

    let mounted = target_mounts(config, runner)?;
    if !mounted.is_empty() {
        return Err(format!(
            "preflight: target disk {} still has mounted partitions after cleanup: {}",
            config.device,
            format_mounts(&mounted)
        ));
    }
    emit_log(logs, "preflight: stale installer mounts removed");
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MountedPartition {
    name: String,
    mountpoint: String,
}

impl MountedPartition {
    fn display(&self) -> String {
        format!("{}:{}", self.name, self.mountpoint)
    }
}

fn target_mounts<R: CommandRunner>(
    config: &InstallConfig,
    runner: &R,
) -> Result<Vec<MountedPartition>, String> {
    let output = runner
        .run("lsblk", &["-nr", "-o", "NAME,MOUNTPOINT", &config.device])
        .map_err(|error| format!("preflight: failed to inspect target disk mounts: {error}"))?;
    if output.exit_code != 0 {
        return Err(format!(
            "preflight: lsblk mount inspection failed: {}",
            output.stderr.trim()
        ));
    }

    Ok(parse_mounted_partitions(&output.stdout))
}

fn parse_mounted_partitions(output: &str) -> Vec<MountedPartition> {
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
                Some(MountedPartition {
                    name: name.to_string(),
                    mountpoint: mountpoint.to_string(),
                })
            }
        })
        .collect()
}

fn is_installer_mountpoint(mountpoint: &str) -> bool {
    mountpoint == MOUNT_ROOT || mountpoint.starts_with("/mnt/")
}

fn format_mounts(mounted: &[MountedPartition]) -> String {
    mounted
        .iter()
        .map(MountedPartition::display)
        .collect::<Vec<_>>()
        .join(", ")
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
        emit_log(logs, "format: swap skipped");
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
        emit_log(logs, "mount: swap skipped");
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
        emit_log(
            logs,
            format!(
                "repo: copied selected repository from {}",
                configured_root.display()
            ),
        );
        return Ok(());
    }

    let source_root = source_tree_root();
    if source_tree_available(&source_root) {
        copy_source_tree(&source_root, Path::new("/mnt/etc/nixos"))
            .map_err(|error| format!("copy source tree failed: {error}"))?;
        emit_log(
            logs,
            format!(
                "repo: copied bundled source tree from {}",
                source_root.display()
            ),
        );
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
    users: &[UserConfig],
    logs: &mut Vec<String>,
    runner: &R,
) -> Result<(), String> {
    let hostname = config.hostname.trim();
    let host_dir = format!("/mnt/etc/nixos/nixos/{hostname}");
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
    emit_log(logs, "hardware: removed generated configuration.nix");
    generate_host_module(config, users, Path::new("/mnt/etc/nixos"))?;
    emit_log(logs, format!("host: generated module for {hostname}"));
    emit_log(
        logs,
        format!("host: applied {} user definition(s)", users.len()),
    );
    Ok(())
}

fn generate_host_module(
    config: &InstallConfig,
    users: &[UserConfig],
    repository_root: &Path,
) -> Result<(), String> {
    let hostname = config.hostname.trim();
    let host_module_dir = repository_root.join("modules").join("hosts").join(hostname);
    fs::create_dir_all(&host_module_dir)
        .map_err(|error| format!("host: failed to create generated host module dir: {error}"))?;

    fs::write(
        host_module_dir.join("flake-parts.nix"),
        host_flake_parts_content(config),
    )
    .map_err(|error| format!("host: failed to write generated flake-parts.nix: {error}"))?;

    fs::write(
        host_module_dir.join(format!("{hostname}.nix")),
        host_module_content(config, users),
    )
    .map_err(|error| format!("host: failed to write generated host module: {error}"))?;

    Ok(())
}

fn host_flake_parts_content(config: &InstallConfig) -> String {
    format!(
        "{{ inputs, ... }}:\n{{\n  flake.nixosConfigurations = inputs.self.lib.mkNixos {} {};\n}}\n",
        nix_string(nixos_system(config)),
        nix_string(config.hostname.trim()),
    )
}

fn host_module_content(config: &InstallConfig, users: &[UserConfig]) -> String {
    let hostname = config.hostname.trim();
    let imports = host_imports(users);
    let import_lines = imports
        .iter()
        .map(|module| format!("      {module}"))
        .collect::<Vec<_>>()
        .join("\n");
    let password_lines = users
        .iter()
        .map(user_password_assignment)
        .collect::<Vec<_>>()
        .join("");
    let custom_user_lines = users
        .iter()
        .filter(|user| preset_user_module(user).is_none())
        .map(custom_user_definition)
        .collect::<Vec<_>>()
        .join("");
    let custom_gui_users = users
        .iter()
        .filter(|user| {
            preset_user_module(user).is_none() && user.user_type == crate::config::UserType::Gui
        })
        .map(|user| nix_string(user.username.trim()))
        .collect::<Vec<_>>()
        .join(" ");

    format!(
        "{{ inputs, ... }}:\n{{\n  flake.modules.nixos.{} = {{ lib, pkgs, ... }}: {{\n    imports = with inputs.self.modules.nixos; [\n{}\n    ] ++ [ \"${{inputs.self}}/nixos/{}/hardware-configuration.nix\" ];\n\n    networking.hostName = {};\n    time.timeZone = {};\n    i18n.defaultLocale = {};\n    i18n.extraLocaleSettings = {{\n      LC_ADDRESS = {};\n      LC_IDENTIFICATION = {};\n      LC_MEASUREMENT = {};\n      LC_MONETARY = {};\n      LC_NAME = {};\n      LC_NUMERIC = {};\n      LC_PAPER = {};\n      LC_TELEPHONE = {};\n      LC_TIME = {};\n    }};\n    console.keyMap = {};\n\n    my.hardware.gpu = lib.mkDefault {};\n    my.hardware.cpu = lib.mkDefault {};\n    my.hardware.storage.enable = {};\n    my.installDisk = {{\n      boot = {};\n      root = {};\n      swap = {};\n    }};\n\n{}    services.openssh.enable = {};\n{}{}{}  }};\n}}\n",
        nix_attr(hostname),
        import_lines,
        hostname,
        nix_string(hostname),
        nix_string(config.timezone.trim()),
        nix_string(config.locale.trim()),
        nix_string(config.locale.trim()),
        nix_string(config.locale.trim()),
        nix_string(config.locale.trim()),
        nix_string(config.locale.trim()),
        nix_string(config.locale.trim()),
        nix_string(config.locale.trim()),
        nix_string(config.locale.trim()),
        nix_string(config.locale.trim()),
        nix_string(config.locale.trim()),
        nix_string(config.keyboard.trim()),
        nix_string(gpu_value(config)),
        nix_string(cpu_value(config)),
        nix_bool(config.storage_enabled),
        nix_string(boot_partition_device(config)),
        nix_string("/dev/disk/by-label/nixos"),
        nix_string(swap_partition_device(config)),
        boot_loader_content(config),
        nix_bool(config.ssh_enabled),
        password_lines,
        custom_desktop_content(&custom_gui_users),
        custom_user_lines,
    )
}

fn host_imports(users: &[UserConfig]) -> Vec<&'static str> {
    let mut imports = vec!["base", "home-manager"];
    if users.iter().any(|user| {
        preset_user_module(user).is_none() && user.user_type == crate::config::UserType::Gui
    }) {
        imports.push("hyprland");
    }
    for user in users {
        if let Some(module) = preset_user_module(user)
            && !imports.contains(&module)
        {
            imports.push(module);
        }
    }
    imports
}

fn preset_user_module(user: &UserConfig) -> Option<&'static str> {
    if !user.is_preset {
        return None;
    }
    match user.username.trim() {
        "jade-core" => Some("jade-core"),
        "jade-office" => Some("jade-office"),
        "jade-gaming" => Some("jade-gaming"),
        "jade-develop" => Some("jade-develop"),
        "jade-full" => Some("jade-full"),
        _ => None,
    }
}

fn custom_desktop_content(custom_gui_users: &str) -> String {
    if custom_gui_users.is_empty() {
        String::new()
    } else {
        format!("    my.desktop.hyprlandUsers = [ {custom_gui_users} ];\n")
    }
}

fn custom_user_definition(user: &UserConfig) -> String {
    let username = user.username.trim();
    format!(
        "\n    users.users.{} = {{\n      isNormalUser = true;\n      description = {};\n      extraGroups = [\n        \"wheel\"\n        \"networkmanager\"\n        \"audio\"\n        \"video\"\n        \"input\"\n        \"seat\"\n      ];\n      shell = pkgs.zsh;\n    }};\n\n    programs.zsh.enable = true;\n\n    home-manager.users.{} = {{\n      imports = with inputs.self.modules.homeManager; [\n{}\n      ];\n\n      my.capabilities = {{\n        user_interface = {};\n        window_manager = {};\n      }};\n\n      home.username = {};\n      home.homeDirectory = {};\n      home.stateVersion = \"25.05\";\n\n      programs.home-manager.enable = true;\n    }};\n",
        nix_attr(username),
        nix_string(user.display_name.trim()),
        nix_attr(username),
        home_manager_import_lines(user),
        nix_string(home_user_interface(user)),
        nix_string(home_window_manager(user)),
        nix_string(username),
        nix_string(&format!("/home/{username}")),
    )
}

fn user_password_assignment(user: &UserConfig) -> String {
    format!(
        "    users.users.{}.hashedPassword = {};\n",
        nix_attr(user.username.trim()),
        nix_string(user.password_hash.trim()),
    )
}

fn home_manager_import_lines(user: &UserConfig) -> String {
    custom_home_imports(user)
        .iter()
        .map(|module| format!("        {module}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn custom_home_imports(user: &UserConfig) -> Vec<&'static str> {
    let mut imports = vec!["base"];
    if user.user_type == crate::config::UserType::Gui {
        imports.push("hyprland");
    }
    for program in &user.programs {
        if let Some(module) = home_manager_module(program)
            && !imports.contains(&module)
        {
            imports.push(module);
        }
    }
    imports
}

fn home_manager_module(program: &str) -> Option<&'static str> {
    match program {
        "browser" => Some("browser"),
        "office" => Some("office"),
        "media" => Some("media"),
        "sns" => Some("sns"),
        "programming" => Some("programming"),
        "gaming" => Some("gaming"),
        "electronics" => Some("electronics"),
        "mechanical" => Some("mechanical"),
        _ => None,
    }
}

fn home_user_interface(user: &UserConfig) -> &'static str {
    match user.user_type {
        crate::config::UserType::Gui => "gui",
        crate::config::UserType::Tui | crate::config::UserType::Cui => "tui",
    }
}

fn home_window_manager(user: &UserConfig) -> &'static str {
    match user.user_type {
        crate::config::UserType::Gui => "hyprland",
        crate::config::UserType::Tui | crate::config::UserType::Cui => "none",
    }
}

fn boot_loader_content(config: &InstallConfig) -> String {
    match config.boot_type {
        BootType::SystemdBoot => {
            "    boot.loader.systemd-boot.enable = true;\n    boot.loader.efi.canTouchEfiVariables = true;\n"
                .to_string()
        }
        BootType::Grub => format!(
            "    boot.loader.grub.enable = true;\n    boot.loader.grub.device = {};\n",
            nix_string(config.device.trim()),
        ),
    }
}

fn boot_partition_device(config: &InstallConfig) -> &'static str {
    match config.boot_type {
        BootType::SystemdBoot => "/dev/disk/by-label/boot",
        BootType::Grub => "",
    }
}

fn swap_partition_device(config: &InstallConfig) -> &'static str {
    if config.has_swap_partition() {
        "/dev/disk/by-label/swap"
    } else {
        ""
    }
}

fn gpu_value(config: &InstallConfig) -> &str {
    config.gpu_custom.trim().if_empty(config.gpu_type.label())
}

fn cpu_value(config: &InstallConfig) -> &str {
    config.cpu_custom.trim().if_empty(config.cpu_type.label())
}

fn nixos_system(config: &InstallConfig) -> &'static str {
    if cpu_value(config) == "aarch64" {
        "aarch64-linux"
    } else {
        "x86_64-linux"
    }
}

fn nix_bool(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn nix_attr(value: &str) -> String {
    nix_string(value)
}

fn nix_string(value: &str) -> String {
    format!("\"{}\"", escape_nix_string(value))
}

fn escape_nix_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
        .replace("${", "\\${")
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
        emit_log(logs, "git: remote add done");
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
    let flake = format!("path:/mnt/etc/nixos#{}", config.hostname.trim());
    run_step(
        logs,
        runner,
        "install: nixos-install",
        "nixos-install",
        &["--flake", &flake, "--no-root-passwd"],
    )?;
    emit_log(logs, "warning: post-install repo sync is not ported yet");
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
    emit_log(logs, format!("{label}..."));
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
        emit_log(logs, format!("{label} output: {}", output.stdout.trim()));
    }
    if !output.stderr.trim().is_empty() {
        emit_log(logs, format!("{label} stderr: {}", output.stderr.trim()));
    }
    emit_log(logs, format!("{label} done"));
    Ok(())
}

fn emit_log(logs: &mut Vec<String>, line: impl Into<String>) {
    let line = line.into();
    LOG_SENDER.with(|sender| {
        if let Some(sender) = sender.borrow().as_ref() {
            let _ = sender.send(line.clone());
        }
    });
    logs.push(line);
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
    use std::{cell::RefCell, collections::VecDeque, io};

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

    #[derive(Default)]
    struct ScriptedRunner {
        calls: RefCell<Vec<(String, Vec<String>)>>,
        outputs: RefCell<VecDeque<CommandOutput>>,
    }

    impl ScriptedRunner {
        fn with_outputs(outputs: Vec<CommandOutput>) -> Self {
            Self {
                calls: RefCell::new(Vec::new()),
                outputs: RefCell::new(VecDeque::from(outputs)),
            }
        }

        fn calls(&self) -> Vec<String> {
            self.calls
                .borrow()
                .iter()
                .map(|(program, args)| format!("{program} {}", args.join(" ")))
                .collect()
        }
    }

    impl CommandRunner for ScriptedRunner {
        fn run(&self, program: &str, args: &[&str]) -> io::Result<CommandOutput> {
            self.calls.borrow_mut().push((
                program.to_string(),
                args.iter().map(|arg| arg.to_string()).collect(),
            ));
            Ok(self
                .outputs
                .borrow_mut()
                .pop_front()
                .unwrap_or(CommandOutput {
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: 0,
                }))
        }
    }

    fn install_config() -> InstallConfig {
        InstallConfig {
            device: "/dev/vda".to_string(),
            ..InstallConfig::default()
        }
    }

    fn user_config(
        username: &str,
        user_type: crate::config::UserType,
        is_preset: bool,
    ) -> UserConfig {
        UserConfig {
            username: username.to_string(),
            display_name: username.to_string(),
            user_type,
            programs: vec!["browser".to_string(), "programming".to_string()],
            password_hash: "$y$j9T$testhash".to_string(),
            is_preset,
        }
    }

    #[test]
    fn generated_host_flake_exposes_configured_hostname() {
        let mut config = install_config();
        config.hostname = "jadeos".to_string();

        let content = host_flake_parts_content(&config);

        assert!(content.contains("inputs.self.lib.mkNixos \"x86_64-linux\" \"jadeos\""));
    }

    #[test]
    fn generated_host_module_registers_users_and_install_settings() {
        let mut config = install_config();
        config.hostname = "jadeos".to_string();
        config.gpu_type = crate::config::GpuType::Nvidia;
        config.storage_enabled = true;
        config.ssh_enabled = true;
        let users = vec![
            user_config("jade-core", crate::config::UserType::Cui, true),
            user_config("alice", crate::config::UserType::Gui, false),
        ];

        let content = host_module_content(&config, &users);

        assert!(content.contains("flake.modules.nixos.\"jadeos\""));
        assert!(content.contains("      jade-core"));
        assert!(content.contains("users.users.\"jade-core\".hashedPassword"));
        assert!(content.contains("users.users.\"alice\" = {"));
        assert!(content.contains("my.desktop.hyprlandUsers = [ \"alice\" ];"));
        assert!(content.contains("my.hardware.gpu = lib.mkDefault \"nvidia\";"));
        assert!(content.contains("my.hardware.storage.enable = true;"));
        assert!(content.contains("services.openssh.enable = true;"));
        assert!(content.contains("boot.loader.systemd-boot.enable = true;"));
        assert!(!content.contains("cli-tools"));
    }

    #[test]
    fn nixos_install_uses_path_flake_reference() {
        let mut config = install_config();
        config.hostname = "jadeos".to_string();
        let runner = RecordingRunner::default();
        let mut logs = Vec::new();

        run_nixos_install(&config, &mut logs, &runner).unwrap();

        assert!(runner.calls().iter().any(
            |call| call == "nixos-install --flake path:/mnt/etc/nixos#jadeos --no-root-passwd"
        ));
    }

    #[test]
    fn parse_mounted_partitions_returns_name_and_mountpoint_pairs() {
        let output = "\n nvme1n1p1 /\nnvme1n1p2 /boot\nnvme1n1 \n";

        assert_eq!(
            parse_mounted_partitions(output)
                .iter()
                .map(MountedPartition::display)
                .collect::<Vec<_>>(),
            vec!["nvme1n1p1:/".to_string(), "nvme1n1p2:/boot".to_string()]
        );
    }

    #[test]
    fn installer_mounts_under_mnt_are_cleaned_before_preflight_check() {
        let config = install_config();
        let runner = ScriptedRunner::with_outputs(vec![
            CommandOutput {
                stdout: "vda \nvda1 /mnt/boot\nvda2 /mnt\n".to_string(),
                stderr: String::new(),
                exit_code: 0,
            },
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            },
            CommandOutput {
                stdout: "vda \nvda1 \nvda2 \n".to_string(),
                stderr: String::new(),
                exit_code: 0,
            },
        ]);
        let mut logs = Vec::new();

        ensure_target_disk_is_not_mounted(&config, &mut logs, &runner).unwrap();

        let calls = runner.calls();
        assert!(calls.iter().any(|call| call == "umount -R /mnt"));
        assert!(
            logs.iter()
                .any(|line| line == "preflight: stale installer mounts removed")
        );
    }

    #[test]
    fn target_mounts_outside_mnt_are_rejected() {
        let config = install_config();
        let runner = ScriptedRunner::with_outputs(vec![CommandOutput {
            stdout: "vda \nvda1 /boot\nvda2 /\n".to_string(),
            stderr: String::new(),
            exit_code: 0,
        }]);
        let mut logs = Vec::new();

        let error = ensure_target_disk_is_not_mounted(&config, &mut logs, &runner).unwrap_err();

        assert!(error.contains("Boot from installer media"));
        assert!(!runner.calls().iter().any(|call| call == "umount -R /mnt"));
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
