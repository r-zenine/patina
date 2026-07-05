# Leader Key System Implementation Plan

## Overview
Implement a Space-based leader key system (similar to vim/neovim with which-key) that moves most actions behind a two-stage mnemonic keybinding system while keeping essential navigation keys direct.

## Goals
- Move action keybindings behind Space leader key with mnemonic hierarchy
- Keep most common navigation keys direct (j/k, g/G, Ctrl+u/d, arrows)
- Display which-key style overlay showing available commands
- Implement 2-second timeout with silent cancel
- Simplify status bar to show minimal hints
- Make system intuitive and discoverable

## Key Design Decisions

### Keys That Stay Direct (No Leader Required)
- **Navigation**: j/k, h/l, g/G, Ctrl+u/d, Ctrl+f/b, arrows, PageUp/PageDown
- **Visual Selection**: v (toggle range selection)
- **Essential Controls**: Tab (focus), q (quit), Esc (cancel), ? (help overlay)

**Note**: We're prioritizing discoverability and consistency over preserving old habits. The leader key system makes all actions self-documenting.

### Keys That Move Behind Leader (Space)
**Actions (Space + a):**
- `<Space>aa` - Toggle approve current diff
- `<Space>af` - Approve all in file

**Comments (Space + c):**
- `<Space>cc` - Add comment
- `<Space>ct` - Toggle comments visibility

**Instructions (Space + i):**
- `<Space>ii` - Add instruction
- `<Space>it` - Toggle instructions visibility

**Toggles (Space + t):**
- `<Space>ts` - Toggle semantic highlighting
- `<Space>tc` - Toggle context folding (formerly 'z')

**Edit (Space + e):**
- `<Space>e` - Enter edit mode

## Implementation Phases

### Phase 1: State Management (src/state.rs)

Add leader key tracking fields to `UiState`:

```rust
pub struct UiState {
    // ... existing fields ...

    /// Leader key state tracking
    pub leader_active: bool,
    pub leader_pressed_at: Option<std::time::Instant>,
    pub leader_submenu: Option<char>,
}
```

Add methods to `impl UiState`:

```rust
/// Activate leader key mode
pub fn activate_leader(&mut self) {
    self.leader_active = true;
    self.leader_pressed_at = Some(std::time::Instant::now());
    self.leader_submenu = None;
}

/// Enter a leader submenu (a, c, i, t, etc.)
pub fn enter_leader_submenu(&mut self, submenu: char) {
    self.leader_submenu = Some(submenu);
    // Reset timeout
    self.leader_pressed_at = Some(std::time::Instant::now());
}

/// Deactivate leader key mode
pub fn deactivate_leader(&mut self) {
    self.leader_active = false;
    self.leader_pressed_at = None;
    self.leader_submenu = None;
}

/// Check if leader key has timed out (2 seconds)
pub fn is_leader_timed_out(&self) -> bool {
    if let Some(pressed_at) = self.leader_pressed_at {
        pressed_at.elapsed() > std::time::Duration::from_secs(2)
    } else {
        false
    }
}

/// Get remaining timeout duration for display
pub fn leader_timeout_remaining(&self) -> Option<std::time::Duration> {
    if let Some(pressed_at) = self.leader_pressed_at {
        let timeout = std::time::Duration::from_secs(2);
        let elapsed = pressed_at.elapsed();
        if elapsed < timeout {
            Some(timeout - elapsed)
        } else {
            None
        }
    } else {
        None
    }
}
```

Update `impl Default for UiState`:

```rust
impl Default for UiState {
    fn default() -> Self {
        Self {
            // ... existing fields ...
            leader_active: false,
            leader_pressed_at: None,
            leader_submenu: None,
        }
    }
}
```

### Phase 2: Event Types (src/events/input.rs)

Add new UiEvent variants:

```rust
pub enum UiEvent {
    // ... existing variants ...

    // Leader key system
    ActivateLeader,
    EnterLeaderSubmenu(char),
    DeactivateLeader,

    // Help overlay
    ToggleHelp,
}
```

### Phase 3: Input Handling (src/events/input.rs)

Refactor `handle_key_event` to accept leader state:

```rust
pub fn handle_key_event(
    key: KeyEvent,
    input_mode: &InputMode,
    leader_active: bool,
    leader_submenu: Option<char>,
) -> Option<UiEvent> {
    match input_mode {
        InputMode::Navigation => {
            if leader_active {
                handle_leader_keys(key, leader_submenu)
            } else {
                handle_navigation_keys(key)
            }
        }
        InputMode::Comment { .. } | InputMode::Instruction { .. } | InputMode::Edit { .. } => {
            handle_input_mode_keys(key)
        }
    }
}
```

