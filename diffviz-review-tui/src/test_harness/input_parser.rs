//! Parse compact input sequences into keyboard events
//!
//! Supports vim-style compact notation:
//! - Single chars: "jjk" → 3 KeyEvents
//! - Special keys: "<Space>", "<Enter>", "<Esc>", arrow keys, etc.
//! - Modifiers: "<C-j>", "<S-?>", "<A-x>"
//! - Delays: "<Wait:100>"

use crate::Result;
use anyhow::anyhow;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Parse a compact input sequence into KeyEvents
///
/// Supports:
/// - Single chars: "jjk" → [KeyEvent(j), KeyEvent(j), KeyEvent(k)]
/// - Special keys: "<Space>", "<Enter>", "<Esc>", "<Tab>"
/// - Arrow keys: "<Up>", "<Down>", "<Left>", "<Right>"
/// - Modifiers: "<C-j>" (Ctrl), "<S-?>" (Shift), "<A-x>" (Alt)
/// - Combined: "<C-S-?>" (Ctrl+Shift)
/// - Delays: "<Wait:100>" (ignored for now)
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
            // Parse special key sequence
            let mut seq = String::new();
            let mut found_close = false;

            while let Some(c) = chars.next() {
                if c == '>' {
                    found_close = true;
                    break;
                }
                seq.push(c);
            }

            if !found_close {
                return Err(anyhow!("Unclosed angle bracket in input sequence"));
            }

            // Parse the special key sequence
            if let Some(key_event) = parse_special_key(&seq)? {
                events.push(key_event);
            }
            // Skip delay sequences (they return None)
        } else {
            // Regular character
            let key_event = KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
            events.push(key_event);
        }
    }

    Ok(events)
}

/// Parse a special key sequence like "Space", "C-j", "S-?", etc.
/// Returns None for delay sequences like "Wait:100"
fn parse_special_key(seq: &str) -> Result<Option<KeyEvent>> {
    // Handle delay sequences
    if seq.starts_with("Wait:") {
        // Ignore delays in parsing (just skip them)
        return Ok(None);
    }

    // Split by '-' to separate modifiers from key
    let parts: Vec<&str> = seq.split('-').collect();

    // Last part is always the key
    let key_str = *parts
        .last()
        .ok_or_else(|| anyhow!("Empty special key sequence"))?;

    // Everything else is modifiers
    let modifiers = parse_modifiers(&parts[..parts.len() - 1])?;

    // Parse the key code
    let key_code = parse_key_code(key_str)?;

    Ok(Some(KeyEvent::new(key_code, modifiers)))
}

/// Parse modifier prefixes (C, S, A)
fn parse_modifiers(modifier_strs: &[&str]) -> Result<KeyModifiers> {
    let mut mods = KeyModifiers::NONE;

    for m in modifier_strs {
        match *m {
            "C" => mods |= KeyModifiers::CONTROL,
            "S" => mods |= KeyModifiers::SHIFT,
            "A" => mods |= KeyModifiers::ALT,
            other => {
                return Err(anyhow!("Unknown modifier: {}", other));
            }
        }
    }

    Ok(mods)
}

/// Parse key code names
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
        // Function keys F1-F12
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
        // Single character keys
        s if s.len() == 1 => {
            let ch = s.chars().next().unwrap();
            Ok(KeyCode::Char(ch))
        }
        other => Err(anyhow!("Unknown key: {}", other)),
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
        // Delay should be ignored, only 'j' chars remain
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_function_keys() {
        let events = parse_input_sequence("<F1><F12>").unwrap();
        assert_eq!(events.len(), 2);
    }
}
