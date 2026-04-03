use thiserror::Error;

#[derive(Debug, Error)]
pub enum TuiError {
    /// I/O errors from terminal and crossterm operations (crossterm uses std::io::Error).
    #[error("Terminal I/O error")]
    Terminal(#[from] std::io::Error),

    /// App-level error, wrapped to avoid leaking the app's error type into the harness.
    #[error("App error: {0}")]
    App(Box<dyn std::error::Error + Send + Sync + 'static>),
}

pub type Result<T> = std::result::Result<T, TuiError>;
