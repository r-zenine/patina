use crate::entities::detector_id::DetectorId;
use crate::entities::evidence::Evidence;
use crate::entities::site::Site;
use crate::entities::symptom_id::SymptomId;
use serde::{Deserialize, Serialize};

/// A detector→triager finding: "here is what looks wrong" (spec.md's
/// `Concept` section). The semantics-reversed sibling of diffviz-review's
/// `Decision` ("here is why I did this").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Symptom {
    pub id: SymptomId,
    pub detector: DetectorId,
    pub title: String,
    pub evidence: Evidence,
    pub sites: Vec<Site>,
}
