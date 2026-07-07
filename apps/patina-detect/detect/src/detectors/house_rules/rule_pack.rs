use crate::entities::{DetectorId, Evidence, LineRange, Site, SiteRole, Symptom, SymptomId};
use ast_grep_config::{GlobalRules, RuleConfig, RuleConfigError, from_yaml_string};
use ast_grep_language::{LanguageExt, SupportLang};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// The detector id every house-rule `Symptom`/`SymptomId` is tagged with.
pub const DETECTOR_ID: &str = "house-rules";

/// The house-rule pack YAML, embedded at compile time so the binary is
/// self-contained. The same file is also valid input for the standalone
/// `ast-grep` CLI (`ast-grep scan -r rules/house-rules.yaml <path>`).
pub const RULE_PACK_YAML: &str = include_str!("../../../rules/house-rules.yaml");

#[derive(Debug, Error)]
pub enum HouseRulesError {
    #[error("failed to parse house-rules rule pack")]
    RulePack(#[from] RuleConfigError),

    #[error("failed to walk directory {path}")]
    Walk {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to read file {path}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// Runs the house-rules pack (embedded `RULE_PACK_YAML`) against every `.rs`
/// file found recursively under `root`, returning one `Symptom` per match.
pub fn run_house_rules(root: &Path) -> Result<Vec<Symptom>, HouseRulesError> {
    let globals = GlobalRules::default();
    let rules: Vec<RuleConfig<SupportLang>> = from_yaml_string(RULE_PACK_YAML, &globals)?;

    let mut symptoms = Vec::new();
    for file in collect_rust_files(root)? {
        let content = fs::read_to_string(&file).map_err(|source| HouseRulesError::Read {
            path: file.clone(),
            source,
        })?;
        let grep = SupportLang::Rust.ast_grep(&content);

        for rule in &rules {
            for m in grep.root().find_all(&rule.matcher) {
                let matched_snippet = m.text().to_string();
                let start_line = m.start_pos().line() + 1;
                let end_line = m.end_pos().line() + 1;
                let fingerprint = fingerprint_bytes(&rule.id, &matched_snippet);

                symptoms.push(Symptom {
                    id: SymptomId::new(DetectorId::new(DETECTOR_ID), &fingerprint),
                    detector: DetectorId::new(DETECTOR_ID),
                    title: rule.message.clone(),
                    evidence: Evidence::RuleMatch {
                        rule_id: rule.id.clone(),
                        matched_snippet: matched_snippet.clone(),
                    },
                    sites: vec![Site {
                        file: file.clone(),
                        line_ranges: vec![LineRange {
                            start: start_line,
                            end: end_line,
                        }],
                        role: SiteRole::MatchSite,
                        note: rule.message.clone(),
                    }],
                });
            }
        }
    }

    Ok(symptoms)
}

/// Builds this detector's fingerprint bytes: rule id + whitespace-normalized
/// match text. Whitespace-insensitive so trivial reformatting (rustfmt
/// re-wrapping a line, for instance) doesn't change the `SymptomId` — only
/// the matched code's actual token content does.
fn fingerprint_bytes(rule_id: &str, matched_snippet: &str) -> Vec<u8> {
    let normalized = matched_snippet
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    format!("{rule_id}:{normalized}").into_bytes()
}

fn collect_rust_files(root: &Path) -> Result<Vec<PathBuf>, HouseRulesError> {
    let mut files = Vec::new();
    visit(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn visit(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), HouseRulesError> {
    let entries = fs::read_dir(dir).map_err(|source| HouseRulesError::Walk {
        path: dir.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| HouseRulesError::Walk {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            visit(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diffviz_core_src() -> PathBuf {
        // Runs from apps/patina-detect/detect, so climb to the repo root.
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../libs/diffviz-core/src")
    }

    #[test]
    fn finds_unwrap_or_default_in_diffviz_core() {
        let symptoms = run_house_rules(&diffviz_core_src()).expect("detector run failed");
        assert!(
            symptoms
                .iter()
                .any(|s| matches!(&s.evidence, Evidence::RuleMatch { rule_id, .. } if rule_id == "no-unwrap-or-default")),
            "expected at least one no-unwrap-or-default symptom, found: {:#?}",
            symptoms
        );
    }

    #[test]
    fn finds_allow_dead_code_in_diffviz_core() {
        let symptoms = run_house_rules(&diffviz_core_src()).expect("detector run failed");
        assert!(
            symptoms
                .iter()
                .any(|s| matches!(&s.evidence, Evidence::RuleMatch { rule_id, .. } if rule_id == "no-allow-dead-code")),
            "expected at least one no-allow-dead-code symptom, found: {:#?}",
            symptoms
        );
    }

    #[test]
    fn finds_catchall_default_arm_in_diffviz_core() {
        let symptoms = run_house_rules(&diffviz_core_src()).expect("detector run failed");
        assert!(
            symptoms
                .iter()
                .any(|s| matches!(&s.evidence, Evidence::RuleMatch { rule_id, .. } if rule_id == "no-catchall-default-arm")),
            "expected at least one no-catchall-default-arm symptom, found: {:#?}",
            symptoms
        );
    }

    #[test]
    fn finds_format_in_error_value_in_diffviz_core() {
        let symptoms = run_house_rules(&diffviz_core_src()).expect("detector run failed");
        assert!(
            symptoms
                .iter()
                .any(|s| matches!(&s.evidence, Evidence::RuleMatch { rule_id, .. } if rule_id == "no-format-in-error-value")),
            "expected at least one no-format-in-error-value symptom, found: {:#?}",
            symptoms
        );
    }

    /// `no-let-underscore-result`, `no-stringly-typed-map-err`, and
    /// `no-todo-unimplemented` have zero live instances in diffviz-core
    /// today (confirmed by grep before writing these tests, and by the
    /// audit output showing no such symptoms). Rather than fake a fixture
    /// inside diffviz-core itself, these three are proven against a
    /// synthetic temp file — the acceptance criterion is "the detector
    /// finds it", not "diffviz-core currently violates every rule".
    fn write_fixture(content: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        std::fs::write(dir.path().join("fixture.rs"), content).expect("failed to write fixture");
        dir
    }

    #[test]
    fn finds_let_underscore_result_in_a_fixture() {
        let dir = write_fixture("fn f() -> Result<(), ()> { let _ = Ok::<(), ()>(()); Ok(()) }");
        let symptoms = run_house_rules(dir.path()).expect("detector run failed");
        assert!(
            symptoms
                .iter()
                .any(|s| matches!(&s.evidence, Evidence::RuleMatch { rule_id, .. } if rule_id == "no-let-underscore-result")),
            "expected a no-let-underscore-result symptom, found: {:#?}",
            symptoms
        );
    }

    #[test]
    fn finds_stringly_typed_map_err_in_a_fixture() {
        let dir = write_fixture(
            "fn f() -> Result<(), String> { std::fs::read(\"x\").map_err(|e| e.to_string())?; Ok(()) }",
        );
        let symptoms = run_house_rules(dir.path()).expect("detector run failed");
        assert!(
            symptoms
                .iter()
                .any(|s| matches!(&s.evidence, Evidence::RuleMatch { rule_id, .. } if rule_id == "no-stringly-typed-map-err")),
            "expected a no-stringly-typed-map-err symptom, found: {:#?}",
            symptoms
        );
    }

    #[test]
    fn finds_todo_unimplemented_in_a_fixture() {
        let dir = write_fixture("fn f() { todo!() }\nfn g() { unimplemented!() }");
        let symptoms = run_house_rules(dir.path()).expect("detector run failed");
        assert!(
            symptoms
                .iter()
                .any(|s| matches!(&s.evidence, Evidence::RuleMatch { rule_id, .. } if rule_id == "no-todo-unimplemented")),
            "expected a no-todo-unimplemented symptom, found: {:#?}",
            symptoms
        );
    }

    #[test]
    fn rerunning_with_no_code_change_produces_identical_symptom_ids() {
        let first = run_house_rules(&diffviz_core_src()).expect("first run failed");
        let second = run_house_rules(&diffviz_core_src()).expect("second run failed");

        let first_ids: Vec<_> = first.iter().map(|s| s.id.to_string()).collect();
        let second_ids: Vec<_> = second.iter().map(|s| s.id.to_string()).collect();
        assert_eq!(first_ids, second_ids);
    }
}
