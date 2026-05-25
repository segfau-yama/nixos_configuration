use std::io;

use crossterm::event::{self, Event, KeyEvent};

pub fn read_key_event() -> io::Result<Option<KeyEvent>> {
    match event::read()? {
        Event::Key(key) => Ok(Some(key)),
        _ => Ok(None),
    }
}
