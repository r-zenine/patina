use crate::entities::{SymptomId, TriageVerdict};
use persistkit::{AssociativeStateWithTTL, ErrorAssociativeState};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BaselineError {
    #[error("failed to open triage baseline store")]
    Open(#[source] ErrorAssociativeState),

    #[error("failed to read verdict from triage baseline")]
    Read(#[source] ErrorAssociativeState),

    #[error("failed to write verdict to triage baseline")]
    Write(#[source] ErrorAssociativeState),
}

/// The triage-state store: persisted `TriageVerdict`s keyed by `SymptomId`.
/// This is the suppression baseline (spec.md's "Baseline ratchet" design
/// rule) — re-runs surface only symptoms with no verdict recorded here.
///
/// Backed by `persistkit::AssociativeStateWithTTL`, the same file-backed
/// key/value primitive already used elsewhere in the workspace, with no TTL:
/// a triage verdict is a durable decision, not a cache entry that should
/// expire.
pub struct Baseline {
    store: AssociativeStateWithTTL<TriageVerdict>,
}

impl Baseline {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, BaselineError> {
        Ok(Baseline {
            store: AssociativeStateWithTTL::new(path).map_err(BaselineError::Open)?,
        })
    }

    pub fn verdict_for(&self, id: &SymptomId) -> Result<Option<TriageVerdict>, BaselineError> {
        self.store.get(id.to_string()).map_err(BaselineError::Read)
    }

    pub fn record_verdict(
        &self,
        id: &SymptomId,
        verdict: TriageVerdict,
    ) -> Result<(), BaselineError> {
        self.store
            .put(id.to_string(), verdict)
            .map_err(BaselineError::Write)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{DetectorId, DismissReason};
    use tempfile::NamedTempFile;

    fn temp_baseline() -> Baseline {
        let f = NamedTempFile::new().expect("failed to create temp file");
        Baseline::open(f.path()).expect("failed to open baseline")
    }

    #[test]
    fn symptom_with_no_verdict_has_none() {
        let baseline = temp_baseline();
        let id = SymptomId::new(DetectorId::new("house-rules"), b"fingerprint");

        assert_eq!(baseline.verdict_for(&id).expect("read failed"), None);
    }

    #[test]
    fn recorded_verdict_is_retrievable() {
        let baseline = temp_baseline();
        let id = SymptomId::new(DetectorId::new("house-rules"), b"fingerprint");
        let verdict = TriageVerdict::Dismissed {
            reason: DismissReason::FalsePositive,
        };

        baseline
            .record_verdict(&id, verdict.clone())
            .expect("write failed");

        assert_eq!(
            baseline.verdict_for(&id).expect("read failed"),
            Some(verdict)
        );
    }
}
