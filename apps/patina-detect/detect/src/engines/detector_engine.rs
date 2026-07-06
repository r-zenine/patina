use crate::entities::Symptom;
use crate::persistence::{Baseline, BaselineError};

/// Runs a detector's raw findings against the persisted baseline, returning
/// only the symptoms that have no recorded verdict yet. Detector-agnostic:
/// every detector phase (house-rule violations, clones, ...) feeds its own
/// `Vec<Symptom>` through the same filtering step.
pub struct DetectorEngine<'a> {
    baseline: &'a Baseline,
}

impl<'a> DetectorEngine<'a> {
    pub fn new(baseline: &'a Baseline) -> Self {
        DetectorEngine { baseline }
    }

    pub fn run(&self, symptoms: Vec<Symptom>) -> Result<Vec<Symptom>, BaselineError> {
        symptoms
            .into_iter()
            .filter_map(|symptom| match self.baseline.verdict_for(&symptom.id) {
                Ok(None) => Some(Ok(symptom)),
                Ok(Some(_)) => None,
                Err(err) => Some(Err(err)),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{DetectorId, DismissReason, Evidence, SymptomId, TriageVerdict};
    use tempfile::NamedTempFile;

    fn synthetic_symptom(fingerprint: &[u8]) -> Symptom {
        Symptom {
            id: SymptomId::new(DetectorId::new("house-rules"), fingerprint),
            detector: DetectorId::new("house-rules"),
            title: "synthetic finding".to_string(),
            evidence: Evidence::RuleMatch {
                rule_id: "no-unwrap-or-default".to_string(),
                matched_snippet: "foo.unwrap_or_default()".to_string(),
            },
            sites: vec![],
        }
    }

    fn temp_baseline() -> Baseline {
        let f = NamedTempFile::new().expect("failed to create temp file");
        Baseline::open(f.path()).expect("failed to open baseline")
    }

    #[test]
    fn symptom_with_no_verdict_passes_through() {
        let baseline = temp_baseline();
        let engine = DetectorEngine::new(&baseline);
        let symptom = synthetic_symptom(b"a");

        let result = engine.run(vec![symptom.clone()]).expect("run failed");

        assert_eq!(result, vec![symptom]);
    }

    #[test]
    fn symptom_with_persisted_verdict_is_excluded() {
        let baseline = temp_baseline();
        let symptom = synthetic_symptom(b"b");
        baseline
            .record_verdict(
                &symptom.id,
                TriageVerdict::Dismissed {
                    reason: DismissReason::Intentional,
                },
            )
            .expect("record failed");

        let engine = DetectorEngine::new(&baseline);
        let result = engine.run(vec![symptom]).expect("run failed");

        assert!(result.is_empty());
    }

    #[test]
    fn mixed_batch_keeps_only_untriaged_symptoms() {
        let baseline = temp_baseline();
        let triaged = synthetic_symptom(b"c");
        let untriaged = synthetic_symptom(b"d");
        baseline
            .record_verdict(
                &triaged.id,
                TriageVerdict::Fix {
                    instruction: "use ? instead".to_string(),
                },
            )
            .expect("record failed");

        let engine = DetectorEngine::new(&baseline);
        let result = engine
            .run(vec![triaged, untriaged.clone()])
            .expect("run failed");

        assert_eq!(result, vec![untriaged]);
    }
}
