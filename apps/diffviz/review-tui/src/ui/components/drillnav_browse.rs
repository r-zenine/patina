//! DrillNav Browse view — top-level decisions as hierarchical cards.
//!
//! Adapts `ReviewEngine`'s decisions to [`tui_design::drillnav::DrillGroup`]
//! and renders through the generic Browse renderer.

use diffviz_review::engines::ReviewEngine;
use ratatui::prelude::*;
use tui_design::Theme;
use tui_design::drillnav::{self, DrillGroup, GroupHeader};

use crate::state::UiState;
use crate::ui::icons::Icons;

struct DecisionGroup<'a> {
    engine: &'a ReviewEngine,
    ui_state: &'a UiState,
    index: usize,
}

impl DrillGroup for DecisionGroup<'_> {
    fn header(&self) -> GroupHeader {
        let decisions = self.engine.get_all_decisions();
        let decision = decisions[self.index];
        let is_approved = self.engine.is_decision_approved(decision.number);
        let index_files = &self.ui_state.drill_index().decisions[self.index].files;
        let n_files = index_files.len();
        let (chunks_approved, chunks_total) =
            self.engine.decision_approval_progress(decision.number);
        let theme = Theme::mocha();

        GroupHeader {
            title: format!("#{} {}", decision.number, decision.title),
            badge: is_approved.then_some((Icons::APPROVED, theme.accents.green)),
            meta: format!(
                "{} file{} · {}/{} chunks approved",
                n_files,
                drillnav::plural_s(n_files),
                chunks_approved,
                chunks_total,
            ),
        }
    }

    fn rationale(&self) -> Option<String> {
        let decisions = self.engine.get_all_decisions();
        decisions[self.index].rationale.clone()
    }

    fn children_preview(&self) -> Vec<String> {
        self.ui_state.drill_index().decisions[self.index]
            .files
            .iter()
            .map(|file| format!("{} {}", Icons::FILE_MODIFIED, file.path))
            .collect()
    }
}

/// Render the Browse view. Must only be called while `UiState` is browsing.
pub fn render(f: &mut Frame, area: Rect, ui_state: &UiState, engine: &ReviewEngine) {
    let cursor = ui_state
        .browse_cursor()
        .expect("browse view rendered outside Browse mode");
    let n = engine.get_all_decisions().len();
    let groups: Vec<DecisionGroup> = (0..n)
        .map(|index| DecisionGroup {
            engine,
            ui_state,
            index,
        })
        .collect();
    let theme = Theme::mocha();
    drillnav::render_browse(f, area, &groups, cursor, &theme);
}
