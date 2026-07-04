//! Phase 0 characterization tests for `handle_key_event` dispatch.
//!
//! These tests pin the exact `(input mode, leader state, key) → UiEvent`
//! mapping BEFORE the keybindings-as-data registry refactor (Phase 3 of
//! plan-tui-harness-agent-discovery). They must pass **unchanged** against
//! both the current match-arm dispatch and the future registry dispatch —
//! any edit to this file during that refactor is a red flag, not a fix.
//!
//! Deliberately characterized quirks (current behavior, pinned as-is):
//! - Leader dispatch matches on `key.code` only, so modifiers are ignored
//!   (`Ctrl-a` opens the actions submenu just like `a`).
//! - Any unknown key in leader mode silently deactivates the leader.
//! - `G` requires the SHIFT modifier; a bare `Char('G')` maps to nothing.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use diffviz_review::{DiffQuery, GitRef, LineRange, ReviewableDiffId};
use diffviz_review_tui::events::{UiEvent, handle_key_event};
use diffviz_review_tui::state::InputMode;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn key_with(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
}

fn nav(k: KeyEvent) -> Option<UiEvent> {
    handle_key_event(k, &InputMode::Navigation, false, None)
}

fn leader(k: KeyEvent, submenu: Option<char>) -> Option<UiEvent> {
    handle_key_event(k, &InputMode::Navigation, true, submenu)
}

fn input(k: KeyEvent) -> Option<UiEvent> {
    handle_key_event(
        k,
        &InputMode::DecisionInstruction { decision_number: 1 },
        false,
        None,
    )
}

// ============================================================================
// Navigation mode
// ============================================================================

#[test]
fn navigation_bindings_are_stable() {
    let cases: Vec<(KeyEvent, UiEvent)> = vec![
        // Application controls
        (key(KeyCode::Char('q')), UiEvent::Quit),
        (
            key_with(KeyCode::Char('c'), KeyModifiers::CONTROL),
            UiEvent::Quit,
        ),
        // Overlays / leader
        (
            key_with(KeyCode::Char('?'), KeyModifiers::SHIFT),
            UiEvent::ToggleHelp,
        ),
        (key(KeyCode::Char(' ')), UiEvent::ActivateLeader),
        (key(KeyCode::Esc), UiEvent::Back),
        // Vim-style movement + arrow aliases
        (key(KeyCode::Char('h')), UiEvent::NavigateLeft),
        (key(KeyCode::Left), UiEvent::NavigateLeft),
        (key(KeyCode::Char('j')), UiEvent::NavigateDown),
        (key(KeyCode::Down), UiEvent::NavigateDown),
        (key(KeyCode::Char('k')), UiEvent::NavigateUp),
        (key(KeyCode::Up), UiEvent::NavigateUp),
        (key(KeyCode::Char('l')), UiEvent::NavigateRight),
        (key(KeyCode::Right), UiEvent::NavigateRight),
        // Extended movement + aliases
        (key(KeyCode::Char('g')), UiEvent::NavigateToTop),
        (
            key_with(KeyCode::Char('G'), KeyModifiers::SHIFT),
            UiEvent::NavigateToBottom,
        ),
        (
            key_with(KeyCode::Char('u'), KeyModifiers::CONTROL),
            UiEvent::NavigatePageUp,
        ),
        (key(KeyCode::PageUp), UiEvent::NavigatePageUp),
        (
            key_with(KeyCode::Char('d'), KeyModifiers::CONTROL),
            UiEvent::NavigatePageDown,
        ),
        (key(KeyCode::PageDown), UiEvent::NavigatePageDown),
        // Per-chunk toggles
        (key(KeyCode::Tab), UiEvent::ToggleChunkContext),
        (key(KeyCode::Char('i')), UiEvent::ToggleNoteExpansion),
        // Review actions
        (key(KeyCode::Enter), UiEvent::SelectCurrent),
        (key(KeyCode::Char('a')), UiEvent::ToggleApprove),
        (key(KeyCode::Char('n')), UiEvent::EnterInstructionMode),
    ];

    for (k, expected) in cases {
        assert_eq!(
            nav(k),
            Some(expected.clone()),
            "navigation dispatch changed for {k:?} (expected {expected:?})"
        );
    }
}

#[test]
fn navigation_unbound_keys_map_to_none() {
    let unbound: Vec<KeyEvent> = vec![
        key(KeyCode::Char('z')),
        key(KeyCode::Char('x')),
        key(KeyCode::Char('0')),
        key(KeyCode::F(1)),
        key(KeyCode::Backspace),
        key(KeyCode::Home),
        key(KeyCode::End),
        key_with(KeyCode::Char('x'), KeyModifiers::CONTROL),
    ];

    for k in unbound {
        assert_eq!(nav(k), None, "unbound key {k:?} now maps to an event");
    }
}

