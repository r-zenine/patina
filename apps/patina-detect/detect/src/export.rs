use crate::entities::Symptom;
use serde::Serialize;

/// Decision-log-YAML-shaped debug export of a batch of symptoms — cheap
/// interop with existing decision-log tooling, not the model (spec.md:69-70:
/// "the decision-log YAML export stays available as a cheap interop/debug
/// format, but it is an export, not the model").
///
/// Deliberately defines its own minimal shape here rather than depending on
/// `diffviz-review::Decision`: `patina-detect` must not depend on
/// `diffviz-review` (that crate's Decision/Reviewable model is a UX
/// ancestor, not a dependency).
#[derive(Serialize)]
struct ExportedLog {
    commit: String,
    decisions: Vec<ExportedDecision>,
}

#[derive(Serialize)]
struct ExportedDecision {
    number: u32,
    title: String,
    code_impacts: Vec<ExportedCodeImpact>,
}

#[derive(Serialize)]
struct ExportedCodeImpact {
    file: String,
    reasoning: String,
    line_ranges: Vec<ExportedLineRange>,
}

#[derive(Serialize)]
struct ExportedLineRange {
    start: usize,
    end: usize,
}

/// Renders a batch of symptoms as decision-log-shaped YAML. `commit` has no
/// meaning for an audit run (symptoms aren't anchored to a single commit),
/// so callers pass whatever value their own export context has on hand.
pub fn export_symptom_log(commit: impl Into<String>, symptoms: &[Symptom]) -> String {
    let decisions = symptoms
        .iter()
        .enumerate()
        .map(|(index, symptom)| ExportedDecision {
            number: (index + 1) as u32,
            title: symptom.title.clone(),
            code_impacts: symptom
                .sites
                .iter()
                .map(|site| ExportedCodeImpact {
                    file: site.file.display().to_string(),
                    reasoning: site.note.clone(),
                    line_ranges: site
                        .line_ranges
                        .iter()
                        .map(|range| ExportedLineRange {
                            start: range.start,
                            end: range.end,
                        })
                        .collect(),
                })
                .collect(),
        })
        .collect();

    let log = ExportedLog {
        commit: commit.into(),
        decisions,
    };
    serde_yaml::to_string(&log).expect("serializing a symptom log to YAML cannot fail")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{DetectorId, Evidence, LineRange, Site, SiteRole, SymptomId};
    use std::path::PathBuf;

    #[test]
    fn export_produces_valid_decision_log_shaped_yaml() {
        let symptom = Symptom {
            id: SymptomId::new(DetectorId::new("house-rules"), b"fp"),
            detector: DetectorId::new("house-rules"),
            title: "no-unwrap-or-default violation".to_string(),
            evidence: Evidence::RuleMatch {
                rule_id: "no-unwrap-or-default".to_string(),
                matched_snippet: "foo.unwrap_or_default()".to_string(),
            },
            sites: vec![Site {
                file: PathBuf::from("apps/diffviz/core/src/lib.rs"),
                line_ranges: vec![LineRange { start: 10, end: 12 }],
                role: SiteRole::MatchSite,
                note: "silently defaults on parse failure".to_string(),
            }],
        };

        let yaml = export_symptom_log("[placeholder]", std::slice::from_ref(&symptom));
        let parsed: serde_yaml::Value =
            serde_yaml::from_str(&yaml).expect("export did not produce valid YAML");

        assert_eq!(parsed["commit"], "[placeholder]");
        assert_eq!(parsed["decisions"][0]["title"], symptom.title.as_str());
        assert_eq!(
            parsed["decisions"][0]["code_impacts"][0]["file"],
            "apps/diffviz/core/src/lib.rs"
        );
    }
}
