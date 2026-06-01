use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
};

use crate::app::{App, Screen, UserMenuChoice};
use crate::components::form::{FormField, FormFieldRole, FormSection};
use crate::config::UserType;
use crate::logic::setup::{
    BOOT_TYPE_OPTIONS, CPU_OPTIONS, GPU_OPTIONS, KEYBOARD_OPTIONS, LOCALE_OPTIONS, TIMEZONE_OPTIONS,
};
use crate::logic::user_programs::program_options_for;

pub fn main_panel_text(app: &App) -> Text<'static> {
    Text::from(vec![
        info_line("phase", app.screen.phase()),
        info_line("screen", app.screen.title()),
    ])
}

pub fn screen_form_section(app: &App) -> FormSection {
    let active = app.active_field_for_current_screen();
    let mut section = match app.screen {
        Screen::Welcome => form_section(
            "welcome",
            vec![
                form_field(
                    "status",
                    app.user_message
                        .clone()
                        .unwrap_or_else(|| "installer bootstrap ready".to_string()),
                    Some("Press Enter to run network check".to_string()),
                    FormFieldRole::ReadOnly,
                ),
                form_field(
                    "mode",
                    "interactive".to_string(),
                    None,
                    FormFieldRole::ReadOnly,
                ),
            ],
            None,
        ),
        Screen::GitHubLogin => form_section(
            "github",
            vec![
                form_field(
                    "repository",
                    app.config.repository.clone(),
                    Some(
                        "owner/name, repo name, URL, or empty for nixos_configuration".to_string(),
                    ),
                    FormFieldRole::Text,
                ),
                form_field(
                    "clone path",
                    app.config.repository_path.clone(),
                    Some("Working clone used before copying to /mnt/etc/nixos".to_string()),
                    FormFieldRole::Text,
                ),
                form_field(
                    "github user",
                    app.config.github_username.clone(),
                    Some("Only required when repository is empty or repo name only".to_string()),
                    FormFieldRole::ReadOnly,
                ),
                form_field(
                    "status",
                    app.user_message.clone().unwrap_or_else(|| {
                        "Press Enter to clone; gh is optional for full URLs".to_string()
                    }),
                    None,
                    FormFieldRole::ReadOnly,
                ),
            ],
            Some(active),
        ),
        Screen::HardwareDetect => form_section(
            "hardware",
            vec![
                form_field(
                    "detected cpu",
                    app.hardware.cpu_brand.clone(),
                    Some(format!("cpu type: {}", app.hardware.cpu_type.label())),
                    FormFieldRole::ReadOnly,
                ),
                form_field(
                    "detected gpu",
                    app.hardware.gpu_brand.clone(),
                    Some(format!("gpu type: {}", app.hardware.gpu_type.label())),
                    FormFieldRole::ReadOnly,
                ),
                form_field(
                    "boot type",
                    app.hardware.boot_type.label().to_string(),
                    Some("detected from firmware".to_string()),
                    FormFieldRole::ReadOnly,
                ),
            ],
            None,
        ),
        Screen::DeviceSelect => form_section(
            "device",
            device_fields(app),
            if app.device_options.is_empty() {
                None
            } else {
                Some(0)
            },
        ),
        Screen::PartitionConfig => form_section(
            "partition",
            vec![
                form_field(
                    "boot size",
                    app.config.boot_size.clone(),
                    Some("Default: 512MiB".to_string()),
                    FormFieldRole::Text,
                ),
                form_field(
                    "swap size",
                    app.config.swap_size.clone(),
                    Some("Use 0 to disable swap; root uses remaining disk".to_string()),
                    FormFieldRole::Text,
                ),
            ],
            Some(active),
        ),
        Screen::PartitionConfirm => form_section(
            "partition confirmation",
            vec![
                form_field(
                    "target disk",
                    app.config.device.clone(),
                    Some("This disk will be repartitioned and formatted".to_string()),
                    FormFieldRole::ReadOnly,
                ),
                form_field(
                    "layout",
                    partition_layout(app),
                    Some("Root partition uses the remaining disk space".to_string()),
                    FormFieldRole::ReadOnly,
                ),
                form_field(
                    "confirmation",
                    "pending in popup".to_string(),
                    Some("Confirm destructive operation in popup".to_string()),
                    FormFieldRole::ReadOnly,
                ),
            ],
            None,
        ),
        Screen::HostnameInput => form_section(
            "hostname",
            vec![form_field(
                "hostname",
                app.config.hostname.clone(),
                Some("Use lowercase letters, numbers, and hyphen".to_string()),
                FormFieldRole::Text,
            )],
            Some(active),
        ),
        Screen::KeyboardSelect => form_section(
            "keyboard",
            vec![
                form_field(
                    "selection",
                    preset_or_custom(app, app.config.keyboard.clone()),
                    Some(format!(
                        "Space: cycle -> custom | {}",
                        KEYBOARD_OPTIONS.join(" / ")
                    )),
                    FormFieldRole::Choice,
                ),
                form_field(
                    "custom value",
                    app.config.keyboard.clone(),
                    Some("Type when selection is custom".to_string()),
                    FormFieldRole::Text,
                ),
            ],
            Some(active),
        ),
        Screen::LocaleSelect => form_section(
            "locale",
            vec![
                form_field(
                    "selection",
                    preset_or_custom(app, app.config.locale.clone()),
                    Some(format!(
                        "Space: cycle -> custom | {}",
                        LOCALE_OPTIONS.join(" / ")
                    )),
                    FormFieldRole::Choice,
                ),
                form_field(
                    "custom value",
                    app.config.locale.clone(),
                    Some("Type when selection is custom".to_string()),
                    FormFieldRole::Text,
                ),
            ],
            Some(active),
        ),
        Screen::TimezoneSelect => form_section(
            "timezone",
            vec![
                form_field(
                    "selection",
                    preset_or_custom(app, app.config.timezone.clone()),
                    Some(format!(
                        "Space: cycle -> custom | {}",
                        TIMEZONE_OPTIONS.join(" / ")
                    )),
                    FormFieldRole::Choice,
                ),
                form_field(
                    "custom value",
                    app.config.timezone.clone(),
                    Some("Type when selection is custom".to_string()),
                    FormFieldRole::Text,
                ),
            ],
            Some(active),
        ),
        Screen::SshToggle => form_section(
            "ssh",
            vec![form_field(
                "enabled",
                app.config.ssh_enabled.to_string(),
                Some("Choose true or false".to_string()),
                FormFieldRole::Toggle,
            )],
            Some(active),
        ),
        Screen::StorageToggle => form_section(
            "storage",
            vec![form_field(
                "enabled",
                app.config.storage_enabled.to_string(),
                Some("Choose true or false".to_string()),
                FormFieldRole::Toggle,
            )],
            Some(active),
        ),
        Screen::GpuSelect => form_section(
            "gpu",
            vec![
                form_field(
                    "selection",
                    if app.custom_selected_for_current_screen() {
                        "custom".to_string()
                    } else {
                        app.config.gpu_type.label().to_string()
                    },
                    Some(format!(
                        "Space: cycle -> custom | {}",
                        GPU_OPTIONS.join(" / ")
                    )),
                    FormFieldRole::Choice,
                ),
                form_field(
                    "custom value",
                    app.config.gpu_custom.clone(),
                    Some("Type when selection is custom".to_string()),
                    FormFieldRole::Text,
                ),
            ],
            Some(active),
        ),
        Screen::CpuSelect => form_section(
            "cpu",
            vec![
                form_field(
                    "selection",
                    if app.custom_selected_for_current_screen() {
                        "custom".to_string()
                    } else {
                        app.config.cpu_type.label().to_string()
                    },
                    Some(format!(
                        "Space: cycle -> custom | {}",
                        CPU_OPTIONS.join(" / ")
                    )),
                    FormFieldRole::Choice,
                ),
                form_field(
                    "custom value",
                    app.config.cpu_custom.clone(),
                    Some("Type when selection is custom".to_string()),
                    FormFieldRole::Text,
                ),
            ],
            Some(active),
        ),
        Screen::BootTypeSelect => form_section(
            "boot type",
            vec![form_field(
                "selection",
                app.config.boot_type.label().to_string(),
                Some(format!("Space: cycle | {}", BOOT_TYPE_OPTIONS.join(" / "))),
                FormFieldRole::Choice,
            )],
            Some(active),
        ),
        Screen::UserMenu => form_section("users", user_menu_fields(app), Some(active)),
        Screen::PresetUserPassword => form_section(
            "preset password",
            vec![
                form_field(
                    "status",
                    app.pending_user
                        .as_ref()
                        .map(|user| format!("Set password for {}", user.username))
                        .unwrap_or_else(|| "No pending user".to_string()),
                    Some("Use popup window for password input".to_string()),
                    FormFieldRole::ReadOnly,
                ),
                form_field(
                    "flow",
                    "preset user setup".to_string(),
                    Some("Press Enter in popup to confirm".to_string()),
                    FormFieldRole::ReadOnly,
                ),
            ],
            None,
        ),
        Screen::CustomUserBasic => {
            form_section("custom user", custom_basic_fields(app), Some(active))
        }
        Screen::CustomUserType => form_section("user type", custom_type_fields(app), Some(0)),
        Screen::CustomUserPrograms => {
            form_section("programs", custom_program_fields(app), Some(active))
        }
        Screen::CustomUserPassword => form_section(
            "custom password",
            vec![
                form_field(
                    "status",
                    app.pending_user
                        .as_ref()
                        .map(|user| format!("Set password for {}", user.username))
                        .unwrap_or_else(|| "No pending user".to_string()),
                    Some("Use popup window for password input".to_string()),
                    FormFieldRole::ReadOnly,
                ),
                form_field(
                    "flow",
                    "custom user setup".to_string(),
                    Some("Press Enter in popup to confirm".to_string()),
                    FormFieldRole::ReadOnly,
                ),
            ],
            None,
        ),
        Screen::UserAddResult => form_section(
            "user added",
            vec![
                form_field(
                    "status",
                    app.user_message
                        .clone()
                        .unwrap_or_else(|| "User added.".to_string()),
                    Some("Press Enter to return to the user menu".to_string()),
                    FormFieldRole::ReadOnly,
                ),
                form_field(
                    "configured users",
                    users_display(app),
                    None,
                    FormFieldRole::ReadOnly,
                ),
            ],
            None,
        ),
        Screen::Summary => form_section(
            "summary",
            vec![
                form_field(
                    "ready to install",
                    "check fields".to_string(),
                    Some("Review all fields before install".to_string()),
                    FormFieldRole::ReadOnly,
                ),
                form_field(
                    "selected users",
                    users_display(app),
                    None,
                    FormFieldRole::ReadOnly,
                ),
                form_field(
                    "confirmation",
                    "pending in popup".to_string(),
                    Some("Install confirmation is handled in popup".to_string()),
                    FormFieldRole::ReadOnly,
                ),
            ],
            None,
        ),
        Screen::Installing => form_section(
            "installing",
            vec![
                form_field(
                    "stage",
                    "running".to_string(),
                    Some("Running phase3 install steps".to_string()),
                    FormFieldRole::ReadOnly,
                ),
                form_field(
                    "install log",
                    install_log_tail(app, 12),
                    Some("Most recent install steps and errors".to_string()),
                    FormFieldRole::Log,
                ),
            ],
            None,
        ),
        Screen::Done => form_section(
            "done",
            vec![
                form_field(
                    "result",
                    app.user_message
                        .clone()
                        .unwrap_or_else(|| "Done".to_string()),
                    Some("Check logs before reboot".to_string()),
                    FormFieldRole::ReadOnly,
                ),
                form_field(
                    "install log",
                    install_log_tail(app, 14),
                    Some("Use this log to identify the failed step".to_string()),
                    FormFieldRole::Log,
                ),
            ],
            None,
        ),
    };
    section.text_editing = app.is_editing();
    section
}

