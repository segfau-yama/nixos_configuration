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
pub struct WelcomePage {
    status_message: Option<String>,
}

pub fn page() -> Box<dyn InstallerPage> {
    Box::new(WelcomePage::default())
}

impl InstallerPage for WelcomePage {
    fn screen(&self) -> Screen {
        Screen::Welcome
    }

    fn sync(&mut self, snapshot: &AppSnapshot) {
        self.status_message = snapshot.status_message.clone();
    }
}

impl Component for WelcomePage {
    fn handle_key_events(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Right | KeyCode::Enter => Action::Navigate(Screen::GitHubLogin),
            _ => Action::Noop,
        }
    }

    fn render(&mut self, f: &mut Frame, rect: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Welcome ")
            .border_style(Style::default().fg(Color::Green));
        let inner = block.inner(rect);
        f.render_widget(block, rect);

        let mut lines = vec![
            Line::from(Span::styled(
                "JadeOS TUI Installer Prototype",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::default(),
            Line::from("This crate demonstrates a component-oriented ratatui architecture."),
            Line::from("Each screen is implemented as a page component first."),
            Line::from("Shared widgets are limited to the header, footer, and sidebar."),
            Line::default(),
            Line::from("Press Right or Enter to begin configuring the installer."),
        ];

        if let Some(message) = self.status_message.as_ref() {
            lines.push(Line::default());
            lines.push(Line::from(vec![
                Span::styled("status: ", Style::default().fg(Color::Yellow)),
                Span::raw(message.clone()),
            ]));
        }

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
        f.render_widget(paragraph, inner);
    }
}