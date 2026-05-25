use crate::{
    app::{App, Screen},
    config::{CpuType, GpuType},
    logic::setup::{CPU_OPTIONS, GPU_OPTIONS, KEYBOARD_OPTIONS, LOCALE_OPTIONS, TIMEZONE_OPTIONS},
    update::user_flow,
};

pub(crate) fn move_active_field(app: &mut App, direction: isize) {
    let count = editable_field_count(app, app.screen);
    if count <= 1 {
        return;
    }

    let current = app.active_field_for_current_screen() as isize;
    let mut next = (current + direction).rem_euclid(count as isize) as usize;

    if requires_custom_mode(app.screen, next) && !app.custom_selected_for_current_screen() {
        next = 0;
    }

    if let Some(state) = app.input_state.get_mut(&app.screen) {
        state.active_field = next;
    }

    if app.screen == Screen::CustomUserType {
        user_flow::sync_custom_user_type(app);
    }
}

pub(crate) fn insert_active_field_char(app: &mut App, c: char) {
    if c.is_control() {
        return;
    }

    let screen = app.screen;
    let active = app.active_field_for_current_screen();
    if requires_custom_mode(screen, active) && !app.custom_selected_for_current_screen() {
        return;
    }
    if let Some(text) = text_field_mut(app, screen, active) {
        text.push(c);
    }
}

pub(crate) fn backspace_active_field(app: &mut App) {
    let screen = app.screen;
    let active = app.active_field_for_current_screen();
    if requires_custom_mode(screen, active) && !app.custom_selected_for_current_screen() {
        return;
    }
    if let Some(text) = text_field_mut(app, screen, active) {
        text.pop();
    }
}

pub(crate) fn toggle_active_field(app: &mut App) {
    let active = app.active_field_for_current_screen();
    match (app.screen, active) {
        (Screen::KeyboardSelect, 0) => toggle_preset_or_custom(app, KEYBOARD_OPTIONS),
        (Screen::LocaleSelect, 0) => toggle_preset_or_custom(app, LOCALE_OPTIONS),
        (Screen::TimezoneSelect, 0) => toggle_preset_or_custom(app, TIMEZONE_OPTIONS),
        (Screen::GpuSelect, 0) => toggle_preset_or_custom(app, GPU_OPTIONS),
        (Screen::CpuSelect, 0) => toggle_preset_or_custom(app, CPU_OPTIONS),
        (Screen::SshToggle, 0) => app.config.ssh_enabled = !app.config.ssh_enabled,
        (Screen::StorageToggle, 0) => app.config.storage_enabled = !app.config.storage_enabled,
        (Screen::UserMenu, _) => move_active_field(app, 1),
        (Screen::CustomUserType, _) => {
            move_active_field(app, 1);
            user_flow::sync_custom_user_type(app);
        }
        (Screen::CustomUserPrograms, _) => user_flow::toggle_program(app, active),
        _ => {}
    }
}

pub(crate) fn editable_field_count(app: &App, screen: Screen) -> usize {
    match screen {
        Screen::DeviceSelect => 1,
        Screen::PartitionConfig => 2,
        Screen::HostnameInput => 1,
        Screen::KeyboardSelect => 2,
        Screen::LocaleSelect => 2,
        Screen::TimezoneSelect => 2,
        Screen::SshToggle => 1,
        Screen::StorageToggle => 1,
        Screen::GpuSelect => 2,
        Screen::CpuSelect => 2,
        Screen::UserMenu => 4,
        Screen::PresetUserPassword => 2,
        Screen::CustomUserBasic => 2,
        Screen::CustomUserType => 2,
        Screen::CustomUserPrograms => user_flow::program_option_count(app),
        Screen::CustomUserPassword => 2,
        Screen::Summary => 1,
        _ => 0,
    }
}

pub(crate) fn active_field_accepts_text(app: &App) -> bool {
    matches!(
        (app.screen, app.active_field_for_current_screen()),
        (Screen::DeviceSelect, 0)
            | (Screen::PartitionConfig, 0)
            | (Screen::PartitionConfig, 1)
            | (Screen::HostnameInput, 0)
            | (Screen::KeyboardSelect, 1)
            | (Screen::LocaleSelect, 1)
            | (Screen::TimezoneSelect, 1)
            | (Screen::GpuSelect, 1)
            | (Screen::CpuSelect, 1)
            | (Screen::PresetUserPassword, 0)
            | (Screen::PresetUserPassword, 1)
            | (Screen::CustomUserBasic, 0)
            | (Screen::CustomUserBasic, 1)
            | (Screen::CustomUserPassword, 0)
            | (Screen::CustomUserPassword, 1)
            | (Screen::Summary, 0)
    )
}

