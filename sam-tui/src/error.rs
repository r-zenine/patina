use thiserror::Error;

#[derive(Debug, Error)]
pub enum SamTuiError {
    #[error("terminal I/O error: {0}")]
    Terminal(#[from] std::io::Error),
}
