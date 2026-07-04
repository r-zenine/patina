use thiserror::Error;

#[derive(Debug, Error)]
pub enum TuiError {
    /// I/O errors from terminal and crossterm operations (crossterm uses std::io::Error).
    #[error("Terminal I/O error")]
    Terminal(#[from] std::io::Error),
    /// Infallible errors from backends that cannot fail (e.g. TestBackend in ratatui 0.30+).
    #[error("unreachable")]
    Infallible(#[from] std::convert::Infallible),

    /// Malformed compact input sequence (vim-style test notation).
    #[error("Invalid input sequence: {0}")]
    Parse(String),

    /// App-level error, wrapped to avoid leaking the app's error type into the harness.
    #[error("App error: {0}")]
    App(Box<dyn std::error::Error + Send + Sync + 'static>),
}

pub type Result<T> = std::result::Result<T, TuiError>;