Refactor `handle_navigation_keys` to remove leader-bound keys:

```rust
fn handle_navigation_keys(key: KeyEvent) -> Option<UiEvent> {
    match key {
        // Application controls
        KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(UiEvent::Quit),
        KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => Some(UiEvent::Quit),

        // Help overlay
        KeyEvent {
            code: KeyCode::Char('?'),
            modifiers: KeyModifiers::SHIFT,
            ..
        } => Some(UiEvent::ToggleHelp),

        // Leader key activation
        KeyEvent {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(UiEvent::ActivateLeader),

        // Navigation - vim-style (KEEP THESE - NO CHANGES)
        KeyEvent {
            code: KeyCode::Char('h'),
            modifiers: KeyModifiers::NONE,
            ..
        }
        | KeyEvent {
            code: KeyCode::Left,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(UiEvent::NavigateLeft),
        // ... rest of navigation keys unchanged ...

        // Visual selection (stays direct)
        KeyEvent {
            code: KeyCode::Char('v'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(UiEvent::ToggleRangeSelection),

        // Focus toggle (stays direct)
        KeyEvent {
            code: KeyCode::Tab,
            modifiers: KeyModifiers::NONE,
            ..
        } => Some(UiEvent::ToggleFocus),

        // REMOVE THESE (move to leader):
        // - 'a' (approve)
        // - 'A' (approve file)
        // - 'c' (comment mode)
        // - 'i' (instruction mode)
        // - 'e' (edit mode)
        // - 's' (semantic toggle)
        // - 'C' (toggle comments view)
        // - 'I' (toggle instructions view)

        _ => None,
    }
}
```

Add new function `handle_leader_keys`:

```rust
fn handle_leader_keys(key: KeyEvent, submenu: Option<char>) -> Option<UiEvent> {
    match (submenu, key.code) {
        // First level - entering submenus
        (None, KeyCode::Char('a')) => Some(UiEvent::EnterLeaderSubmenu('a')),
        (None, KeyCode::Char('c')) => Some(UiEvent::EnterLeaderSubmenu('c')),
        (None, KeyCode::Char('i')) => Some(UiEvent::EnterLeaderSubmenu('i')),
        (None, KeyCode::Char('t')) => Some(UiEvent::EnterLeaderSubmenu('t')),
        (None, KeyCode::Char('e')) => {
            // Edit mode is single-key, not a submenu
            Some(UiEvent::EnterEditMode)
        }

        // Actions submenu (Space + a + ?)
        (Some('a'), KeyCode::Char('a')) => Some(UiEvent::ToggleApprove),
        (Some('a'), KeyCode::Char('f')) => Some(UiEvent::ApproveFile),

        // Comments submenu (Space + c + ?)
        (Some('c'), KeyCode::Char('c')) => Some(UiEvent::EnterCommentMode),
        (Some('c'), KeyCode::Char('t')) => Some(UiEvent::ToggleComments),

        // Instructions submenu (Space + i + ?)
        (Some('i'), KeyCode::Char('i')) => Some(UiEvent::EnterInstructionMode),
        (Some('i'), KeyCode::Char('t')) => Some(UiEvent::ToggleInstructions),

        // Toggles submenu (Space + t + ?)
        (Some('t'), KeyCode::Char('s')) => Some(UiEvent::ToggleSemanticHighlight),
        (Some('t'), KeyCode::Char('c')) => Some(UiEvent::ToggleContextDisplay),

        // Cancel leader mode
        (_, KeyCode::Esc) => Some(UiEvent::DeactivateLeader),

        // Unknown key - deactivate leader silently
        _ => Some(UiEvent::DeactivateLeader),
    }
}
```

### Phase 4: Event Loop (src/app.rs)

Update `handle_events` to check leader timeout and pass leader state:

