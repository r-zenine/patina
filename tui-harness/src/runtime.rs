//! Production TUI runtime — terminal setup, 60fps event loop, teardown.

use std::time::Duration;

use crossterm::{
    cursor::Show,
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::{Result, TuiError, traits::ELMApp};

/// Restores the terminal on drop, so teardown runs on normal exit, on error,
/// and on panic in `dispatch_key`/`draw`.
struct TerminalGuard;

impl TerminalGuard {
    fn new() -> Result<Self> {
        enable_raw_mode()?;
        // Guard exists before entering the alternate screen so raw mode is
        // restored even if entering fails.
        let guard = Self;
        execute!(std::io::stdout(), EnterAlternateScreen)?;
        Ok(guard)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Best-effort: a teardown failure must not mask the app's own result.
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen, Show);
    }
}

/// Run an ELMApp on a real terminal.
///
/// Sets up crossterm raw mode + alternate screen, runs a 60fps event loop,
/// and tears everything down on exit — including on error or panic.
pub fn run_app<M: ELMApp>(app: &mut M) -> Result<()> {
    let _guard = TerminalGuard::new()?;
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    event_loop(&mut terminal, app)
}

fn event_loop<M: ELMApp>(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut M,
) -> Result<()> {
    let frame_duration = Duration::from_millis(16); // ~60fps

    loop {
        app.on_tick();
        terminal.draw(|f| app.draw(f))?;

        // Press only: Windows emits Press + Release for every key.
        if event::poll(frame_duration)?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            app.dispatch_key(key)
                .map_err(|e| TuiError::App(Box::new(e)))?;
        }

        if app.should_quit() {
            break;
        }
    }

    Ok(())
}
