use std::time::Duration;

use color_eyre::eyre::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyModifiers};

#[derive(Debug, Clone)]
pub enum Event {
    Quit,
    Tick,
    Key(crossterm::event::KeyEvent),
    Mouse(crossterm::event::MouseEvent),
}

pub struct EventHandler {
    tick_rate: Duration,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        Self { tick_rate }
    }

    pub fn next(&mut self) -> Result<Option<Event>> {
        if !event::poll(self.tick_rate)? {
            return Ok(Some(Event::Tick));
        }

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