```rust
fn handle_events(&mut self) -> Result<bool> {
    // Check leader timeout first
    if self.ui_state.leader_active && self.ui_state.is_leader_timed_out() {
        self.ui_state.deactivate_leader();
    }

    // Poll for input events with timeout
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            // Convert keyboard input to UI event, passing leader state
            if let Some(ui_event) = handle_key_event(
                key,
                &self.ui_state.input_mode,
                self.ui_state.leader_active,
                self.ui_state.leader_submenu,
            ) {
                // Handle UI-only events first
                self.handle_ui_event(&ui_event)?;

                // Convert to business events and handle through ReviewEngine
                if let Some(business_event) =
                    ui_event_to_business_event(&ui_event, &self.ui_state)
                {
                    self.handle_business_event(business_event)?;

                    // Exit input mode after successful business operation
                    if matches!(ui_event, UiEvent::SubmitInput) {
                        self.ui_state.exit_input_mode();
                    }
                }
            }
        }
    }

    Ok(self.ui_state.should_quit)
}
```

Update `handle_ui_event` to handle new leader events:

```rust
fn handle_ui_event(&mut self, event: &UiEvent) -> Result<()> {
    // ... existing event handling ...

    match event {
        // ... existing cases ...

        UiEvent::ActivateLeader => {
            self.ui_state.activate_leader();
        }

        UiEvent::EnterLeaderSubmenu(submenu) => {
            self.ui_state.enter_leader_submenu(*submenu);
        }

        UiEvent::DeactivateLeader => {
            self.ui_state.deactivate_leader();
        }

        UiEvent::ToggleHelp => {
            self.ui_state.show_help = !self.ui_state.show_help;
        }

        // ... rest of existing cases ...
    }

    Ok(())
}
```

### Phase 5: Which-Key Overlay Component (NEW FILE: src/ui/components/which_key.rs)

Create new overlay component for displaying leader key menu:

```rust
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::{state::UiState, theme::{Colors, Styles}};

/// Render the which-key overlay when leader is active
pub fn render(f: &mut Frame, ui_state: &UiState) {
    if !ui_state.leader_active {
        return;
    }

    let area = centered_popup(f.size(), 60, 40);

    // Clear the area
    f.render_widget(Clear, area);

    let content = match ui_state.leader_submenu {
        None => create_root_menu(ui_state),
        Some('a') => create_actions_submenu(),
        Some('c') => create_comments_submenu(),
        Some('i') => create_instructions_submenu(),
        Some('t') => create_toggles_submenu(),
        _ => create_root_menu(ui_state),
    };

    let title = match ui_state.leader_submenu {
        None => "Leader Menu (Space)",
        Some('a') => "Actions (Space + a)",
        Some('c') => "Comments (Space + c)",
        Some('i') => "Instructions (Space + i)",
        Some('t') => "Toggles (Space + t)",
        _ => "Leader Menu",
    };

    // Add timeout indicator
    let timeout_text = if let Some(remaining) = ui_state.leader_timeout_remaining() {
        format!(" [{}s]", remaining.as_secs())
    } else {
        String::new()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!("{}{}", title, timeout_text))
        .border_style(Style::default().fg(Colors::CYAN));

    let paragraph = Paragraph::new(content).block(block);
    f.render_widget(paragraph, area);
}

fn create_root_menu(_ui_state: &UiState) -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        create_menu_line("a", "Actions", "approve, approve file"),
        create_menu_line("c", "Comments", "add, toggle view"),
        create_menu_line("i", "Instructions", "add, toggle view"),
        create_menu_line("t", "Toggles", "semantic, context folding"),
        create_menu_line("e", "Edit mode", "edit current diff"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Esc", Style::default().fg(Colors::CYAN)),
            Span::styled(" cancel", Styles::muted()),
        ]),
    ]
}

fn create_actions_submenu() -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        create_menu_line("a", "Approve", "toggle approval for current diff"),
        create_menu_line("f", "Approve File", "approve all diffs in file"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Esc", Style::default().fg(Colors::CYAN)),
            Span::styled(" back", Styles::muted()),
        ]),
    ]
}

fn create_comments_submenu() -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        create_menu_line("c", "Add Comment", "add comment to current diff"),
        create_menu_line("t", "Toggle View", "show/hide comments overlay"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Esc", Style::default().fg(Colors::CYAN)),
            Span::styled(" back", Styles::muted()),
        ]),
    ]
}

fn create_instructions_submenu() -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        create_menu_line("i", "Add Instruction", "add instruction to current diff"),
        create_menu_line("t", "Toggle View", "show/hide instructions overlay"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Esc", Style::default().fg(Colors::CYAN)),
            Span::styled(" back", Styles::muted()),
        ]),
    ]
}

fn create_toggles_submenu() -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        create_menu_line("s", "Semantic", "toggle semantic highlighting"),
        create_menu_line("c", "Context", "toggle context folding"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Esc", Style::default().fg(Colors::CYAN)),
            Span::styled(" back", Styles::muted()),
        ]),
    ]
}

fn create_menu_line(key: &str, name: &str, description: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            key.to_string(),
            Style::default()
                .fg(Colors::CYAN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ", Style::default()),
        Span::styled(
            format!("{:<20}", name),
            Style::default().fg(Colors::WHITE),
        ),
        Span::styled(description.to_string(), Styles::muted()),
    ])
}

fn centered_popup(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
```

