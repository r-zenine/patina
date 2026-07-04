use crate::manifest::{Affordance, AppDescription};

/// ELM-architecture trait for TUI applications.
///
/// Implementing this trait provides a production runtime (`run_app`),
/// headless test harnesses (`InputTestHarness`, `RenderTestHarness`,
/// `CombinedTestHarness`), and the agent-facing discovery CLI
/// (`run_agent_cli` / `--describe`).
pub trait ELMApp {
    /// Serializable state snapshot for test assertions and debug modes.
    /// The `JsonSchema` bound lets `--describe` publish the snapshot's
    /// schema so agents can interpret snapshots without reading source.
    type Snapshot: serde::Serialize + schemars::JsonSchema;

    /// App-owned error type — no framework type is imposed.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Process a raw key event, mutating internal state.
    fn dispatch_key(
        &mut self,
        key: crossterm::event::KeyEvent,
    ) -> std::result::Result<(), Self::Error>;

    /// Draw current state into a ratatui Frame.
    fn draw(&self, frame: &mut ratatui::Frame);

    /// Whether the app should stop running (drives the run_app loop).
    fn should_quit(&self) -> bool;

    /// Capture current state as a serializable snapshot.
    fn snapshot(&self) -> Self::Snapshot;

    /// Called once per frame before event polling.
    ///
    /// Use for time-based logic (e.g. leader key timeouts) that must fire
    /// even when no key event arrives. Default is a no-op.
    fn on_tick(&mut self) {}

    /// Static discovery data: app name/version, modes, keybindings.
    ///
    /// Default `None` marks the app as undescribed in the manifest — agents
    /// can tell "no discovery data" apart from an unsupported flag.
    fn describe(&self) -> Option<AppDescription> {
        None
    }

    /// Keys that are meaningful in the app's *current* state.
    ///
    /// Default is empty. Apps with mode-dependent bindings should override
    /// this so agents get a closed loop: act, then observe the new state
    /// and the new legal moves.
    fn affordances(&self) -> Vec<Affordance> {
        Vec::new()
    }
}
