use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::{
    app::{AppSnapshot, PendingUser},
    component::Component,
    config::InstallConfig,
    terminal::Frame,
};

pub struct Sidebar {
    snapshot: Option<AppSnapshot>,
}

impl Sidebar {
    pub fn new() -> Self {
        Self { snapshot: None }
    }

    pub fn sync(&mut self, snapshot: &AppSnapshot) {
        self.snapshot = Some(snapshot.clone());
    }
}

impl Component for Sidebar {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let Some(snapshot) = self.snapshot.as_ref() else {
            return;
        };

        let paragraph = Paragraph::new(sidebar_text(snapshot))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" current config ")
                    .border_style(Style::default().fg(Color::Magenta)),
            )
            .wrap(Wrap { trim: false });
        f.render_widget(paragraph, rect);
    }
}

fn sidebar_text(snapshot: &AppSnapshot) -> Text<'static> {
    let mut lines = vec![
        summary_line("github", &github_display(&snapshot.config)),
        summary_line("repo path", &snapshot.config.repository_path),
        Line::default(),
        summary_line("hostname", &snapshot.config.hostname),
        summary_line("boot loader", snapshot.config.boot_type.label()),
        summary_line("device", &snapshot.config.device),
        summary_line("boot size", &snapshot.config.boot_size),
        summary_line("swap size", &swap_display(&snapshot.config)),
        summary_line("root", "remaining disk"),
        Line::default(),
        summary_line("keyboard", &snapshot.config.keyboard),
        summary_line("locale", &snapshot.config.locale),
        summary_line("timezone", &snapshot.config.timezone),
        Line::default(),
        summary_line("gpu", &gpu_display(&snapshot.config)),
        summary_line("cpu", &cpu_display(&snapshot.config)),
        summary_line("ssh", &snapshot.config.ssh_enabled.to_string()),
        summary_line("users", &users_display(&snapshot.config)),
        summary_line("gui user", &snapshot.config.has_gui_user().to_string()),
        summary_line(
            "programming",
            &snapshot.config.needs_programming_cli().to_string(),
        ),
    ];

    if let Some(pending_user) = snapshot.pending_user.as_ref() {
        lines.push(Line::default());
        lines.push(summary_line("draft", &pending_user_display(pending_user)));
    }

    if let Some(message) = snapshot.status_message.as_ref() {
        lines.push(Line::default());
        lines.push(Line::from(vec![
            Span::styled(
                "status",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": ", Style::default().fg(Color::DarkGray)),
            Span::styled(message.clone(), Style::default().fg(Color::White)),
        ]));
    }

    Text::from(lines)
}

fn github_display(config: &InstallConfig) -> String {
    if !config.repository_url.trim().is_empty() {
        trim_git_suffix(config.repository_url.trim()).to_string()
    } else if looks_like_repo_url(config.repository.trim()) {
        trim_git_suffix(config.repository.trim()).to_string()
    } else if config.repository.trim().contains('/') {
        format!(
            "https://github.com/{}",
            trim_git_suffix(config.repository.trim())
        )
    } else if !config.github_username.trim().is_empty() {
        config.github_username.clone()
    } else {
        "not selected".to_string()
    }
}

fn looks_like_repo_url(value: &str) -> bool {
    value.contains("://") || value.starts_with("git@")
}

fn trim_git_suffix(value: &str) -> &str {
    value.strip_suffix(".git").unwrap_or(value)
}

fn gpu_display(config: &InstallConfig) -> String {
    if config.gpu_custom.trim().is_empty() {
        config.gpu_type.label().to_string()
    } else {
        config.gpu_custom.clone()
    }
}

fn cpu_display(config: &InstallConfig) -> String {
    if config.cpu_custom.trim().is_empty() {
        config.cpu_type.label().to_string()
    } else {
        config.cpu_custom.clone()
    }
}

fn swap_display(config: &InstallConfig) -> String {
    if config.has_swap_partition() {
        config.swap_size.clone()
    } else {
        "disabled".to_string()
    }
}

fn users_display(config: &InstallConfig) -> String {
    if config.users.is_empty() {
        "none".to_string()
    } else {
        config
            .users
            .iter()
            .map(|user| format!("{}({})", user.username, user.user_type.label()))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn pending_user_display(user: &PendingUser) -> String {
    format!(
        "{} / {} / {}",
        user.username,
        user.display_name,
        user.user_type.label()
    )
}

fn summary_line(label: &'static str, value: &str) -> Line<'static> {
    let value = if value.trim().is_empty() {
        "<empty>".to_string()
    } else {
        value.to_string()
    };

    Line::from(vec![
        Span::styled(label, Style::default().fg(Color::Gray)),
        Span::styled(": ", Style::default().fg(Color::DarkGray)),
        Span::styled(value.clone(), summary_value_style(&value)),
    ])
}

fn summary_value_style(value: &str) -> Style {
    match value {
        "<empty>" => Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
        "true" => Style::default().fg(Color::Green),
        "false" => Style::default().fg(Color::Red),
        _ => Style::default().fg(Color::Cyan),
    }
}