#[test]
fn navigation_bindings_are_modifier_sensitive() {
    // Bindings that require NONE must not fire with modifiers held.
    assert_eq!(
        nav(key_with(KeyCode::Char('a'), KeyModifiers::CONTROL)),
        None
    );
    assert_eq!(
        nav(key_with(KeyCode::Char('j'), KeyModifiers::CONTROL)),
        None
    );
    assert_eq!(nav(key_with(KeyCode::Char(' '), KeyModifiers::SHIFT)), None);
    assert_eq!(nav(key_with(KeyCode::Esc, KeyModifiers::CONTROL)), None);
    // Bindings that require SHIFT must not fire without it.
    assert_eq!(nav(key(KeyCode::Char('G'))), None);
    assert_eq!(nav(key(KeyCode::Char('?'))), None);
    // And CONTROL-bindings must not fire bare.
    assert_eq!(nav(key(KeyCode::Char('u'))), None);
    assert_eq!(nav(key(KeyCode::Char('d'))), None);
}

// ============================================================================
// Leader mode (Space held; keyed on (submenu, key.code))
// ============================================================================

#[test]
fn leader_root_menu_bindings_are_stable() {
    assert_eq!(
        leader(key(KeyCode::Char('a')), None),
        Some(UiEvent::EnterLeaderSubmenu('a'))
    );
    assert_eq!(
        leader(key(KeyCode::Char('t')), None),
        Some(UiEvent::EnterLeaderSubmenu('t'))
    );
}

#[test]
fn leader_actions_submenu_bindings_are_stable() {
    assert_eq!(
        leader(key(KeyCode::Char('a')), Some('a')),
        Some(UiEvent::ToggleApprove)
    );
    assert_eq!(
        leader(key(KeyCode::Char('d')), Some('a')),
        Some(UiEvent::ToggleApprove)
    );
    assert_eq!(
        leader(key(KeyCode::Char('f')), Some('a')),
        Some(UiEvent::ApproveFile)
    );
}

#[test]
fn leader_toggles_submenu_bindings_are_stable() {
    assert_eq!(
        leader(key(KeyCode::Char('r')), Some('t')),
        Some(UiEvent::ToggleReasoning)
    );
}

#[test]
fn leader_esc_deactivates_at_every_level() {
    assert_eq!(
        leader(key(KeyCode::Esc), None),
        Some(UiEvent::DeactivateLeader)
    );
    assert_eq!(
        leader(key(KeyCode::Esc), Some('a')),
        Some(UiEvent::DeactivateLeader)
    );
    assert_eq!(
        leader(key(KeyCode::Esc), Some('t')),
        Some(UiEvent::DeactivateLeader)
    );
}

#[test]
fn leader_unknown_keys_silently_deactivate() {
    // Includes navigation keys: leader mode swallows them.
    assert_eq!(
        leader(key(KeyCode::Char('x')), None),
        Some(UiEvent::DeactivateLeader)
    );
    assert_eq!(
        leader(key(KeyCode::Char('j')), None),
        Some(UiEvent::DeactivateLeader)
    );
    assert_eq!(
        leader(key(KeyCode::Char('z')), Some('a')),
        Some(UiEvent::DeactivateLeader)
    );
    // Submenu keys are scoped: 'a' in the toggles submenu is unknown.
    assert_eq!(
        leader(key(KeyCode::Char('a')), Some('t')),
        Some(UiEvent::DeactivateLeader)
    );
    // Root-menu 'f' only exists inside the actions submenu.
    assert_eq!(
        leader(key(KeyCode::Char('f')), None),
        Some(UiEvent::DeactivateLeader)
    );
    assert_eq!(
        leader(key(KeyCode::Enter), None),
        Some(UiEvent::DeactivateLeader)
    );
}

#[test]
fn leader_dispatch_ignores_modifiers_current_behavior() {
    // Characterized quirk: leader matching is on key.code only.
    assert_eq!(
        leader(key_with(KeyCode::Char('a'), KeyModifiers::CONTROL), None),
        Some(UiEvent::EnterLeaderSubmenu('a'))
    );
    assert_eq!(
        leader(key_with(KeyCode::Char('f'), KeyModifiers::ALT), Some('a')),
        Some(UiEvent::ApproveFile)
    );
}

