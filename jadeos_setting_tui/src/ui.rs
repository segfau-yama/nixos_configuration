use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
};

use crate::app::{App, Screen};

pub fn render(frame: &mut Frame, app: &App) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let tabs = Tabs::new(["PC Config", "Users", "Install"])
        .select(match app.screen {
            Screen::UserMenu => 1,
            Screen::Summary | Screen::Installing | Screen::Done => 2,
            _ => 0,
        })
        .block(Block::default().borders(Borders::ALL).title("jadeos installer"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    frame.render_widget(tabs, layout[0]);

    let body = Paragraph::new(body_text(app))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(app.screen.title()),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(body, layout[1]);

    let help = Paragraph::new(Line::from(vec![
        "q".bold(),
        " quit   ".into(),
        "Enter/Right".bold(),
        " next   ".into(),
        "Esc/Left".bold(),
        " back".into(),
    ]))
    .block(Block::default().borders(Borders::ALL).title("controls"));
    frame.render_widget(help, layout[2]);
}

fn body_text(app: &App) -> Text<'static> {
    let lines = vec![
        Line::from(format!("phase: {}", app.screen.phase())),
        Line::from(format!("screen: {}", app.screen.title())),
        Line::default(),
        Line::from("Phase 1 skeleton is active."),
        Line::from("This build wires state, event loop, and placeholder screens."),
        Line::from("Phase 2 will replace placeholders with actual setup.sh forms."),
        Line::default(),
        Line::from(format!("hostname: {}", app.config.hostname)),
        Line::from(format!("boot loader: {}", app.config.boot_type.label())),
        Line::from(format!("gpu: {}", app.config.gpu_type.label())),
        Line::from(format!("cpu: {}", app.config.cpu_type.label())),
        Line::from(format!("locale: {}", app.config.locale)),
    ];

    Text::from(lines)
}