use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    config::{DEFAULT_REPOSITORY_PATH, InstallConfig},
    infra::command_runner::{CommandOutput, CommandRunner, SystemCommandRunner},
};

pub fn prepare_github_repository(
    config: &mut InstallConfig,
    logs: &mut Vec<String>,
) -> Result<(), String> {
    let runner = SystemCommandRunner;
    prepare_github_repository_with_runner(config, logs, &runner)
}

fn prepare_github_repository_with_runner<R: CommandRunner>(
    config: &mut InstallConfig,
    logs: &mut Vec<String>,
    runner: &R,
) -> Result<(), String> {
    let repository_path = normalize_repository_path(&config.repository_path);
    validate_repository_path(&repository_path)?;
    let repository = resolve_repository(config, runner)?;
    clone_repository(&repository, &repository_path, logs, runner)?;

    config.repository = repository.display();
    config.repository_url = repository.remote_url();
    config.repository_path = repository_path.display().to_string();
    logs.push(format!(
        "github: prepared {} at {}",
        config.repository, config.repository_path
    ));
    Ok(())
}

fn clone_repository<R: CommandRunner>(
    repository: &RepositoryRef,
    repository_path: &Path,
    logs: &mut Vec<String>,
    runner: &R,
) -> Result<(), String> {
    if repository_root_available(repository_path) {
        logs.push(format!(
            "github: using existing repository at {}",
            repository_path.display()
        ));
        return Ok(());
    }

    if repository_path.exists() {
        return Err(format!(
            "Clone path exists but is not a nixos_configuration repository: {}",
            repository_path.display()
        ));
    }

    if let Some(parent) = repository_path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create clone parent: {error}"))?;
    }

    match repository {
        RepositoryRef::OwnerName(owner_name) => {
            let path = repository_path.display().to_string();
            run_checked_owned(
                runner,
                "git",
                vec![
                    "clone".to_string(),
                    format!("https://github.com/{owner_name}.git"),
                    path,
                ],
            )?;
        }
        RepositoryRef::Url(url) => {
            let path = repository_path.display().to_string();
            run_checked_owned(runner, "git", vec!["clone".to_string(), url.clone(), path])?;
        }
    }

    if !repository_root_available(repository_path) {
        return Err(format!(
            "Clone completed but repository root was not detected at {}",
            repository_path.display()
        ));
    }

    logs.push(format!(
        "github: cloned {} to {}",
        repository.display(),
        repository_path.display()
    ));
    Ok(())
}

fn run_checked<R: CommandRunner>(
    runner: &R,
    program: &str,
    args: &[&str],
) -> Result<CommandOutput, String> {
    let output = runner
        .run(program, args)
        .map_err(|error| format!("{program} failed to start: {error}"))?;
    if output.exit_code != 0 {
        return Err(format!(
            "{} {} failed: {}",
            program,
            args.join(" "),
            output.stderr.trim()
        ));
    }
    Ok(output)
}

fn run_checked_owned<R: CommandRunner>(
    runner: &R,
    program: &str,
    args: Vec<String>,
) -> Result<CommandOutput, String> {
    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    run_checked(runner, program, &refs)
}

fn normalize_repository_path(path: &str) -> PathBuf {
    let path = path.trim();
    if path.is_empty() {
        PathBuf::from(DEFAULT_REPOSITORY_PATH)
    } else {
        PathBuf::from(path)
    }
}

fn validate_repository_path(path: &Path) -> Result<(), String> {
    if path.as_os_str().is_empty() {
        return Err("Clone path cannot be empty.".to_string());
    }
    if path == Path::new("/") {
        return Err("Clone path cannot be filesystem root (/).".to_string());
    }
    Ok(())
}

