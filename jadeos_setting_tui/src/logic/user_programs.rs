use crate::config::UserType;

const GUI_PROGRAM_OPTIONS: &[(&str, &str)] = &[
    ("browser", "Chromium web browser"),
    ("gaming", "Steam + Lutris + Wine"),
    ("media", "Spotify + mpv"),
    ("sns", "Discord"),
    ("kicad", "KiCad"),
    ("freecad", "FreeCAD + MeshLab"),
    ("zed", "Zed editor"),
];

const DEV_PROGRAM_OPTIONS: &[(&str, &str)] = &[
    ("programming", "Shell setup"),
    ("lang", "Language toolchains"),
    ("nix-tools", "Nix ecosystem"),
    ("cli-tools", "General CLI tools"),
];

pub fn program_options_for(user_type: UserType) -> Vec<(&'static str, &'static str)> {
    match user_type {
        UserType::Gui => std::iter::once(("desktop", "Niri desktop environment"))
            .chain(GUI_PROGRAM_OPTIONS.iter().copied())
            .chain(DEV_PROGRAM_OPTIONS.iter().copied())
            .collect(),
        UserType::Cui => DEV_PROGRAM_OPTIONS.to_vec(),
    }
}
