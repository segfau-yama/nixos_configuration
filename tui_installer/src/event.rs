use color_eyre::eyre::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyModifiers};

#[derive(Debug, Clone)]
pub enum Event {
    Quit,
    Key(crossterm::event::KeyEvent),
    Mouse(crossterm::event::MouseEvent),
}

pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn next(&mut self) -> Result<Option<Event>> {
        Ok(match event::read()? {
            CrosstermEvent::Key(key)
                if key.code == KeyCode::Char('c')
                    && key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                Some(Event::Quit)
            }
            CrosstermEvent::Key(key) => Some(Event::Key(key)),
            CrosstermEvent::Mouse(mouse) => Some(Event::Mouse(mouse)),
            CrosstermEvent::Resize(_, _)
            | CrosstermEvent::FocusGained
            | CrosstermEvent::FocusLost
            | CrosstermEvent::Paste(_) => None,
        })
    }
}
