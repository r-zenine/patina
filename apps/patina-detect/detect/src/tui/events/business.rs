//! Business events that require the triage baseline.
//!
//! Unlike `diffviz-review-tui` (which resolves its target IDs from
//! `UiState` alone, since `ReviewableDiffId`s are precomputed into the
//! state's drill index), a `SymptomId` here depends on `TriageApp::symptoms`
//! — `UiState` only tracks the *index* of the focused symptom. So
//! `ui_event_to_business_event` takes the already-resolved id from the
//! caller (`app.rs`) rather than reaching into a data structure `UiState`
//! doesn't own.

use crate::entities::{DismissReason, SymptomId, TriageVerdict};
use crate::tui::events::UiEvent;

/// Business events that require baseline operations.
#[derive(Debug, Clone)]
pub enum BusinessEvent {
    /// Record a triage verdict for a symptom.
    RecordVerdict {
        symptom_id: SymptomId,
        verdict: TriageVerdict,
    },
}

/// Convert a UI event to a business event, given the symptom currently in
/// focus (if any) and the pending fix-instruction text.
pub fn ui_event_to_business_event(
    ui_event: &UiEvent,
    focused_symptom_id: Option<&SymptomId>,
    fix_instruction_symptom_id: Option<&SymptomId>,
    input_buffer: &str,
) -> Option<BusinessEvent> {
    match ui_event {
        UiEvent::DismissFalsePositive => {
            focused_symptom_id.map(|id| BusinessEvent::RecordVerdict {
                symptom_id: id.clone(),
                verdict: TriageVerdict::Dismissed {
                    reason: DismissReason::FalsePositive,
                },
            })
        }
        UiEvent::DismissIntentional => focused_symptom_id.map(|id| BusinessEvent::RecordVerdict {
            symptom_id: id.clone(),
            verdict: TriageVerdict::Dismissed {
                reason: DismissReason::Intentional,
            },
        }),
        UiEvent::DismissAcceptedDebt => focused_symptom_id.map(|id| BusinessEvent::RecordVerdict {
            symptom_id: id.clone(),
            verdict: TriageVerdict::Dismissed {
                reason: DismissReason::AcceptedDebt,
            },
        }),
        UiEvent::SubmitInput => fix_instruction_symptom_id.map(|id| BusinessEvent::RecordVerdict {
            symptom_id: id.clone(),
            verdict: TriageVerdict::Fix {
                instruction: input_buffer.to_string(),
            },
        }),
        _ => None,
    }
}
