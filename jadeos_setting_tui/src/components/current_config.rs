use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::{App, Screen};

pub fn current_config_panel(app: &App) -> Paragraph<'static> {
    Paragraph::new(current_config_text(app))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" current config ")
                .border_style(Style::default().fg(Color::Magenta)),
        )
        .style(Style::default().fg(Color::Gray))
        .wrap(Wrap { trim: false })
}

fn current_config_text(app: &App) -> Text<'static> {
    Text::from(vec![
        summary_line("github", &github_display(app)),
        summary_line("repo path", &app.config.repository_path),
        Line::default(),
        summary_line("hostname", &app.config.hostname),
        summary_line("boot loader", app.config.boot_type.label()),
        summary_line("device", &app.config.device),
        summary_line("boot size", &app.config.boot_size),
        summary_line("swap size", &swap_display(app)),
        summary_line("root", "remaining disk"),
        Line::default(),
        summary_line("keyboard", &app.config.keyboard),
        summary_line("locale", &app.config.locale),
        summary_line("timezone", &app.config.timezone),
        Line::default(),
        summary_line("gpu", &gpu_display(app)),
        summary_line("cpu", &cpu_display(app)),
        summary_line("ssh", &app.config.ssh_enabled.to_string()),
        summary_line("storage", &app.config.storage_enabled.to_string()),
        summary_line("users", &users_display(app)),
        summary_line("gui user", &app.config.has_gui_user().to_string()),
        summary_line(
            "programming",
            &app.config.needs_programming_cli().to_string(),
        ),
    ])
}

fn github_display(app: &App) -> String {
    if !app.config.repository.trim().is_empty() {
        app.config.repository.clone()
    } else if !app.config.repository_url.trim().is_empty() {
        app.config.repository_url.clone()
    } else {
        "not selected".to_string()
    }
}

fn users_display(app: &App) -> String {
    if app.config.users.is_empty() {
        "none".to_string()
    } else {
        app.config
            .users
            .iter()
            .map(|user| format!("{}({})", user.username, user.user_type.label()))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn swap_display(app: &App) -> String {
    if app.config.has_swap_partition() {
        app.config.swap_size.clone()
    } else {
        "disabled".to_string()
    }
}

fn gpu_display(app: &App) -> String {
    if app
        .input_state
        .get(&Screen::GpuSelect)
        .map(|state| state.custom_selected)
        .unwrap_or(false)
        && !app.config.gpu_custom.is_empty()
    {
        app.config.gpu_custom.clone()
    } else {
        app.config.gpu_type.label().to_string()
    }
}

fn cpu_display(app: &App) -> String {
    if app
        .input_state
        .get(&Screen::CpuSelect)
        .map(|state| state.custom_selected)
        .unwrap_or(false)
        && !app.config.cpu_custom.is_empty()
    {
        app.config.cpu_custom.clone()
    } else {
        app.config.cpu_type.label().to_string()
    }
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
