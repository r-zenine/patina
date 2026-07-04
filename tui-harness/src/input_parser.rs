//! Parse compact input sequences into input steps
//!
//! Supports vim-style compact notation:
//! - Single chars: "jjk" → 3 key steps
//! - Special keys: "<Space>", "<Enter>", "<Esc>", arrow keys, etc.
//! - Modifiers: "<C-j>", "<S-?>", "<A-x>"
//! - Delays: "<Wait:100>" → sleep 100ms, then run `on_tick`

use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{Result, TuiError, traits::ELMApp};

/// A single step of a parsed input sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputStep {
    /// Dispatch a key event to the app.
    Key(KeyEvent),
    /// Advance time: sleep for the duration, then run `on_tick` so
    /// time-based logic (e.g. leader-key timeouts) actually fires.
    Wait(Duration),
}

impl InputStep {
    /// Apply this step to an app.
    pub fn apply<M: ELMApp>(self, app: &mut M) -> Result<()> {
        match self {
            InputStep::Key(event) => app
                .dispatch_key(event)
                .map_err(|e| TuiError::App(Box::new(e))),
            InputStep::Wait(duration) => {
                std::thread::sleep(duration);
                app.on_tick();
                Ok(())
            }
        }
    }
}

/// Parse a compact input sequence into input steps.
///
/// Supports:
/// - Single chars: "jjk" → [Key(j), Key(j), Key(k)]
/// - Special keys: "<Space>", "<Enter>", "<Esc>", "<Tab>"
/// - Arrow keys: "<Up>", "<Down>", "<Left>", "<Right>"
/// - Modifiers: "<C-j>" (Ctrl), "<S-?>" (Shift), "<A-x>" (Alt)
/// - Combined: "<C-S-?>" (Ctrl+Shift)
/// - Delays: "<Wait:100>" → Wait(100ms)
///
/// Examples:
/// - "jjk" → 3 key steps
/// - "<Space>aa" → Space, 'a', 'a'
/// - "<C-j>" → Ctrl+j
pub fn parse_input_sequence(input: &str) -> Result<Vec<InputStep>> {
    let mut steps = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '<' {
            let mut seq = String::new();
            let mut found_close = false;

            for c in chars.by_ref() {
                if c == '>' {
                    found_close = true;
                    break;
                }
                seq.push(c);
            }

            if !found_close {
                return Err(TuiError::Parse(
                    "Unclosed angle bracket in input sequence".to_string(),
                ));
            }

            steps.push(parse_special_key(&seq)?);
        } else {
            let key_event = KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
            steps.push(InputStep::Key(key_event));
        }
    }

    Ok(steps)
}

fn parse_special_key(seq: &str) -> Result<InputStep> {
    if let Some(ms) = seq.strip_prefix("Wait:") {
        let ms: u64 = ms
            .parse()
            .map_err(|_| TuiError::Parse(format!("Invalid wait duration: {ms}")))?;
        return Ok(InputStep::Wait(Duration::from_millis(ms)));
    }

    let (modifier_part, key_str) = split_modifiers_and_key(seq)?;

    let modifiers = parse_modifiers(modifier_part)?;
    let key_code = parse_key_code(key_str)?;

    Ok(InputStep::Key(KeyEvent::new(key_code, modifiers)))
}

/// Split "C-S-?" into ("C-S", "?"). A trailing '-' is the key itself,
/// so "C--" (Ctrl+minus) splits into ("C", "-").
fn split_modifiers_and_key(seq: &str) -> Result<(&str, &str)> {
    if seq.is_empty() {
        return Err(TuiError::Parse("Empty special key sequence".to_string()));
    }
    if seq == "-" {
        return Ok(("", "-"));
    }
    if let Some(mods) = seq.strip_suffix("--") {
        return Ok((mods, "-"));
    }
    match seq.rfind('-') {
        Some(idx) => Ok((&seq[..idx], &seq[idx + 1..])),
        None => Ok(("", seq)),
    }
}

fn parse_modifiers(modifier_part: &str) -> Result<KeyModifiers> {
    let mut mods = KeyModifiers::NONE;

    for m in modifier_part.split('-').filter(|s| !s.is_empty()) {
        match m {
            "C" => mods |= KeyModifiers::CONTROL,
            "S" => mods |= KeyModifiers::SHIFT,
            "A" => mods |= KeyModifiers::ALT,
            other => {
                return Err(TuiError::Parse(format!("Unknown modifier: {other}")));
            }
        }
    }
    debug_assert_eq!(
        MODIFIERS.len(),
        3,
        "keep parse_modifiers arms in sync with the MODIFIERS table"
    );

    Ok(mods)
}

/// Special key names accepted inside angle brackets, and their key codes.
///
/// Single source of truth: the parser looks keys up here, and the
/// `--describe` notation doc lists these same names — they cannot drift.
const SPECIAL_KEYS: &[(&str, KeyCode)] = &[
    ("Space", KeyCode::Char(' ')),
    ("Enter", KeyCode::Enter),
    ("Tab", KeyCode::Tab),
    ("Esc", KeyCode::Esc),
    ("Backspace", KeyCode::Backspace),
    ("Delete", KeyCode::Delete),
    ("Insert", KeyCode::Insert),
    ("Home", KeyCode::Home),
    ("End", KeyCode::End),
    ("PageUp", KeyCode::PageUp),
    ("PageDown", KeyCode::PageDown),
    ("Up", KeyCode::Up),
    ("Down", KeyCode::Down),
    ("Left", KeyCode::Left),
    ("Right", KeyCode::Right),
    ("F1", KeyCode::F(1)),
    ("F2", KeyCode::F(2)),
    ("F3", KeyCode::F(3)),
    ("F4", KeyCode::F(4)),
    ("F5", KeyCode::F(5)),
    ("F6", KeyCode::F(6)),
    ("F7", KeyCode::F(7)),
    ("F8", KeyCode::F(8)),
    ("F9", KeyCode::F(9)),
    ("F10", KeyCode::F(10)),
    ("F11", KeyCode::F(11)),
    ("F12", KeyCode::F(12)),
];

