use crate::infra::command_runner::{CommandRunner, SystemCommandRunner};

pub fn check_network_connectivity() -> Result<(), String> {
    let runner = SystemCommandRunner;
    check_network_connectivity_with_runner(&runner)
}

fn check_network_connectivity_with_runner<R: CommandRunner>(runner: &R) -> Result<(), String> {
    let output = runner
        .run("ping", &["-c", "1", "8.8.8.8"])
        .map_err(|error| format!("network check failed to run ping: {error}"))?;
    if output.exit_code != 0 {
        return Err(format!("network check failed: {}", output.stderr.trim()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, io};

    use super::*;
    use crate::infra::command_runner::CommandOutput;

    struct StaticRunner {
        exit_code: i32,
        stderr: &'static str,
        calls: RefCell<Vec<(String, Vec<String>)>>,
    }

    impl StaticRunner {
        fn new(exit_code: i32, stderr: &'static str) -> Self {
            Self {
                exit_code,
                stderr,
                calls: RefCell::new(Vec::new()),
            }
        }
    }

    impl CommandRunner for StaticRunner {
        fn run(&self, program: &str, args: &[&str]) -> io::Result<CommandOutput> {
            self.calls.borrow_mut().push((
                program.to_string(),
                args.iter().map(|arg| arg.to_string()).collect(),
            ));
            Ok(CommandOutput {
                stdout: String::new(),
                stderr: self.stderr.to_string(),
                exit_code: self.exit_code,
            })
        }
    }

    #[test]
    fn network_check_uses_single_packet_ping() {
        let runner = StaticRunner::new(0, "");

        check_network_connectivity_with_runner(&runner).unwrap();

        assert_eq!(
            runner.calls.borrow().as_slice(),
            &[(
                "ping".to_string(),
                vec!["-c".to_string(), "1".to_string(), "8.8.8.8".to_string()]
            )]
        );
    }

    #[test]
    fn network_check_reports_ping_failure() {
        let runner = StaticRunner::new(1, "network unreachable");

        let error = check_network_connectivity_with_runner(&runner).unwrap_err();

        assert!(error.contains("network unreachable"));
    }
}
