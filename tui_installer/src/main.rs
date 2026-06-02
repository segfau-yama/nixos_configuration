mod action;
mod app;
mod component;
mod components;
mod config;
mod event;
mod pages;
mod terminal;

use std::{
    io::{self, stdout},
    time::Duration,
};

use color_eyre::eyre::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::{app::App, event::EventHandler};

type TuiTerminal = Terminal<CrosstermBackend<io::Stdout>>;

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = setup_terminal()?;
    let result = run(&mut terminal);
    restore_terminal(&mut terminal)?;
    result
}

fn run(terminal: &mut TuiTerminal) -> Result<()> {
    let mut app = App::new()?;
    let mut events = EventHandler::new(Duration::from_millis(80));

    while !app.should_quit {
        let event = events.next()?;
        let action = app.handle_event(event);
        app.update(action);
        terminal.draw(|frame| app.render(frame))?;
    }

    Ok(())
}

fn setup_terminal() -> Result<TuiTerminal> {
    enable_raw_mode()?;
    let mut output = stdout();
    execute!(output, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(output);
    Ok(Terminal::new(backend)?)
}

fn restore_terminal(terminal: &mut TuiTerminal) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