/// Modifier prefixes accepted inside angle brackets, with their meanings.
const MODIFIERS: &[(&str, &str)] = &[("C", "Control"), ("S", "Shift"), ("A", "Alt")];

/// Names accepted inside angle brackets, for the `--describe` notation doc.
pub fn special_key_names() -> Vec<String> {
    SPECIAL_KEYS
        .iter()
        .map(|(name, _)| name.to_string())
        .collect()
}

/// Modifier prefixes with meanings (e.g. "C = Control"), for the notation doc.
pub fn modifier_names() -> Vec<String> {
    MODIFIERS
        .iter()
        .map(|(prefix, meaning)| format!("{prefix} = {meaning}"))
        .collect()
}

fn parse_key_code(key_str: &str) -> Result<KeyCode> {
    if let Some((_, code)) = SPECIAL_KEYS.iter().find(|(name, _)| *name == key_str) {
        return Ok(*code);
    }
    match key_str {
        s if s.chars().count() == 1 => {
            let ch = s.chars().next().unwrap();
            Ok(KeyCode::Char(ch))
        }
        other => Err(TuiError::Parse(format!("Unknown key: {other}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expect_key(step: &InputStep) -> KeyEvent {
        match step {
            InputStep::Key(event) => *event,
            InputStep::Wait(d) => panic!("Expected key step, got Wait({d:?})"),
        }
    }

    #[test]
    fn test_single_chars() {
        let steps = parse_input_sequence("jjk").unwrap();
        assert_eq!(steps.len(), 3);
    }

    #[test]
    fn test_mixed_chars() {
        let steps = parse_input_sequence("abc123").unwrap();
        assert_eq!(steps.len(), 6);
    }

    #[test]
    fn test_empty_string() {
        let steps = parse_input_sequence("").unwrap();
        assert_eq!(steps.len(), 0);
    }

    #[test]
    fn test_space_key() {
        let steps = parse_input_sequence("<Space>").unwrap();
        assert_eq!(steps.len(), 1);
    }

    #[test]
    fn test_enter_key() {
        let steps = parse_input_sequence("<Enter>").unwrap();
        assert_eq!(steps.len(), 1);
    }

    #[test]
    fn test_ctrl_modifier() {
        let steps = parse_input_sequence("<C-j>").unwrap();
        assert_eq!(steps.len(), 1);
        assert_eq!(expect_key(&steps[0]).modifiers, KeyModifiers::CONTROL);
    }

    #[test]
    fn test_shift_modifier() {
        let steps = parse_input_sequence("<S-?>").unwrap();
        assert_eq!(steps.len(), 1);
        assert_eq!(expect_key(&steps[0]).modifiers, KeyModifiers::SHIFT);
    }

    #[test]
    fn test_combined_modifiers() {
        let steps = parse_input_sequence("<C-S-?>").unwrap();
        assert_eq!(steps.len(), 1);
        let key = expect_key(&steps[0]);
        assert!(key.modifiers.contains(KeyModifiers::CONTROL));
        assert!(key.modifiers.contains(KeyModifiers::SHIFT));
    }

    #[test]
    fn test_ctrl_minus() {
        let steps = parse_input_sequence("<C-->").unwrap();
        assert_eq!(steps.len(), 1);
        let key = expect_key(&steps[0]);
        assert_eq!(key.code, KeyCode::Char('-'));
        assert_eq!(key.modifiers, KeyModifiers::CONTROL);
    }

    #[test]
    fn test_bare_minus() {
        let steps = parse_input_sequence("<->").unwrap();
        let key = expect_key(&steps[0]);
        assert_eq!(key.code, KeyCode::Char('-'));
        assert_eq!(key.modifiers, KeyModifiers::NONE);
    }

    #[test]
    fn test_arrow_keys() {
        let steps = parse_input_sequence("<Up><Down><Left><Right>").unwrap();
        assert_eq!(steps.len(), 4);
    }

    #[test]
    fn test_space_aa() {
        let steps = parse_input_sequence("<Space>aa").unwrap();
        assert_eq!(steps.len(), 3);
    }

    #[test]
    fn test_wait_parsed_as_step() {
        let steps = parse_input_sequence("<Wait:100>jj").unwrap();
        assert_eq!(steps.len(), 3);
        assert_eq!(steps[0], InputStep::Wait(Duration::from_millis(100)));
    }

    #[test]
    fn test_invalid_wait_duration_rejected() {
        assert!(parse_input_sequence("<Wait:abc>").is_err());
        assert!(parse_input_sequence("<Wait:>").is_err());
    }

    #[test]
    fn test_unknown_key_rejected() {
        assert!(parse_input_sequence("<Bogus>").is_err());
        assert!(parse_input_sequence("<>").is_err());
    }

    #[test]
    fn test_function_keys() {
        let steps = parse_input_sequence("<F1><F12>").unwrap();
        assert_eq!(steps.len(), 2);
    }
}
