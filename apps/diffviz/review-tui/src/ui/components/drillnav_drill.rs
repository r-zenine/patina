//! DrillNav Drill view — inside a decision: pinned file header, dot
//! pagination for sibling files, chunk cards scrolling below.
//!
//! Adapts a chunk's `ReviewEngine`/`UiState`-derived data to
//! [`tui_design::drillnav::DrillChunk`] and renders through the generic
//! Drill renderer.

use diffviz_review::engines::ReviewEngine;
use diffviz_review::{Instruction, ReviewableDiffId};
use ratatui::prelude::*;
use tui_design::Theme;
use tui_design::drillnav::{self, CodeRow, DrillChunk, DrillChunkIcons, FileHeader, LineColor};

use super::drillnav_common::{
    annotations_for, line_change_type, line_has_change, note_for, note_rows,
};
use crate::state::UiState;
use crate::ui::icons::Icons;

struct ChunkAdapter<'a> {
    engine: &'a ReviewEngine,
    ui_state: &'a UiState,
    chunk_id: &'a ReviewableDiffId,
    index: usize,
    text_width: usize,
}

impl DrillChunk for ChunkAdapter<'_> {
    fn is_approved(&self) -> bool {
        self.engine.state().is_approved(self.chunk_id)
    }

    fn note_rows(&self) -> Option<Vec<String>> {
        let instr: &Instruction = note_for(self.engine, self.chunk_id)?;
        let note_expanded = self.ui_state.drill_chunk_note_expanded(self.index);
        let wrap_width = self.text_width.saturating_sub(6);
        let mut rows = note_rows(instr, wrap_width);
        let has_more = rows.len() > 1;
        if !note_expanded {
            let first = rows.into_iter().next().unwrap_or_default();
            rows = vec![if has_more {
                format!("{}…", first.trim_end_matches(' '))
            } else {
                first
            }];
        }
        Some(rows)
    }

    fn code_rows(&self) -> Vec<CodeRow> {
        let Some(renderable) = self.engine.get_renderable_diff_object(self.chunk_id) else {
            return Vec::new();
        };
        let chunk_expanded = self.ui_state.drill_chunk_expanded(self.index);
        renderable
            .lines
            .iter()
            .filter(|line| chunk_expanded || line_has_change(line))
            .map(|line| {
                let color = match line_change_type(line) {
                    Some(diffviz_core::renderable_diff::ChangeType::Added) => LineColor::Added,
                    Some(diffviz_core::renderable_diff::ChangeType::Deleted) => LineColor::Deleted,
                    _ => LineColor::Neutral,
                };
                CodeRow {
                    line_number: line.line_number,
                    color,
                    content: line.content.to_string(),
                }
            })
            .collect()
    }

    fn annotations(&self) -> Vec<drillnav::AnnotationRow> {
        if !self.ui_state.show_reasoning {
            return Vec::new();
        }
        annotations_for(self.engine, self.chunk_id)
            .into_iter()
            .map(|a| drillnav::AnnotationRow {
                trigger_line: a.trigger_line,
                label: a.label,
                reasoning: a.reasoning,
            })
            .collect()
    }
}

/// Render the Drill view. Must only be called while `UiState` is drilled in.
pub fn render(f: &mut Frame, area: Rect, ui_state: &UiState, engine: &ReviewEngine) {
    let (decision_idx, file_idx, cursor) = ui_state
        .drill_position()
        .expect("drill view rendered outside Drill mode");
    let theme = Theme::mocha();
    let cr = drillnav::content_rect(area);
    let text_width = (cr.width as usize).saturating_sub(4);

    let decisions = engine.get_all_decisions();
    let decision = decisions[decision_idx];
    let drill_decision = &ui_state.drill_index().decisions[decision_idx];
    let file = &drill_decision.files[file_idx];
    let chunk_ids = &file.chunks;

    // The code impact reasoning for this file. Index files derive from the
    // decision's impacts, so a missing impact is a broken invariant.
    let impact = decision
        .code_impacts
        .iter()
        .find(|ci| ci.file == file.path)
        .expect("file path from drill index should match a decision impact");

    let is_decision_approved = engine.is_decision_approved(decision.number);
    let header = FileHeader {
        label: format!("{} {}", Icons::FILE_MODIFIED, file.path),
        badge: is_decision_approved.then_some((Icons::APPROVED, theme.accents.green)),
        reasoning: impact.reasoning.clone(),
    };

    let total_siblings = drill_decision.files.len();
    let siblings = (total_siblings > 1).then_some((file_idx, total_siblings));

    let chunks: Vec<ChunkAdapter> = chunk_ids
        .iter()
        .enumerate()
        .map(|(index, chunk_id)| ChunkAdapter {
            engine,
            ui_state,
            chunk_id,
            index,
            text_width,
        })
        .collect();

    let page_offset = ui_state.drill_page_offset().unwrap_or(0);
    let icons = DrillChunkIcons {
        note_marker: (Icons::HAS_INSTRUCTIONS, theme.accents.yellow),
        approved_badge: (Icons::APPROVED, theme.accents.green),
    };

    drillnav::render_drill(
        f,
        area,
        header,
        siblings,
        &chunks,
        cursor,
        page_offset,
        &theme,
        &icons,
    );
}
