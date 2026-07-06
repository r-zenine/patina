use crate::entities::detector_id::DetectorId;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// Content-addressed identifier for a `Symptom`: `detector_id` plus a stable,
/// detector-supplied fingerprint (e.g. a clone group's subtree hash, a rule
/// id + normalized match text). Never derived from line numbers or ordinal
/// position, so the same underlying finding produces the same id across
/// re-runs and rebases — the baseline ratchet (see `persistence::Baseline`)
/// depends on that stability.
///
/// Each detector designs its own fingerprint bytes (spec.md's open question
/// 1); this type only owns the hashing contract that turns those bytes,
/// together with the detector id, into one comparable, hashable value.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SymptomId {
    detector: DetectorId,
    fingerprint_hash: String,
}

impl SymptomId {
    /// Builds a `SymptomId` from a detector id and that detector's own
    /// normalized fingerprint bytes. Hashing the fingerprint (rather than
    /// storing it verbatim) keeps the id a fixed-size, opaque, comparable
    /// value regardless of how large or shaped the fingerprint input is.
    pub fn new(detector: DetectorId, fingerprint: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(detector.as_str().as_bytes());
        hasher.update(fingerprint);
        let digest = hasher.finalize();
        let fingerprint_hash = digest.iter().map(|b| format!("{b:02x}")).collect();
        SymptomId {
            detector,
            fingerprint_hash,
        }
    }

    pub fn detector(&self) -> &DetectorId {
        &self.detector
    }
}

impl fmt::Display for SymptomId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.detector, self.fingerprint_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detector(id: &str) -> DetectorId {
        DetectorId::new(id)
    }

    #[test]
    fn same_detector_and_fingerprint_produce_the_same_id() {
        let a = SymptomId::new(detector("house-rules"), b"rule-1:let _ = foo();");
        let b = SymptomId::new(detector("house-rules"), b"rule-1:let _ = foo();");
        assert_eq!(a, b);
        assert_eq!(a.to_string(), b.to_string());
    }

    #[test]
    fn different_fingerprints_produce_different_ids() {
        let a = SymptomId::new(detector("house-rules"), b"rule-1:let _ = foo();");
        let b = SymptomId::new(detector("house-rules"), b"rule-2:let _ = bar();");
        assert_ne!(a, b);
    }

    #[test]
    fn different_detectors_with_the_same_fingerprint_bytes_produce_different_ids() {
        let a = SymptomId::new(detector("house-rules"), b"same-bytes");
        let b = SymptomId::new(detector("type2-clones"), b"same-bytes");
        assert_ne!(a, b);
    }
}
