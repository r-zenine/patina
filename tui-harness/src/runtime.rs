//! Production TUI runtime — terminal setup, 60fps event loop, teardown.

use std::time::Duration;

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::{Result, TuiError, traits::ELMApp};

/// Run an ELMApp on a real terminal.
///
/// Sets up crossterm raw mode + alternate screen, runs a 60fps event loop,
/// and tears everything down on exit — even if `dispatch_key` returns an error.
pub fn run_app<M: ELMApp>(app: &mut M) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = event_loop(&mut terminal, app);

    // Always restore terminal, even on error.
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn event_loop<M: ELMApp>(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut M,
) -> Result<()> {
    let frame_duration = Duration::from_millis(16); // ~60fps

    loop {
        terminal.draw(|f| app.draw(f))?;

        if event::poll(frame_duration)? {
            if let Event::Key(key) = event::read()? {
                app.dispatch_key(key)
                    .map_err(|e| TuiError::App(Box::new(e)))?;
            }
        }

        if app.should_quit() {
            break;
        }
    }

    Ok(())
}
