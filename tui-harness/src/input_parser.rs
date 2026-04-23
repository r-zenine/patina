//! Parse compact input sequences into keyboard events
//!
//! Supports vim-style compact notation:
//! - Single chars: "jjk" → 3 KeyEvents
//! - Special keys: "<Space>", "<Enter>", "<Esc>", arrow keys, etc.
//! - Modifiers: "<C-j>", "<S-?>", "<A-x>"
//! - Delays: "<Wait:100>" (ignored)

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{Result, TuiError};

/// Parse a compact input sequence into KeyEvents.
///
/// Supports:
/// - Single chars: "jjk" → [KeyEvent(j), KeyEvent(j), KeyEvent(k)]
/// - Special keys: "<Space>", "<Enter>", "<Esc>", "<Tab>"
/// - Arrow keys: "<Up>", "<Down>", "<Left>", "<Right>"
/// - Modifiers: "<C-j>" (Ctrl), "<S-?>" (Shift), "<A-x>" (Alt)
/// - Combined: "<C-S-?>" (Ctrl+Shift)
/// - Delays: "<Wait:100>" (ignored)
///
/// Examples:
/// - "jjk" → 3 char events
/// - "<Space>aa" → Space, 'a', 'a'
/// - "<C-j>" → Ctrl+j
pub fn parse_input_sequence(input: &str) -> Result<Vec<KeyEvent>> {
    let mut events = Vec::new();
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
                return Err(TuiError::App(
                    "Unclosed angle bracket in input sequence".into(),
                ));
            }

            if let Some(key_event) = parse_special_key(&seq)? {
                events.push(key_event);
            }
        } else {
            let key_event = KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
            events.push(key_event);
        }
    }

    Ok(events)
}

fn parse_special_key(seq: &str) -> Result<Option<KeyEvent>> {
    if seq.starts_with("Wait:") {
        return Ok(None);
    }

    let parts: Vec<&str> = seq.split('-').collect();

    let key_str = *parts
        .last()
        .ok_or_else(|| TuiError::App("Empty special key sequence".into()))?;

    let modifiers = parse_modifiers(&parts[..parts.len() - 1])?;
    let key_code = parse_key_code(key_str)?;

    Ok(Some(KeyEvent::new(key_code, modifiers)))
}

fn parse_modifiers(modifier_strs: &[&str]) -> Result<KeyModifiers> {
    let mut mods = KeyModifiers::NONE;

    for m in modifier_strs {
        match *m {
            "C" => mods |= KeyModifiers::CONTROL,
            "S" => mods |= KeyModifiers::SHIFT,
            "A" => mods |= KeyModifiers::ALT,
            other => {
                return Err(TuiError::App(format!("Unknown modifier: {other}").into()));
            }
        }
    }

    Ok(mods)
}

fn parse_key_code(key_str: &str) -> Result<KeyCode> {
    match key_str {
        "Space" => Ok(KeyCode::Char(' ')),
        "Enter" => Ok(KeyCode::Enter),
        "Tab" => Ok(KeyCode::Tab),
        "Esc" => Ok(KeyCode::Esc),
        "Backspace" => Ok(KeyCode::Backspace),
        "Delete" => Ok(KeyCode::Delete),
        "Insert" => Ok(KeyCode::Insert),
        "Home" => Ok(KeyCode::Home),
        "End" => Ok(KeyCode::End),
        "PageUp" => Ok(KeyCode::PageUp),
        "PageDown" => Ok(KeyCode::PageDown),
        "Up" => Ok(KeyCode::Up),
        "Down" => Ok(KeyCode::Down),
        "Left" => Ok(KeyCode::Left),
        "Right" => Ok(KeyCode::Right),
        "F1" => Ok(KeyCode::F(1)),
        "F2" => Ok(KeyCode::F(2)),
        "F3" => Ok(KeyCode::F(3)),
        "F4" => Ok(KeyCode::F(4)),
        "F5" => Ok(KeyCode::F(5)),
        "F6" => Ok(KeyCode::F(6)),
        "F7" => Ok(KeyCode::F(7)),
        "F8" => Ok(KeyCode::F(8)),
        "F9" => Ok(KeyCode::F(9)),
        "F10" => Ok(KeyCode::F(10)),
        "F11" => Ok(KeyCode::F(11)),
        "F12" => Ok(KeyCode::F(12)),
        s if s.len() == 1 => {
            let ch = s.chars().next().unwrap();
            Ok(KeyCode::Char(ch))
        }
        other => Err(TuiError::App(format!("Unknown key: {other}").into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_chars() {
        let events = parse_input_sequence("jjk").unwrap();
        assert_eq!(events.len(), 3);
    }

    #[test]
    fn test_mixed_chars() {
        let events = parse_input_sequence("abc123").unwrap();
        assert_eq!(events.len(), 6);
    }

    #[test]
    fn test_empty_string() {
        let events = parse_input_sequence("").unwrap();
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_space_key() {
        let events = parse_input_sequence("<Space>").unwrap();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_enter_key() {
        let events = parse_input_sequence("<Enter>").unwrap();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_ctrl_modifier() {
        let events = parse_input_sequence("<C-j>").unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].modifiers, KeyModifiers::CONTROL);
    }

    #[test]
    fn test_shift_modifier() {
        let events = parse_input_sequence("<S-?>").unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].modifiers, KeyModifiers::SHIFT);
    }

    #[test]
    fn test_combined_modifiers() {
        let events = parse_input_sequence("<C-S-?>").unwrap();
        assert_eq!(events.len(), 1);
        assert!(events[0].modifiers.contains(KeyModifiers::CONTROL));
        assert!(events[0].modifiers.contains(KeyModifiers::SHIFT));
    }

    #[test]
    fn test_arrow_keys() {
        let events = parse_input_sequence("<Up><Down><Left><Right>").unwrap();
        assert_eq!(events.len(), 4);
    }

    #[test]
    fn test_space_aa() {
        let events = parse_input_sequence("<Space>aa").unwrap();
        assert_eq!(events.len(), 3);
    }

    #[test]
    fn test_delay_ignored() {
        let events = parse_input_sequence("<Wait:100>jj").unwrap();
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_function_keys() {
        let events = parse_input_sequence("<F1><F12>").unwrap();
        assert_eq!(events.len(), 2);
    }
}
