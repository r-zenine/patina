/// ELM-architecture trait for TUI applications.
///
/// Implementing this trait provides both a production runtime (`run_app`) and
/// headless test harnesses (`InputTestHarness`, `RenderTestHarness`, `CombinedTestHarness`).
pub trait ELMApp {
    /// Serializable state snapshot for test assertions and debug modes.
    type Snapshot: serde::Serialize;

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
}
