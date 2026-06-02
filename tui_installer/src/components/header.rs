use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Tabs},
};

use crate::{
    app::{AppSnapshot, Phase},
    component::Component,
    terminal::Frame,
};

pub struct Header {
    phase: Phase,
}

impl Header {
    pub fn new() -> Self {
        Self {
            phase: Phase::PcConfig,
        }
    }

    pub fn sync(&mut self, snapshot: &AppSnapshot) {
        self.phase = snapshot.screen.phase();
    }
}

impl Component for Header {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let tabs = Tabs::new(["PC Config", "Users", "Install"])
            .select(self.phase.tab_index())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" installer ")
                    .border_style(Style::default().fg(Color::Blue)),
            )
            .style(Style::default().fg(Color::DarkGray))
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .divider(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
        f.render_widget(tabs, rect);
    }
}