fn text_field_mut(app: &mut App, screen: Screen, active_field: usize) -> Option<&mut String> {
    match (screen, active_field) {
        (Screen::DeviceSelect, 0) => Some(&mut app.config.device),
        (Screen::PartitionConfig, 0) => Some(&mut app.config.boot_end),
        (Screen::PartitionConfig, 1) => Some(&mut app.config.root_end),
        (Screen::HostnameInput, 0) => Some(&mut app.config.hostname),
        (Screen::KeyboardSelect, 1) => Some(&mut app.config.keyboard),
        (Screen::LocaleSelect, 1) => Some(&mut app.config.locale),
        (Screen::TimezoneSelect, 1) => Some(&mut app.config.timezone),
        (Screen::GpuSelect, 1) => Some(&mut app.config.gpu_custom),
        (Screen::CpuSelect, 1) => Some(&mut app.config.cpu_custom),
        (Screen::PresetUserPassword, 0) | (Screen::CustomUserPassword, 0) => {
            app.pending_user.as_mut().map(|user| &mut user.password)
        }
        (Screen::PresetUserPassword, 1) | (Screen::CustomUserPassword, 1) => app
            .pending_user
            .as_mut()
            .map(|user| &mut user.password_confirm),
        (Screen::CustomUserBasic, 0) => app.pending_user.as_mut().map(|user| &mut user.username),
        (Screen::CustomUserBasic, 1) => {
            app.pending_user.as_mut().map(|user| &mut user.display_name)
        }
        (Screen::Summary, 0) => Some(&mut app.install_confirmation),
        _ => None,
    }
}

fn requires_custom_mode(screen: Screen, active_field: usize) -> bool {
    matches!(
        (screen, active_field),
        (Screen::KeyboardSelect, 1)
            | (Screen::LocaleSelect, 1)
            | (Screen::TimezoneSelect, 1)
            | (Screen::GpuSelect, 1)
            | (Screen::CpuSelect, 1)
    )
}

fn toggle_preset_or_custom(app: &mut App, values: &[&str]) {
    let is_custom = app.custom_selected_for_current_screen();
    if is_custom {
        set_custom_selected(app, false);
        apply_preset_for_current_screen(app, values.first().copied().unwrap_or_default());
        return;
    }

    let current = current_text_value_for_screen(app);
    if let Some(index) = values.iter().position(|candidate| *candidate == current) {
        if index + 1 < values.len() {
            apply_preset_for_current_screen(app, values[index + 1]);
        } else {
            set_custom_selected(app, true);
            if let Some(state) = app.input_state.get_mut(&app.screen) {
                state.active_field = 1;
            }
        }
    } else if let Some(first) = values.first() {
        apply_preset_for_current_screen(app, first);
    }
}

fn set_custom_selected(app: &mut App, selected: bool) {
    if let Some(state) = app.input_state.get_mut(&app.screen) {
        state.custom_selected = selected;
        if !selected && state.active_field > 0 {
            state.active_field = 0;
        }
    }
}

fn current_text_value_for_screen(app: &App) -> &str {
    match app.screen {
        Screen::KeyboardSelect => &app.config.keyboard,
        Screen::LocaleSelect => &app.config.locale,
        Screen::TimezoneSelect => &app.config.timezone,
        Screen::GpuSelect => app.config.gpu_type.label(),
        Screen::CpuSelect => app.config.cpu_type.label(),
        _ => "",
    }
}

fn apply_preset_for_current_screen(app: &mut App, value: &str) {
    if value.is_empty() {
        return;
    }

    match app.screen {
        Screen::KeyboardSelect => app.config.keyboard = value.to_string(),
        Screen::LocaleSelect => app.config.locale = value.to_string(),
        Screen::TimezoneSelect => app.config.timezone = value.to_string(),
        Screen::GpuSelect => {
            app.config.gpu_type = match value {
                "none" => GpuType::None,
                "nvidia" => GpuType::Nvidia,
                "amd" => GpuType::Amd,
                "intel" => GpuType::Intel,
                _ => app.config.gpu_type,
            };
        }
        Screen::CpuSelect => {
            app.config.cpu_type = match value {
                "amd" => CpuType::Amd,
                "intel" => CpuType::Intel,
                "aarch64" => CpuType::Aarch64,
                _ => app.config.cpu_type,
            };
        }
        _ => {}
    }
    set_custom_selected(app, false);
}
