use std::{
    io::{self, BufRead, BufReader, Read},
    process::{Command, Stdio},
    sync::mpsc::{self, RecvTimeoutError},
    thread,
    time::Duration,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

pub trait CommandRunner {
    fn run(&self, program: &str, args: &[&str]) -> io::Result<CommandOutput>;

    fn run_with_log(
        &self,
        program: &str,
        args: &[&str],
        on_line: &mut dyn FnMut(String),
    ) -> io::Result<CommandOutput> {
        let output = self.run(program, args)?;
        for line in output.stdout.lines() {
            on_line(line.to_string());
        }
        for line in output.stderr.lines() {
            on_line(line.to_string());
        }
        Ok(output)
    }
}

pub struct SystemCommandRunner;

impl CommandRunner for SystemCommandRunner {
    fn run(&self, program: &str, args: &[&str]) -> io::Result<CommandOutput> {
        run_system_command(program, args, &mut |_| {})
    }

    fn run_with_log(
        &self,
        program: &str,
        args: &[&str],
        on_line: &mut dyn FnMut(String),
    ) -> io::Result<CommandOutput> {
        run_system_command(program, args, on_line)
    }
}

fn run_system_command(
    program: &str,
    args: &[&str],
    on_line: &mut dyn FnMut(String),
) -> io::Result<CommandOutput> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let (sender, receiver) = mpsc::channel();

    let stdout_reader = stdout.map(|stream| spawn_reader(stream, false, sender.clone()));
    let stderr_reader = stderr.map(|stream| spawn_reader(stream, true, sender.clone()));
    drop(sender);

    let mut stdout = String::new();
    let mut stderr = String::new();

    let status = loop {
        while let Ok(line) = receiver.try_recv() {
            collect_stream_line(line, &mut stdout, &mut stderr, on_line);
        }

        if let Some(status) = child.try_wait()? {
            break status;
        }

        match receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(line) => collect_stream_line(line, &mut stdout, &mut stderr, on_line),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                if let Some(status) = child.try_wait()? {
                    break status;
                }
            }
        }
    };

    while let Ok(line) = receiver.try_recv() {
        collect_stream_line(line, &mut stdout, &mut stderr, on_line);
    }

    if let Some(reader) = stdout_reader {
        let _ = reader.join();
    }
    if let Some(reader) = stderr_reader {
        let _ = reader.join();
    }

    while let Ok(line) = receiver.try_recv() {
        collect_stream_line(line, &mut stdout, &mut stderr, on_line);
    }

    Ok(CommandOutput {
        stdout,
        stderr,
        exit_code: status.code().unwrap_or(-1),
    })
}

fn collect_stream_line(
    line: StreamLine,
    stdout: &mut String,
    stderr: &mut String,
    on_line: &mut dyn FnMut(String),
) {
    if line.is_stderr {
        if !stderr.is_empty() {
            stderr.push('\n');
        }
        stderr.push_str(&line.content);
        on_line(line.content);
    } else {
        if !stdout.is_empty() {
            stdout.push('\n');
        }
        stdout.push_str(&line.content);
        on_line(line.content);
    }
}

struct StreamLine {
    is_stderr: bool,
    content: String,
}

fn spawn_reader<R: Read + Send + 'static>(
    stream: R,
    is_stderr: bool,
    sender: mpsc::Sender<StreamLine>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let reader = BufReader::new(stream);
        for line in reader.lines().map_while(Result::ok) {
            let _ = sender.send(StreamLine {
                is_stderr,
                content: line,
            });
        }
    })
}