Add to `src/ui/components/mod.rs`:
```rust
pub mod which_key;
```

### Phase 6: Help Overlay Component (NEW FILE: src/ui/components/help_overlay.rs)

Create help overlay for showing all keybindings when '?' is pressed:

```rust
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::{state::UiState, theme::{Colors, Styles}};

pub fn render(f: &mut Frame, ui_state: &UiState) {
    if !ui_state.show_help {
        return;
    }

    let area = centered_popup(f.size(), 80, 80);

    f.render_widget(Clear, area);

    let content = create_help_content();

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Keybindings Help (? to close)")
        .border_style(Style::default().fg(Colors::CYAN));

    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn create_help_content() -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        create_section_header("NAVIGATION (Direct Keys)"),
        create_help_line("j/k or ↓/↑", "Move cursor up/down"),
        create_help_line("h/l or ←/→", "Switch panels / navigate files"),
        create_help_line("g/G", "Jump to top/bottom"),
        create_help_line("Ctrl+u/d", "Page up/down"),
        create_help_line("Ctrl+j/k", "Scroll inactive panel"),
        create_help_line("Tab", "Toggle focus between panels"),
        Line::from(""),
        create_section_header("VISUAL (Direct Keys)"),
        create_help_line("v", "Toggle visual selection mode"),
        Line::from(""),
        create_section_header("LEADER KEY SYSTEM (Space + key)"),
        create_help_line("Space", "Activate leader menu (2s timeout)"),
        Line::from(""),
        create_help_line("Space a a", "Toggle approve current diff"),
        create_help_line("Space a f", "Approve all diffs in file"),
        Line::from(""),
        create_help_line("Space c c", "Add comment"),
        create_help_line("Space c t", "Toggle comments overlay"),
        Line::from(""),
        create_help_line("Space i i", "Add instruction"),
        create_help_line("Space i t", "Toggle instructions overlay"),
        Line::from(""),
        create_help_line("Space t s", "Toggle semantic highlighting"),
        create_help_line("Space t c", "Toggle context folding"),
        Line::from(""),
        create_help_line("Space e", "Enter edit mode"),
        Line::from(""),
        create_section_header("ESSENTIAL (Direct Keys)"),
        create_help_line("?", "Toggle this help overlay"),
        create_help_line("q", "Quit application"),
        create_help_line("Esc", "Cancel / close overlay"),
        Line::from(""),
    ]
}

fn create_section_header(text: &str) -> Line<'static> {
    Line::from(vec![Span::styled(
        format!("  {}", text),
        Style::default()
            .fg(Colors::CYAN)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    )])
}

fn create_help_line(keys: &str, description: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled("    ", Style::default()),
        Span::styled(
            format!("{:<20}", keys),
            Style::default().fg(Colors::YELLOW),
        ),
        Span::styled(description.to_string(), Style::default().fg(Colors::WHITE)),
    ])
}

fn centered_popup(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
```

Add to `src/ui/components/mod.rs`:
```rust
pub mod help_overlay;
```

Add `show_help` field to UiState (in src/state.rs):
```rust
pub struct UiState {
    // ... existing fields ...
    pub show_help: bool,
}

// In Default impl:
show_help: false,
```

### Phase 7: Status Bar Simplification (src/ui/components/status_bar.rs)

Replace `create_file_list_keybindings` and `create_diff_view_keybindings`:

