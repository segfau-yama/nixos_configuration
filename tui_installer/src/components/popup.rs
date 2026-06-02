use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear},
};

use crate::{
    components::form::{FormSection, render_form_section},
    terminal::Frame,
};

pub struct Popup {
    pub title: String,
    pub width_percent: u16,
    pub height_percent: u16,
    pub section: FormSection,
}

impl Popup {
    pub fn new(
        title: impl Into<String>,
        width_percent: u16,
        height_percent: u16,
        section: FormSection,
    ) -> Self {
        Self {
            title: title.into(),
            width_percent,
            height_percent,
            section,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let popup_area = centered_rect(self.width_percent, self.height_percent, area);
        let popup = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", self.title))
            .border_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
        let popup_inner = popup.inner(popup_area);

        frame.render_widget(Clear, popup_area);
        frame.render_widget(popup, popup_area);
        render_form_section(frame, popup_inner, &self.section);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let percent_x = percent_x.min(100);
    let percent_y = percent_y.min(100);
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
