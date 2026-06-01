use crate::{
    app::{App, PendingUser, Screen, UserMenuChoice},
    config::{UserConfig, UserType},
    infra::password_hasher::hash_password,
    logic::user_programs::program_options_for,
};

pub(crate) fn confirm_user_menu(app: &mut App) {
    match UserMenuChoice::from_index(app.active_field_for_current_screen()) {
        UserMenuChoice::JadeCore => start_preset_user(app, "jade-core"),
        UserMenuChoice::JadeOffice => start_preset_user(app, "jade-office"),
        UserMenuChoice::JadeGaming => start_preset_user(app, "jade-gaming"),
        UserMenuChoice::JadeDevelop => start_preset_user(app, "jade-develop"),
        UserMenuChoice::JadeFull => start_preset_user(app, "jade-full"),
        UserMenuChoice::Custom => {
            app.pending_user = Some(PendingUser::custom());
            set_active_field(app, Screen::CustomUserBasic, 0);
            app.screen = Screen::CustomUserBasic;
        }
        UserMenuChoice::Finish => {
            if app.config.users.is_empty() {
                app.user_message = Some("Please add at least one user.".to_string());
            } else {
                app.user_message = None;
                app.screen = Screen::Summary;
            }
        }
    }
}

pub(crate) fn confirm_custom_basic(app: &mut App) {
    let Some(user) = app.pending_user.as_ref() else {
        app.pending_user = Some(PendingUser::custom());
        return;
    };

    let username = user.username.trim().to_string();
    let display_name = user.display_name.trim().to_string();
    if username.is_empty() {
        app.user_message = Some("Username cannot be empty.".to_string());
        return;
    }
    if !is_valid_username(&username) {
        app.user_message = Some(
            "Use lowercase letters, digits, '_' or '-' and start with a letter or '_'.".to_string(),
        );
        return;
    }
    if is_reserved_username(&username) || app.user_exists(&username) {
        app.user_message = Some(format!("{username} is reserved or already added."));
        return;
    }

    let Some(user) = app.pending_user.as_mut() else {
        return;
    };
    user.username = username;
    if display_name.is_empty() {
        user.display_name = default_display_name(&user.username);
    } else {
        user.display_name = display_name;
    }
    set_active_field(app, Screen::CustomUserType, 0);
    sync_custom_user_type(app);
    app.screen = Screen::CustomUserType;
}

pub(crate) fn confirm_user_password(app: &mut App) {
    let Some(user) = app.pending_user.as_mut() else {
        app.user_message = Some("No pending user to add.".to_string());
        app.screen = Screen::UserMenu;
        return;
    };

    if user.password.is_empty() {
        app.user_message = Some("Password cannot be empty.".to_string());
        return;
    }
    if user.password != user.password_confirm {
        app.user_message = Some("Passwords do not match.".to_string());
        return;
    }

    let hash = match hash_password(&user.password) {
        Ok(hash) => hash,
        Err(error) => {
            app.user_message = Some(format!("Password hash failed: {error}"));
            return;
        }
    };

    let user = app.pending_user.take().expect("pending user exists");
    app.config.users.push(UserConfig {
        username: user.username,
        display_name: user.display_name,
        user_type: user.user_type,
        programs: normalize_programs(user.user_type, user.programs),
        password_hash: hash,
        is_preset: user.is_preset,
    });
    app.user_message = Some("User added. Press Enter to return to user menu.".to_string());
    app.screen = Screen::UserAddResult;
}

pub(crate) fn sync_custom_user_type(app: &mut App) {
    let active = app.active_field_for_current_screen();
    if let Some(user) = app.pending_user.as_mut() {
        user.user_type = match active {
            0 => UserType::Gui,
            1 => UserType::Tui,
            _ => UserType::Cui,
        };
        user.programs = normalize_programs(user.user_type, user.programs.clone());
    }
}

pub(crate) fn toggle_program(app: &mut App, active: usize) {
    let Some(user) = app.pending_user.as_mut() else {
        return;
    };
    let options = program_options_for(user.user_type);
    let Some(program) = options.get(active).map(|(name, _)| *name) else {
        return;
    };
    if user.programs.iter().any(|candidate| candidate == program) {
        user.programs.retain(|candidate| candidate != program);
    } else {
        user.programs.push(program.to_string());
    }
    user.programs = normalize_programs(user.user_type, user.programs.clone());
}

pub(crate) fn program_option_count(app: &App) -> usize {
    app.pending_user
        .as_ref()
        .map(|user| program_options_for(user.user_type).len())
        .unwrap_or(0)
}

fn start_preset_user(app: &mut App, username: &str) {
    if app.user_exists(username) {
        app.user_message = Some(format!("{username} is already added."));
        return;
    }

    app.pending_user = match username {
        "jade-core" => Some(PendingUser::preset(
            "jade-core",
            "Jade Core",
            UserType::Tui,
            &["programming", "browser", "media", "sns"],
        )),
        "jade-office" => Some(PendingUser::preset(
            "jade-office",
            "Jade Office",
            UserType::Gui,
            &["browser", "media", "sns", "office"],
        )),
        "jade-gaming" => Some(PendingUser::preset(
            "jade-gaming",
            "Jade Gaming",
            UserType::Gui,
            &["browser", "gaming", "media", "sns"],
        )),
        "jade-develop" => Some(PendingUser::preset(
            "jade-develop",
            "Jade Develop",
            UserType::Gui,
            &[
                "programming",
                "browser",
                "media",
                "sns",
                "electronics",
                "mechanical",
            ],
        )),
        "jade-full" => Some(PendingUser::preset(
            "jade-full",
            "Jade Full",
            UserType::Gui,
            &[
                "programming",
                "browser",
                "gaming",
                "media",
                "sns",
                "electronics",
                "mechanical",
            ],
        )),
        _ => None,
    };
    set_active_field(app, Screen::PresetUserPassword, 0);
    app.screen = Screen::PresetUserPassword;
}

fn normalize_programs(user_type: UserType, programs: Vec<String>) -> Vec<String> {
    let options = program_options_for(user_type);
    let mut normalized = Vec::new();
    for (program, _) in options {
        if programs.iter().any(|candidate| candidate == program) {
            normalized.push(program.to_string());
        }
    }
    normalized
}

fn is_reserved_username(username: &str) -> bool {
    matches!(
        username,
        "jade-core" | "jade-office" | "jade-gaming" | "jade-develop" | "jade-full"
    )
}

fn is_valid_username(username: &str) -> bool {
    let mut chars = username.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_lowercase() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

fn default_display_name(username: &str) -> String {
    let mut chars = username.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    format!("{}{}", first.to_ascii_uppercase(), chars.as_str())
}

fn set_active_field(app: &mut App, screen: Screen, active_field: usize) {
    if let Some(state) = app.input_state.get_mut(&screen) {
        state.active_field = active_field;
    }
}
