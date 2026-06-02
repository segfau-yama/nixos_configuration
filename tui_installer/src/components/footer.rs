use ratatui::{
    layout::Rect,
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{app::AppSnapshot, component::Component, terminal::Frame};

pub struct Footer;

impl Footer {
    pub fn new() -> Self {
        Self
    }

    pub fn sync(&mut self, _snapshot: &AppSnapshot) {}
}

impl Component for Footer {
    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let line = Line::from(vec![
            "q".red().bold(),
            Span::raw(" quit   "),
            "Enter".green().bold(),
            Span::raw(" edit/confirm   "),
            "Right".green().bold(),
            Span::raw(" next   "),
            "Left".yellow().bold(),
            Span::raw(" back   "),
            "Up/Down".cyan().bold(),
            Span::raw(" select   "),
            "Space".blue().bold(),
            Span::raw(" toggle"),
        ]);

        let paragraph = Paragraph::new(line).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" controls "),
        );
        f.render_widget(paragraph, rect);
    }
}