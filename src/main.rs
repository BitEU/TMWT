mod app;
mod settings;
mod ui;
mod launcher;
mod event;

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::{io, panic, time::Duration};                                                                                                                                                           │
use crate::event::{Event, EventHandler};

fn main() -> Result<()> {
    // Setup panic handler to restore terminal
    setup_panic_handler();
    
    // Setup terminal
    let mut terminal = setup_terminal()?;
    
     // Create event handler                                                                                                                                                                     │
 │   let event_handler = EventHandler::new(Duration::from_millis(250));

    // Create app and run
    let app = app::App::new();
    let res = app::run_app(&mut terminal, app, event_handler);
    
    // Restore terminal
    restore_terminal(&mut terminal)?;
    
    if let Err(e) = res {
        eprintln!("Error: {:?}", e);
    }
    
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn setup_panic_handler() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal(&mut setup_terminal().unwrap());
        original_hook(panic_info);
    }));
}