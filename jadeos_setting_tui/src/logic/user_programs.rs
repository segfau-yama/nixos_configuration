use crate::config::UserType;

const DEV_PROGRAM_OPTIONS: &[(&str, &str)] = &[("programming", "Development toolset")];

const TUI_PROGRAM_OPTIONS: &[(&str, &str)] = &[
    ("browser", "TUI web browser"),
    ("media", "TUI media tools"),
    ("sns", "TUI communication tools"),
];

const GUI_PROGRAM_OPTIONS: &[(&str, &str)] = &[
    ("browser", "Chromium web browser"),
    ("gaming", "Steam + Lutris + Wine"),
    ("media", "Spotify + mpv"),
    ("sns", "Discord"),
    ("office", "LibreOffice office suite"),
    ("electronics", "KiCad"),
    ("mechanical", "FreeCAD + MeshLab"),
];

pub fn program_options_for(user_type: UserType) -> Vec<(&'static str, &'static str)> {
    match user_type {
        UserType::Gui => DEV_PROGRAM_OPTIONS
            .iter()
            .copied()
            .chain(GUI_PROGRAM_OPTIONS.iter().copied())
            .collect(),
        UserType::Tui => DEV_PROGRAM_OPTIONS
            .iter()
            .copied()
            .chain(TUI_PROGRAM_OPTIONS.iter().copied())
            .collect(),
        UserType::Cui => DEV_PROGRAM_OPTIONS.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::program_options_for;
    use crate::config::UserType;

    fn option_names(user_type: UserType) -> Vec<&'static str> {
        program_options_for(user_type)
            .into_iter()
            .map(|(name, _)| name)
            .collect()
    }

    #[test]
    fn gui_options_include_all_agent_package_tokens() {
        assert_eq!(
            option_names(UserType::Gui),
            vec![
                "programming",
                "browser",
                "gaming",
                "media",
                "sns",
                "office",
                "electronics",
                "mechanical",
            ]
        );
    }

    #[test]
    fn tui_options_include_dev_and_tui_variant_tokens() {
        assert_eq!(
            option_names(UserType::Tui),
            vec!["programming", "browser", "media", "sns",]
        );
    }

    #[test]
    fn cui_options_include_dev_tokens_only() {
        assert_eq!(option_names(UserType::Cui), vec!["programming"]);
    }
}
