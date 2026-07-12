use thiserror::Error;

#[derive(Debug, Error)]
pub enum TriageTuiError {
    #[error("key event dispatch failed: {0}")]
    KeyDispatch(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("command execution failed: {0}")]
    CommandExecution(Box<dyn std::error::Error + Send + Sync + 'static>),
}

#[allow(dead_code)]
fn _assert_triage_tui_error_bounds() {
    fn _check<E: std::error::Error + Send + Sync + 'static>() {}
    _check::<TriageTuiError>();
}