fn repository_root_available(path: &Path) -> bool {
    path.join("flake.nix").is_file() && path.join("modules").is_dir()
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RepositoryRef {
    OwnerName(String),
    Url(String),
}

impl RepositoryRef {
    fn display(&self) -> String {
        match self {
            Self::OwnerName(owner_name) => owner_name.clone(),
            Self::Url(url) => url.clone(),
        }
    }

    fn remote_url(&self) -> String {
        match self {
            Self::OwnerName(owner_name) => format!("https://github.com/{owner_name}.git"),
            Self::Url(url) => url.clone(),
        }
    }
}

fn normalize_repository(username: &str, input: &str) -> Result<RepositoryRef, String> {
    let input = input.trim();
    if input.is_empty() {
        if username.trim().is_empty() {
            return Err("GitHub username is empty; repository cannot be inferred.".to_string());
        }
        return Ok(RepositoryRef::OwnerName(format!(
            "{}/nixos_configuration",
            username.trim()
        )));
    }

    if looks_like_repository_url(input) {
        return Ok(RepositoryRef::Url(input.to_string()));
    }

    if input.contains('/') {
        validate_owner_name(input)?;
        return Ok(RepositoryRef::OwnerName(input.to_string()));
    }

    validate_repo_name(input)?;
    Ok(RepositoryRef::OwnerName(format!(
        "{}/{}",
        username.trim(),
        input
    )))
}

fn resolve_repository<R: CommandRunner>(
    config: &mut InstallConfig,
    runner: &R,
) -> Result<RepositoryRef, String> {
    let input = config.repository.trim().to_string();

    if input.is_empty() && !config.repository_url.trim().is_empty() {
        return Ok(RepositoryRef::Url(config.repository_url.trim().to_string()));
    }

    if looks_like_repository_url(&input) {
        return normalize_repository("", &input);
    }

    if input.contains('/') {
        return normalize_repository("", &input);
    }

    let configured_username = config.github_username.trim().to_string();
    if !configured_username.is_empty() {
        return normalize_repository(&configured_username, &input);
    }

    let username = github_username(runner)?;
    config.github_username = username.clone();
    normalize_repository(&username, &input)
}

fn github_username<R: CommandRunner>(runner: &R) -> Result<String, String> {
    run_checked(runner, "gh", &["auth", "status"])
        .map_err(|error| format!("{error}. Install gh or enter a full repository URL."))?;

    let username_output = run_checked(runner, "gh", &["api", "user", "--jq", ".login"])?;
    let username = username_output.stdout.trim();
    if username.is_empty() {
        return Err("GitHub username could not be detected from gh.".to_string());
    }
    Ok(username.to_string())
}

fn looks_like_repository_url(input: &str) -> bool {
    input.contains("://") || input.starts_with("git@") || input.ends_with(".git")
}

fn validate_owner_name(input: &str) -> Result<(), String> {
    let Some((owner, repo)) = input.split_once('/') else {
        return Err("Repository must be owner/name.".to_string());
    };
    validate_repo_name(owner)?;
    validate_repo_name(repo)
}

fn validate_repo_name(input: &str) -> Result<(), String> {
    if input.is_empty() {
        return Err("Repository name cannot be empty.".to_string());
    }
    if !input
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err("Repository name contains invalid characters.".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, io};

    use super::*;

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
            let stdout = if program == "gh" && args == ["api", "user", "--jq", ".login"] {
                "suichan\n"
            } else {
                ""
            };
            Ok(CommandOutput {
                stdout: stdout.to_string(),
                stderr: String::new(),
                exit_code: 0,
            })
        }
    }

    #[test]
    fn normalize_repository_infers_default_fork_from_username() {
        assert_eq!(
            normalize_repository("suichan", "").unwrap(),
            RepositoryRef::OwnerName("suichan/nixos_configuration".to_string())
        );
    }

    #[test]
    fn normalize_repository_accepts_repo_name_owner_name_and_url() {
        assert_eq!(
            normalize_repository("suichan", "custom_config").unwrap(),
            RepositoryRef::OwnerName("suichan/custom_config".to_string())
        );
        assert_eq!(
            normalize_repository("suichan", "owner/nixos_configuration").unwrap(),
            RepositoryRef::OwnerName("owner/nixos_configuration".to_string())
        );
        assert_eq!(
            normalize_repository("suichan", "https://github.com/owner/repo.git").unwrap(),
            RepositoryRef::Url("https://github.com/owner/repo.git".to_string())
        );
    }

    #[test]
    fn prepare_github_repository_uses_git_clone_for_owner_name_without_gh() {
        let runner = RecordingRunner::default();
        let mut config = InstallConfig {
            repository: "segfau-yama/nixos_configuration".to_string(),
            repository_path: format!(
                "{}/jadeos_setting_tui_missing_repo_for_test",
                std::env::temp_dir().display()
            ),
            ..InstallConfig::default()
        };
        let mut logs = Vec::new();

        let result = prepare_github_repository_with_runner(&mut config, &mut logs, &runner);

        assert!(result.is_err());
        assert!(runner.calls().iter().any(|call| {
            call.starts_with("git clone https://github.com/segfau-yama/nixos_configuration.git ")
        }));
        assert!(!runner.calls().iter().any(|call| call.starts_with("gh ")));
        assert_eq!(config.github_username, "");
    }

    #[test]
    fn prepare_github_repository_uses_configured_username_for_repo_name_without_gh() {
        let runner = RecordingRunner::default();
        let mut config = InstallConfig {
            github_username: "suichan".to_string(),
            repository: "nixos_configuration".to_string(),
            repository_path: format!(
                "{}/jadeos_setting_tui_missing_repo_name_for_test",
                std::env::temp_dir().display()
            ),
            ..InstallConfig::default()
        };
        let mut logs = Vec::new();

        let result = prepare_github_repository_with_runner(&mut config, &mut logs, &runner);

        assert!(result.is_err());
        assert!(runner.calls().iter().any(|call| {
            call.starts_with("git clone https://github.com/suichan/nixos_configuration.git ")
        }));
        assert!(!runner.calls().iter().any(|call| call.starts_with("gh ")));
    }

    #[test]
    fn prepare_github_repository_uses_git_clone_for_full_url_without_gh() {
        let runner = RecordingRunner::default();
        let mut config = InstallConfig {
            repository: "https://github.com/segfau-yama/nixos_configuration.git".to_string(),
            repository_path: format!(
                "{}/jadeos_setting_tui_missing_url_repo_for_test",
                std::env::temp_dir().display()
            ),
            ..InstallConfig::default()
        };
        let mut logs = Vec::new();

        let result = prepare_github_repository_with_runner(&mut config, &mut logs, &runner);

        assert!(result.is_err());
        assert!(
            runner
                .calls()
                .iter()
                .any(|call| call.starts_with("git clone https://github.com/segfau-yama/"))
        );
        assert!(!runner.calls().iter().any(|call| call.starts_with("gh ")));
    }

    #[test]
    fn prepare_github_repository_rejects_filesystem_root_clone_path() {
        let runner = RecordingRunner::default();
        let mut config = InstallConfig {
            repository: "https://github.com/segfau-yama/nixos_configuration.git".to_string(),
            repository_path: "/".to_string(),
            ..InstallConfig::default()
        };
        let mut logs = Vec::new();

        let error =
            prepare_github_repository_with_runner(&mut config, &mut logs, &runner).unwrap_err();

        assert!(error.contains("filesystem root"));
        assert!(runner.calls().is_empty());
    }
}
