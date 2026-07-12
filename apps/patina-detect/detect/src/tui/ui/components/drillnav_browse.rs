//! DrillNav Browse view — untriaged symptoms as hierarchical cards.
//!
//! Adapts `TriageData`'s symptoms to [`tui_design::drillnav::DrillGroup`] and
//! renders through the generic Browse renderer.

use ratatui::prelude::*;
use tui_design::Theme;
use tui_design::drillnav::{self, DrillGroup, GroupHeader};

use super::drillnav_common::evidence_rationale;
use crate::entities::Symptom;
use crate::tui::app::TriageData;
use crate::tui::state::UiState;
use crate::tui::ui::icons::Icons;

struct SymptomGroup<'a> {
    symptom: &'a Symptom,
}

impl DrillGroup for SymptomGroup<'_> {
    fn header(&self) -> GroupHeader {
        let n_sites = self.symptom.sites.len();
        GroupHeader {
            title: self.symptom.title.clone(),
            badge: None,
            meta: format!(
                "{} · {} site{}",
                self.symptom.detector,
                n_sites,
                drillnav::plural_s(n_sites)
            ),
        }
    }

    fn rationale(&self) -> Option<String> {
        Some(evidence_rationale(&self.symptom.evidence))
    }

    fn children_preview(&self) -> Vec<String> {
        self.symptom
            .sites
            .iter()
            .map(|site| {
                let range_label = site
                    .line_ranges
                    .first()
                    .map(|r| format!(":{}-{}", r.start, r.end))
                    .unwrap_or_default();
                format!("{} {}{}", Icons::SITE, site.file.display(), range_label)
            })
            .collect()
    }
}

/// Render the Browse view. Must only be called while `UiState` is browsing.
pub fn render(f: &mut Frame, area: Rect, ui_state: &UiState, data: &TriageData) {
    let cursor = ui_state
        .browse_cursor()
        .expect("browse view rendered outside Browse mode");
    let groups: Vec<SymptomGroup> = data
        .symptoms
        .iter()
        .map(|symptom| SymptomGroup { symptom })
        .collect();
    let theme = Theme::mocha();
    drillnav::render_browse(f, area, &groups, cursor, &theme);
}
