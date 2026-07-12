//! DrillNav Drill view — inside a symptom: pinned header, site cards
//! scrolling below. `Symptom -> Site` is a flat 2-level hierarchy (no
//! sibling-file cycling), so `siblings: None` is always passed to
//! `render_drill` (roadmap Phase 0 guidance).
//!
//! Adapts a site's rendered `diffviz-core::ReviewableDiff` to
//! [`tui_design::drillnav::DrillChunk`] and renders through the generic
//! Drill renderer.

use ratatui::prelude::*;
use tui_design::Theme;
use tui_design::drillnav::{self, AnnotationRow, CodeRow, DrillChunk, DrillChunkIcons, FileHeader};

use super::drillnav_common::{evidence_rationale, line_color};
use crate::entities::Site;
use crate::tui::app::TriageData;
use crate::tui::state::UiState;
use crate::tui::ui::icons::Icons;
use diffviz_core::{RenderableDiff, ReviewableDiff};

struct SiteAdapter<'a> {
    site: &'a Site,
    core_diff: &'a ReviewableDiff,
    wrap_width: usize,
}

impl DrillChunk for SiteAdapter<'_> {
    /// No per-site verdict concept exists — verdicts are recorded per
    /// *symptom* (see `TriageVerdict` in `crate::entities`), not per
    /// site, so a site never has its own "approved" state to show.
    fn is_approved(&self) -> bool {
        false
    }

    fn note_rows(&self) -> Option<Vec<String>> {
        if self.site.note.trim().is_empty() {
            return None;
        }
        Some(drillnav::wrap_text(&self.site.note, self.wrap_width))
    }

    fn code_rows(&self) -> Vec<CodeRow> {
        let Ok(renderable) = RenderableDiff::try_from(self.core_diff) else {
            return Vec::new();
        };
        renderable
            .lines
            .iter()
            .map(|line| CodeRow {
                line_number: line.line_number,
                color: line_color(line),
                content: line.content.to_string(),
            })
            .collect()
    }

    fn annotations(&self) -> Vec<AnnotationRow> {
        Vec::new()
    }
}

/// Render the Drill view. Must only be called while `UiState` is drilled in.
pub fn render(f: &mut Frame, area: Rect, ui_state: &UiState, data: &TriageData) {
    let (symptom_idx, cursor) = ui_state
        .drill_position()
        .expect("drill view rendered outside Drill mode");
    let theme = Theme::mocha();
    let cr = drillnav::content_rect(area);
    let text_width = (cr.width as usize).saturating_sub(4);

    let symptom = &data.symptoms[symptom_idx];
    let core_diffs = &data.core_diffs[symptom_idx];

    let header = FileHeader {
        label: format!("{} {}", Icons::SITE, symptom.title),
        badge: None,
        reasoning: evidence_rationale(&symptom.evidence),
    };

    let sites: Vec<SiteAdapter> = symptom
        .sites
        .iter()
        .zip(core_diffs.iter())
        .map(|(site, core_diff)| SiteAdapter {
            site,
            core_diff,
            wrap_width: text_width.saturating_sub(6),
        })
        .collect();

    let page_offset = ui_state.drill_page_offset().unwrap_or(0);
    let icons = DrillChunkIcons {
        note_marker: (Icons::NOTE, theme.accents.yellow),
        approved_badge: (Icons::SITE, theme.accents.green),
    };

    drillnav::render_drill(
        f,
        area,
        header,
        None,
        &sites,
        cursor,
        page_offset,
        &theme,
        &icons,
    );
}
