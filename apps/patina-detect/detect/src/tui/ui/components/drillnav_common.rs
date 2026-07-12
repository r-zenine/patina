//! Triage-domain helpers for the DrillNav Drill view: mapping a
//! `RenderableLine`'s change annotation to `tui_design::drillnav::LineColor`.
//!
//! Generic rendering helpers (content width, word wrap, cards) live in
//! `tui_design::drillnav` — only the piece that reaches into
//! `diffviz-core`'s `RenderableLine`/`ChangeType` stays here.

use diffviz_core::RenderableLine;
use diffviz_core::renderable_diff::ChangeType;
use tui_design::drillnav::LineColor;

use crate::entities::Evidence;

pub fn line_color(line: &RenderableLine<'_>) -> LineColor {
    match line.annotations.first().and_then(|a| a.change_type.clone()) {
        Some(ChangeType::Added) => LineColor::Added,
        Some(ChangeType::Deleted) => LineColor::Deleted,
        _ => LineColor::Neutral,
    }
}

/// One-line rationale for a symptom's evidence. `Evidence` is
/// `#[non_exhaustive]` (more detector phases add variants), so this must
/// stay exhaustive-with-fallback rather than assuming `RuleMatch` forever.
pub fn evidence_rationale(evidence: &Evidence) -> String {
    match evidence {
        Evidence::RuleMatch {
            rule_id,
            matched_snippet,
        } => format!("[{rule_id}] {matched_snippet}"),
        _ => "(evidence detail not available for this detector)".to_string(),
    }
}
