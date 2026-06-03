use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
};

use crate::{
    action::Action,
    app::{AppSnapshot, Screen},
    component::Component,
    components::{
        Popup,
        form::{FormFieldRole, FormSection, render_form_section},
    },
    pages::{InstallerPage, form_field, status_field},
    terminal::Frame,
};

#[derive(Default)]
pub struct DonePage {
    install_log: Vec<String>,
    install_running: bool,
    install_finished: bool,
    status_message: Option<String>,
}

pub fn page() -> Box<dyn InstallerPage> {
    Box::new(DonePage::default())
}

impl InstallerPage for DonePage {
    fn screen(&self) -> Screen {
        Screen::Done
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.install_log = snapshot.install_log.clone();
        self.install_running = snapshot.install_running;
        self.install_finished = snapshot.install_finished;
        self.status_message = snapshot.status_message.clone();
    }

    fn popup(&self) -> Option<Popup> {
        Some(Popup::new(
            popup_title(
                self.install_running,
                self.install_finished,
                self.status_message.as_deref(),
            ),
            88,
            76,
            FormSection::new(
                section_title(
                    self.install_running,
                    self.install_finished,
                    self.status_message.as_deref(),
                ),
                vec![
                    form_field(
                        "result",
                        result_display(
                            self.status_message.as_deref(),
                            self.install_running,
                            self.install_finished,
                            self.install_log.is_empty(),
                        ),
                        Some(done_hint(self.install_running).to_string()),
                        FormFieldRole::ReadOnly,
                    ),
                    form_field(
                        "install log",
                        log_display(&self.install_log, self.install_running),
                        None,
                        FormFieldRole::Log,
                    ),
                ],
                None,
                false,
            ),
        ))
    }
}

impl Component for DonePage {
    fn update(&mut self, action: Action) -> Action {
        match action {
            Action::Navigate(Screen::Done) => Action::StartInstall,
            _ => Action::Noop,
        }
    }

    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') if self.install_running => Action::SetStatus(Some(
                "Installation is running. Wait for completion before quitting.".to_string(),
            )),
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Left | KeyCode::Esc if self.install_running => Action::SetStatus(Some(
                "Installation is running. Wait for completion before returning.".to_string(),
            )),
            KeyCode::Left | KeyCode::Esc => Action::Navigate(Screen::Summary),
            _ => Action::Noop,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let section = FormSection::new(
            "done",
            vec![
                form_field(
                    "result",
                    result_display(
                        self.status_message.as_deref(),
                        self.install_running,
                        self.install_finished,
                        self.install_log.is_empty(),
                    ),
                    Some(done_hint(self.install_running).to_string()),
                    FormFieldRole::ReadOnly,
                ),
                status_field(self.status_message.as_deref()),
            ],
            None,
            false,
        );
        render_form_section(f, rect, &section);
    }
}

fn result_display(
    status_message: Option<&str>,
    install_running: bool,
    install_finished: bool,
    log_is_empty: bool,
) -> &'static str {
    if install_running {
        return "RUNNING - installation in progress";
    }

    match status_message {
        Some(message) if message.contains("failed") || message.contains("Failed") => {
            "FAILED - installation did not complete"
        }
        Some(_) => "SUCCESS - installation complete",
        None if install_finished => "SUCCESS - installation finished",
        None if log_is_empty => "installation pending",
        None => "installation finished",
    }
}

fn log_display(install_log: &[String], install_running: bool) -> String {
    if install_log.is_empty() {
        if install_running {
            return "install: waiting for first log entry".to_string();
        }
        return "No install log entries".to_string();
    }

    let start = install_log.len().saturating_sub(24);
    install_log[start..].join("\n")
}

fn done_hint(install_running: bool) -> &'static str {
    if install_running {
        "Install is running; live logs appear below"
    } else {
        "Review logs, then reboot or press q to quit"
    }
}

fn popup_title(
    install_running: bool,
    install_finished: bool,
    status_message: Option<&str>,
) -> &'static str {
    if install_running {
        return "Install Running";
    }
    if status_message
        .map(|message| message.contains("failed") || message.contains("Failed"))
        .unwrap_or(false)
    {
        return "Install Failed";
    }
    if install_finished {
        return "Install Complete";
    }
    "Install Log"
}

fn section_title(
    install_running: bool,
    install_finished: bool,
    status_message: Option<&str>,
) -> &'static str {
    if install_running {
        return "install running";
    }
    if status_message
        .map(|message| message.contains("failed") || message.contains("Failed"))
        .unwrap_or(false)
    {
        return "install failed";
    }
    if install_finished {
        return "install complete";
    }
    "install log"
}