```rust
fn create_file_list_keybindings() -> Line<'static> {
    Line::from(vec![
        Span::styled("j/k", Style::default().fg(Colors::CYAN)),
        Span::styled(" nav  ", Styles::muted()),
        Span::styled("Space", Style::default().fg(Colors::CYAN)),
        Span::styled(" leader  ", Styles::muted()),
        Span::styled("?", Style::default().fg(Colors::CYAN)),
        Span::styled(" help  ", Styles::muted()),
        Span::styled("q", Style::default().fg(Colors::CYAN)),
        Span::styled(" quit", Styles::muted()),
    ])
}

fn create_diff_view_keybindings() -> Line<'static> {
    Line::from(vec![
        Span::styled("j/k", Style::default().fg(Colors::CYAN)),
        Span::styled(" nav  ", Styles::muted()),
        Span::styled("v", Style::default().fg(Colors::CYAN)),
        Span::styled(" visual  ", Styles::muted()),
        Span::styled("Space", Style::default().fg(Colors::CYAN)),
        Span::styled(" leader  ", Styles::muted()),
        Span::styled("?", Style::default().fg(Colors::CYAN)),
        Span::styled(" help", Styles::muted()),
    ])
}
```

### Phase 8: UI Integration (src/ui/mod.rs)

Update the main draw function to render overlays:

```rust
pub fn draw(f: &mut Frame, ui_state: &mut UiState, review_engine: &ReviewEngine) {
    // ... existing layout and rendering ...

    // Render overlays (in order - last rendered is on top)
    input_modal::render(f, ui_state);
    components::which_key::render(f, ui_state);
    components::help_overlay::render(f, ui_state);
}
```

## Critical Files to Modify

1. **src/state.rs** - Add leader state tracking fields and methods
2. **src/events/input.rs** - Refactor input handling for two-stage leader key system
3. **src/app.rs** - Add timeout checking and leader state handling
4. **src/ui/components/which_key.rs** - NEW: Create which-key overlay component
5. **src/ui/components/help_overlay.rs** - NEW: Create help overlay component
6. **src/ui/components/status_bar.rs** - Simplify keybinding display
7. **src/ui/components/mod.rs** - Export new components
8. **src/ui/mod.rs** - Render overlays in correct order

## Testing Strategy

### Manual Test Cases

1. **Leader Key Activation**
   - Press Space → verify which-key overlay appears
   - Wait 2 seconds → verify overlay disappears silently
   - Press Space then Esc → verify overlay closes immediately

2. **Two-Stage Commands**
   - Press Space, then 'a', then 'a' → verify approve toggles
   - Press Space, then 'c', then 'c' → verify comment mode activates
   - Press Space, then 'i', then 't' → verify instructions overlay toggles
   - Press Space, then 't', then 's' → verify semantic highlighting toggles
   - Press Space, then 't', then 'c' → verify context folding toggles

3. **Direct Keys Still Work**
   - j/k → verify cursor movement works
   - v → verify visual selection works
   - Tab → verify focus toggle works
   - ? → verify help overlay appears
   - q → verify quit works

4. **Submenu Navigation**
   - Press Space → see root menu
   - Press 'a' → see actions submenu
   - Press Esc → return to root menu
   - Press Esc again → close overlay

5. **Timeout Visual Feedback**
   - Press Space → verify timeout countdown shows in title
   - Watch countdown → verify it decrements

6. **Help Overlay**
   - Press ? → verify help overlay appears with all keybindings
   - Press ? again → verify help overlay closes
   - Verify help shows both direct keys and leader key sequences

## Success Criteria

- ✅ All action keys moved behind Space leader except navigation/visual/essential
- ✅ Which-key overlay shows immediately when Space pressed
- ✅ Submenu navigation works (Space → a → a for approve)
- ✅ 2-second timeout with silent cancel
- ✅ Help overlay (?) shows comprehensive keybinding reference
- ✅ Status bar simplified to show only essential hints
- ✅ No regressions in existing navigation or input modes
- ✅ All original functionality preserved, just accessed differently

## Risks & Mitigation

**Risk**: Users get confused by two-stage input
- *Mitigation*: Which-key overlay provides instant visual feedback with clear labels

**Risk**: Timeout too short/long
- *Mitigation*: 2 seconds is standard, can be made configurable later if needed

**Risk**: Breaking existing workflows
- *Mitigation*: User confirmed they're not worried about this - we're prioritizing discoverability over preserving old habits

**Risk**: Complex refactoring introduces bugs
- *Mitigation*: Incremental implementation - test after each phase

## Notes

- Leader key defaults to Space but architecture supports making it configurable later
- Timeout mechanism uses existing 100ms polling loop - no new timers needed
- Overlay z-ordering: input_modal → which_key → help_overlay (help on top)
- All business logic (ReviewEngine) unchanged - purely UI refactoring
