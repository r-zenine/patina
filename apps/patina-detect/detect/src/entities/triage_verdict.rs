use serde::{Deserialize, Serialize};

/// Why a `Symptom` was dismissed rather than fixed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DismissReason {
    FalsePositive,
    Intentional,
    AcceptedDebt,
}

/// A triager's verdict on a `Symptom`. Persisted keyed by `SymptomId` in the
/// baseline (see `persistence::Baseline`) so re-runs don't resurface
/// already-triaged findings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriageVerdict {
    Dismissed { reason: DismissReason },
    Fix { instruction: String },
}
