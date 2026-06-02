use std::time::{Duration, Instant};

use color_eyre::eyre::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyModifiers};

#[derive(Debug, Clone)]
pub enum Event {
    Tick,
    Quit,
    Key(crossterm::event::KeyEvent),
    Mouse(crossterm::event::MouseEvent),
    Resize(u16, u16),
    FocusGained,
    FocusLost,
    Paste(String),
}

pub struct EventHandler {
    tick_rate: Duration,
    last_tick: Instant,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        Self {
            tick_rate,
            last_tick: Instant::now(),
        }
    }

    pub fn next(&mut self) -> Result<Option<Event>> {
        let timeout = self
            .tick_rate
            .saturating_sub(self.last_tick.elapsed());

        if event::poll(timeout)? {
            let next = match event::read()? {
                CrosstermEvent::Key(key)
                    if key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    Some(Event::Quit)
                }
                CrosstermEvent::Key(key) => Some(Event::Key(key)),
                CrosstermEvent::Mouse(mouse) => Some(Event::Mouse(mouse)),
                CrosstermEvent::Resize(x, y) => Some(Event::Resize(x, y)),
                CrosstermEvent::FocusGained => Some(Event::FocusGained),
                CrosstermEvent::FocusLost => Some(Event::FocusLost),
                CrosstermEvent::Paste(text) => Some(Event::Paste(text)),
            };
            return Ok(next);
        }

        if self.last_tick.elapsed() >= self.tick_rate {
            self.last_tick = Instant::now();
            return Ok(Some(Event::Tick));
        }

        Ok(None)
    }
}