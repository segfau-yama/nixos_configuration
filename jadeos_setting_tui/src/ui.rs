use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Tabs, Wrap},
};

use crate::app::{App, Screen};
use crate::components::controls::controls_line;
use crate::components::current_config::current_config_panel;
use crate::components::form::{FormSection, render_form_section};
use crate::logic::screen_content::{main_panel_text, screen_form_section, screen_popup_section};

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
            Screen::UserMenu
            | Screen::PresetUserPassword
            | Screen::CustomUserBasic
            | Screen::CustomUserType
            | Screen::CustomUserPrograms
            | Screen::CustomUserPassword
            | Screen::UserAddResult => 1,
            Screen::Summary | Screen::Installing | Screen::Done => 2,
            _ => 0,
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
    frame.render_widget(tabs, layout[0]);

    let body_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(68), Constraint::Percentage(32)])
        .split(layout[1]);

    let body_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", app.screen.title()))
        .border_style(Style::default().fg(main_border_color(app.screen)));
    let body_inner = body_block.inner(body_layout[0]);
    frame.render_widget(body_block, body_layout[0]);

    let body_inner_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(3)])
        .split(body_inner);

    let header = Paragraph::new(main_panel_text(app))
        .style(Style::default().fg(Color::Gray))
        .wrap(Wrap { trim: false });
    frame.render_widget(header, body_inner_layout[0]);

    let section = screen_form_section(app);
    render_form_section(frame, body_inner_layout[1], &section);
    frame.render_widget(current_config_panel(app), body_layout[1]);

    if let Some(popup_section) = screen_popup_section(app) {
        render_popup_overlay(frame, app, popup_section);
    }

    let help = Paragraph::new(Line::from(controls_line())).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" controls ")
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(help, layout[2]);
}

fn render_popup_overlay(frame: &mut Frame, app: &App, section: FormSection) {
    let (width, height) = match app.screen {
        Screen::Summary => (72, 36),
        Screen::PartitionConfirm => (72, 36),
        Screen::PresetUserPassword | Screen::CustomUserPassword => (72, 54),
        _ => (66, 44),
    };
    let popup_area = centered_rect(width, height, frame.area());
    let popup = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", app.screen.title()))
        .border_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    let popup_inner = popup.inner(popup_area);

    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup, popup_area);
    render_form_section(frame, popup_inner, &section);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

fn main_border_color(screen: Screen) -> Color {
    match screen {
        Screen::Welcome | Screen::Done => Color::Green,
        Screen::Installing => Color::Yellow,
        Screen::Summary => Color::Magenta,
        Screen::UserMenu => Color::Cyan,
        _ => Color::Blue,
    }
}