// ============================================================================
// Input mode (Instruction / DecisionInstruction)
// ============================================================================

#[test]
fn input_mode_control_bindings_are_stable() {
    assert_eq!(input(key(KeyCode::Esc)), Some(UiEvent::CancelInput));
    assert_eq!(
        input(key_with(KeyCode::Char('c'), KeyModifiers::CONTROL)),
        Some(UiEvent::CancelInput)
    );
    assert_eq!(input(key(KeyCode::Enter)), Some(UiEvent::SubmitInput));
}

#[test]
fn input_mode_text_entry_is_stable() {
    assert_eq!(
        input(key(KeyCode::Char('x'))),
        Some(UiEvent::InputChar('x'))
    );
    assert_eq!(
        input(key_with(KeyCode::Char('X'), KeyModifiers::SHIFT)),
        Some(UiEvent::InputChar('X'))
    );
    assert_eq!(
        input(key(KeyCode::Char(' '))),
        Some(UiEvent::InputChar(' '))
    );
    assert_eq!(
        input(key(KeyCode::Char('!'))),
        Some(UiEvent::InputChar('!'))
    );
    // 'q' and 'a' are plain text here, not Quit/ToggleApprove.
    assert_eq!(
        input(key(KeyCode::Char('q'))),
        Some(UiEvent::InputChar('q'))
    );
    assert_eq!(
        input(key(KeyCode::Char('a'))),
        Some(UiEvent::InputChar('a'))
    );
}

#[test]
fn input_mode_editing_bindings_are_stable() {
    assert_eq!(input(key(KeyCode::Backspace)), Some(UiEvent::DeleteChar));
    assert_eq!(input(key(KeyCode::Delete)), Some(UiEvent::DeleteForward));
    assert_eq!(input(key(KeyCode::Left)), Some(UiEvent::MoveCursorLeft));
    assert_eq!(input(key(KeyCode::Right)), Some(UiEvent::MoveCursorRight));
    assert_eq!(input(key(KeyCode::Home)), Some(UiEvent::MoveCursorHome));
    assert_eq!(input(key(KeyCode::End)), Some(UiEvent::MoveCursorEnd));
    assert_eq!(
        input(key_with(KeyCode::Left, KeyModifiers::CONTROL)),
        Some(UiEvent::MoveCursorWordLeft)
    );
    assert_eq!(
        input(key_with(KeyCode::Right, KeyModifiers::CONTROL)),
        Some(UiEvent::MoveCursorWordRight)
    );
}

#[test]
fn input_mode_unbound_keys_map_to_none() {
    assert_eq!(input(key(KeyCode::Up)), None);
    assert_eq!(input(key(KeyCode::Down)), None);
    assert_eq!(input(key(KeyCode::Tab)), None);
    assert_eq!(input(key(KeyCode::F(1))), None);
    // Ctrl+letter is not text; only Ctrl-c is bound.
    assert_eq!(
        input(key_with(KeyCode::Char('v'), KeyModifiers::CONTROL)),
        None
    );
}

#[test]
fn instruction_and_decision_instruction_modes_route_identically() {
    let reviewable_id = ReviewableDiffId::new(
        DiffQuery::new(GitRef::Head, GitRef::Unstaged),
        "src/lib.rs".to_string(),
        LineRange {
            start_line: 1,
            end_line: 2,
            start_column: 0,
            end_column: 0,
        },
    );
    let instruction_mode = InputMode::Instruction { reviewable_id };
    let decision_mode = InputMode::DecisionInstruction { decision_number: 1 };

    let probes = vec![
        key(KeyCode::Esc),
        key(KeyCode::Enter),
        key(KeyCode::Char('x')),
        key(KeyCode::Backspace),
        key_with(KeyCode::Left, KeyModifiers::CONTROL),
        key(KeyCode::Tab),
    ];

    for k in probes {
        assert_eq!(
            handle_key_event(k, &instruction_mode, false, None),
            handle_key_event(k, &decision_mode, false, None),
            "Instruction and DecisionInstruction diverged for {k:?}"
        );
    }
}

// ============================================================================
// Mode routing invariants
// ============================================================================

#[test]
fn leader_state_is_ignored_outside_navigation_mode() {
    // Even with leader flags set, input mode routes to text handling.
    let decision_mode = InputMode::DecisionInstruction { decision_number: 1 };
    assert_eq!(
        handle_key_event(key(KeyCode::Char('a')), &decision_mode, true, Some('a')),
        Some(UiEvent::InputChar('a')),
        "leader state must not leak into input mode"
    );
}
