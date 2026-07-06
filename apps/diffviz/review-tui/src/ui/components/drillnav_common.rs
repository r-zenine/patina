//! Review-domain helpers for the DrillNav views: notes and reasoning
//! annotations, coupled to `ReviewEngine`/`Decision`/`Instruction`.
//!
//! Generic rendering helpers (content width, word wrap, cards, dot
//! pagination) live in `tui_design::drillnav` — only the pieces that
//! actually reach into the review domain stay here.

use diffviz_core::RenderableLine;
use diffviz_core::renderable_diff::ChangeType;
use diffviz_review::{Instruction, ReviewableDiffId, engines::ReviewEngine};
use tui_design::drillnav::wrap_text;

/// The chunk's single note. One note per chunk is a product invariant —
/// adding an instruction to an annotated chunk appends to the existing note.
pub fn note_for<'a>(
    engine: &'a ReviewEngine,
    chunk_id: &ReviewableDiffId,
) -> Option<&'a Instruction> {
    engine
        .state()
        .get_instructions(chunk_id)
        .and_then(|v| v.first())
}

/// Wrapped display rows for a note: authors prefix the first contribution;
/// each appended contribution starts on its own row (note content is
/// newline-separated under the single-note model).
pub fn note_rows(instr: &Instruction, wrap_width: usize) -> Vec<String> {
    let text = format!("{}: {}", instr.author, instr.content);
    text.split('\n')
        .flat_map(|segment| wrap_text(segment, wrap_width))
        .collect()
}

pub fn line_change_type(line: &RenderableLine<'_>) -> Option<ChangeType> {
    line.annotations.first().and_then(|a| a.change_type.clone())
}

pub fn line_has_change(line: &RenderableLine<'_>) -> bool {
    line.annotations
        .first()
        .and_then(|a| a.change_type.as_ref())
        .is_some()
}

/// A decision's reasoning, anchored to the line (relative to the chunk,
/// matching `RenderableLine::line_number`) it should render above. Ported
/// from the old diff_view (plan-inline-reasoning-annotations D1/D3): pure
/// view-layer computation over `Decision.code_impacts`, no domain changes.
pub struct ReasoningAnnotation {
    /// Chunk-relative line number (1-based) the annotation renders above —
    /// matches `RenderableLine::line_number`, not the absolute file line.
    pub trigger_line: usize,
    /// Decision label, e.g. "D1".
    pub label: String,
    /// `CodeImpact.reasoning` text.
    pub reasoning: String,
}

/// Reasoning annotations for a chunk, one per decision that references this
/// chunk's file — a chunk can map to more than one decision
/// (`get_decisions_for_diff` returns a `Vec`).
pub fn annotations_for(
    engine: &ReviewEngine,
    chunk_id: &ReviewableDiffId,
) -> Vec<ReasoningAnnotation> {
    let diff_start = engine
        .get_renderable_diff_object(chunk_id)
        .map(|d| d.metadata.overall_line_range.start_line)
        .unwrap_or(chunk_id.line_range.start_line);

    engine
        .get_decisions_for_diff(chunk_id)
        .iter()
        .flat_map(|decision| {
            decision
                .code_impacts
                .iter()
                .filter(|impact| impact.file == chunk_id.file_path)
                .map(|impact| {
                    let abs_trigger = impact
                        .line_ranges
                        .iter()
                        .map(|r| r.start)
                        .min()
                        .unwrap_or(chunk_id.line_range.start_line)
                        .max(diff_start);
                    let trigger_line = abs_trigger.saturating_sub(diff_start).saturating_add(1);
                    ReasoningAnnotation {
                        trigger_line,
                        label: format!("D{}", decision.number),
                        reasoning: impact.reasoning.clone(),
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
