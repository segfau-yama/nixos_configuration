use crate::{
    app::{PendingUser, Screen},
    config::{BootType, CpuType, GpuType, UserType},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigChange {
    GitHubUsername(String),
    Repository(String),
    RepositoryPath(String),
    Device(String),
    BootSize(String),
    SwapSize(String),
    Hostname(String),
    Keyboard(String),
    Locale(String),
    Timezone(String),
    SshEnabled(bool),
    StorageEnabled(bool),
    GpuType(GpuType),
    GpuCustom(String),
    CpuType(CpuType),
    CpuCustom(String),
    BootType(BootType),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingUserChange {
    Username(String),
    DisplayName(String),
    UserType(UserType),
    ToggleProgram(String),
    Password(String),
    PasswordConfirm(String),
    Replace(PendingUser),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Noop,
    Quit,
    Navigate(Screen),
    Batch(Vec<Action>),
    SetStatus(Option<String>),
    CheckNetwork,
    PrepareRepository,
    StartInstall,
    ConfigChanged(ConfigChange),
    PendingUserChanged(PendingUserChange),
    StartPresetUser(PendingUser),
    CommitPendingUser,
    ResetPendingUser,
}
