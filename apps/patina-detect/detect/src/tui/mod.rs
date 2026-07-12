//! Interactive triage TUI for `patina-detect` findings, bootstrapped on
//! `tui-harness`/`tui-elm`/`tui-design`. Browse untriaged symptoms, drill
//! into their sites (rendered via `diffviz-core`'s range-based rendering),
//! and record `Dismissed`/`Fix` verdicts that persist through the baseline.

pub mod app;
pub mod command;
pub mod error;
pub mod events;
pub mod rendering;
pub mod state;
pub mod state_snapshot;
pub mod ui;

pub use app::{TriageApp, TriageData};
pub use error::TriageTuiError;
pub use state::{InputMode, UiState};
pub use state_snapshot::StateSnapshot;

pub(crate) type Result<T, E = anyhow::Error> = std::result::Result<T, E>;
