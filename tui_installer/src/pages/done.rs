use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::{
    action::Action,
    app::{AppSnapshot, Screen},
    component::Component,
    pages::InstallerPage,
    terminal::Frame,
};

#[derive(Default)]
pub struct DonePage {
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
        self.status_message = snapshot.status_message.clone();
    }
}

impl Component for DonePage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Left | KeyCode::Right | KeyCode::Enter => Action::Navigate(Screen::Summary),
            _ => Action::Noop,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Done ")
            .border_style(Style::default().fg(Color::Green));
        let inner = block.inner(rect);
        f.render_widget(block, rect);

        let mut lines = vec![
            Line::from(Span::styled(
                "Install flow complete",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::default(),
            Line::from("This prototype stops after a simulated install log."),
            Line::from("Press Enter to return to Summary or q to quit."),
        ];
        if let Some(message) = self.status_message.as_ref() {
            lines.push(Line::default());
            lines.push(Line::from(vec![
                Span::styled("status: ", Style::default().fg(Color::Yellow)),
                Span::raw(message.clone()),
            ]));
        }

        f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
    }
}