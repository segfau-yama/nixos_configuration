use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::config::InstallConfig;

pub const STATE_FILE_PATH: &str = "/tmp/jadeos_setting_tui_state.toml";

pub fn load_install_config(path: impl AsRef<Path>) -> io::Result<InstallConfig> {
    let raw = fs::read_to_string(path)?;
    toml::from_str::<InstallConfig>(&raw).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("failed to parse state TOML: {error}"),
        )
    })
}

pub fn save_install_config(path: impl AsRef<Path>, config: &InstallConfig) -> io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let body = toml::to_string_pretty(config).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("failed to serialize state TOML: {error}"),
        )
    })?;

    let tmp_path = temp_path(path);
    fs::write(&tmp_path, body)?;
    fs::rename(tmp_path, path)?;
    Ok(())
}

fn temp_path(path: &Path) -> PathBuf {
    let mut os = path.as_os_str().to_os_string();
    os.push(".tmp");
    PathBuf::from(os)
}