pub fn screen_popup_section(app: &App) -> Option<FormSection> {
    let active = app.active_field_for_current_screen();
    let mut section = match app.screen {
        Screen::PresetUserPassword => Some(form_section(
            "preset password",
            password_fields(app),
            Some(active),
        )),
        Screen::CustomUserPassword => Some(form_section(
            "custom password",
            password_fields(app),
            Some(active),
        )),
        Screen::PartitionConfirm => Some(form_section(
            "destructive operation",
            vec![form_field(
                "confirm",
                app.partition_confirmation.clone(),
                Some(app.user_message.clone().unwrap_or_else(|| {
                    "Type 'yes' to accept repartitioning/formatting".to_string()
                })),
                FormFieldRole::Text,
            )],
            Some(active),
        )),
        Screen::Summary => Some(form_section(
            "install confirmation",
            vec![form_field(
                "confirm",
                app.install_confirmation.clone(),
                Some(app.user_message.clone().unwrap_or_else(|| {
                    "Type 'yes' then press Enter to start installation".to_string()
                })),
                FormFieldRole::Text,
            )],
            Some(active),
        )),
        _ => None,
    }?;
    section.text_editing = app.is_editing();
    Some(section)
}

fn info_line(label: &'static str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(label, Style::default().fg(Color::DarkGray)),
        Span::styled(": ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            value.to_string(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ])
}

fn form_section(title: &str, fields: Vec<FormField>, active_field: Option<usize>) -> FormSection {
    FormSection {
        title: title.to_string(),
        fields,
        active_field,
        text_editing: false,
    }
}

fn form_field(label: &str, value: String, hint: Option<String>, role: FormFieldRole) -> FormField {
    FormField {
        label: label.to_string(),
        value,
        hint,
        role,
    }
}

fn preset_or_custom(app: &App, value: String) -> String {
    if app.custom_selected_for_current_screen() {
        "custom".to_string()
    } else {
        value
    }
}

fn device_fields(app: &App) -> Vec<FormField> {
    if app.device_options.is_empty() {
        return vec![form_field(
            "target device",
            "not detected".to_string(),
            Some("No TYPE=disk entries were returned by lsblk".to_string()),
            FormFieldRole::ReadOnly,
        )];
    }

    let active = app.active_field_for_current_screen();
    let selected = app
        .device_options
        .get(active)
        .or_else(|| app.device_options.first())
        .expect("device options are not empty");

    vec![form_field(
        "target device",
        selected.label(),
        Some(format!(
            "{} / {} - Tab/Up/Down/Space: change, Enter: confirm",
            active + 1,
            app.device_options.len()
        )),
        FormFieldRole::Choice,
    )]
}

fn user_menu_fields(app: &App) -> Vec<FormField> {
    let mut fields = vec![
        user_menu_choice_field(app, UserMenuChoice::JadeCore, "jade-core", "TUI core user"),
        user_menu_choice_field(
            app,
            UserMenuChoice::JadeOffice,
            "jade-office",
            "KDE Plasma office user",
        ),
        user_menu_choice_field(
            app,
            UserMenuChoice::JadeGaming,
            "jade-gaming",
            "KDE Plasma gaming user",
        ),
        user_menu_choice_field(
            app,
            UserMenuChoice::JadeDevelop,
            "jade-develop",
            "Hyprland development user",
        ),
        user_menu_choice_field(
            app,
            UserMenuChoice::JadeFull,
            "jade-full",
            "Hyprland full package user",
        ),
        form_field(
            "custom",
            "Add custom user".to_string(),
            Some("Create username, display name, type, programs, and password".to_string()),
            FormFieldRole::Choice,
        ),
        form_field(
            "finish",
            if app.config.users.is_empty() {
                "blocked".to_string()
            } else {
                "ready".to_string()
            },
            Some(
                app.user_message
                    .clone()
                    .unwrap_or_else(|| "Finish user configuration".to_string()),
            ),
            FormFieldRole::Choice,
        ),
    ];

    if !app.config.users.is_empty() {
        fields.push(form_field(
            "added users",
            users_display(app),
            None,
            FormFieldRole::ReadOnly,
        ));
    }

    fields
}

fn user_menu_choice_field(
    app: &App,
    choice: UserMenuChoice,
    username: &str,
    description: &str,
) -> FormField {
    let added = app.user_exists(username);
    let value = if added {
        "added".to_string()
    } else {
        "default config".to_string()
    };
    let hint = match choice {
        UserMenuChoice::JadeCore => "preset TUI user; password required",
        UserMenuChoice::JadeOffice
        | UserMenuChoice::JadeGaming
        | UserMenuChoice::JadeDevelop
        | UserMenuChoice::JadeFull => "preset GUI user; password required",
        _ => description,
    };
    form_field(
        username,
        value,
        Some(format!("{description}; {hint}")),
        FormFieldRole::Choice,
    )
}

fn custom_basic_fields(app: &App) -> Vec<FormField> {
    let Some(user) = &app.pending_user else {
        return vec![form_field(
            "status",
            "No pending custom user".to_string(),
            None,
            FormFieldRole::ReadOnly,
        )];
    };

    vec![
        form_field(
            "username",
            user.username.clone(),
            Some(
                app.user_message
                    .clone()
                    .unwrap_or_else(|| "lowercase letters, digits, '_' or '-'".to_string()),
            ),
            FormFieldRole::Text,
        ),
        form_field(
            "display name",
            user.display_name.clone(),
            Some("Leave empty to derive from username".to_string()),
            FormFieldRole::Text,
        ),
    ]
}

fn custom_type_fields(app: &App) -> Vec<FormField> {
    let user_type = app
        .pending_user
        .as_ref()
        .map(|user| user.user_type)
        .unwrap_or(UserType::Gui);
    let active = app.active_field_for_current_screen();

    vec![form_field(
        "user type",
        user_type.label().to_string(),
        Some(format!(
            "{} / 3 - Tab/Up/Down/Space: change, Enter: confirm",
            active + 1
        )),
        FormFieldRole::Choice,
    )]
}

fn custom_program_fields(app: &App) -> Vec<FormField> {
    let Some(user) = &app.pending_user else {
        return vec![form_field(
            "status",
            "No pending custom user".to_string(),
            None,
            FormFieldRole::ReadOnly,
        )];
    };

    program_options_for(user.user_type)
        .into_iter()
        .map(|(name, description)| {
            let selected = user.programs.iter().any(|program| program == name);
            form_field(
                name,
                if selected { "[x]" } else { "[ ]" }.to_string(),
                Some(description.to_string()),
                FormFieldRole::Toggle,
            )
        })
        .collect()
}

fn password_fields(app: &App) -> Vec<FormField> {
    let Some(user) = &app.pending_user else {
        return vec![form_field(
            "status",
            "No pending user".to_string(),
            None,
            FormFieldRole::ReadOnly,
        )];
    };

    vec![
        form_field(
            "password",
            mask(&user.password),
            Some(
                app.user_message
                    .clone()
                    .unwrap_or_else(|| format!("Password for {}", user.username)),
            ),
            FormFieldRole::Text,
        ),
        form_field(
            "confirm",
            mask(&user.password_confirm),
            Some("Must match password".to_string()),
            FormFieldRole::Text,
        ),
    ]
}

fn mask(value: &str) -> String {
    if value.is_empty() {
        String::new()
    } else {
        "*".repeat(value.chars().count())
    }
}

fn users_display(app: &App) -> String {
    if app.config.users.is_empty() {
        "none".to_string()
    } else {
        app.config
            .users
            .iter()
            .map(|user| format!("{}({})", user.username, user.user_type.label()))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn partition_layout(app: &App) -> String {
    if app.config.has_swap_partition() {
        format!(
            "boot={} / swap={} / root=remaining",
            app.config.boot_size, app.config.swap_size
        )
    } else {
        format!(
            "boot={} / swap=disabled / root=remaining",
            app.config.boot_size
        )
    }
}

fn install_log_tail(app: &App, lines: usize) -> String {
    if app.install_log.is_empty() {
        return "no logs yet".to_string();
    }

    let start = app.install_log.len().saturating_sub(lines);
    app.install_log[start..].join("\n")
}
