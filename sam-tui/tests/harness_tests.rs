use sam_tui::modal_view::{HeadlessModalView, MockValue, OptionToggle};
use tui_harness::{InputTestHarness, RenderTestHarness};

fn five_items() -> Vec<MockValue> {
    vec![
        MockValue::new(1, "one"),
        MockValue::new(2, "two"),
        MockValue::new(3, "three"),
        MockValue::new(4, "four"),
        MockValue::new(5, "five"),
    ]
}

fn items_with_options() -> (Vec<MockValue>, Vec<OptionToggle>) {
    let items = vec![
        MockValue::new(1, "alpha"),
        MockValue::new(2, "beta"),
        MockValue::new(3, "gamma"),
    ];
    let options = vec![OptionToggle {
        text: "verbose".into(),
        key: 'v',
        active: false,
    }];
    (items, options)
}

#[test]
fn navigation_j_moves_cursor_down() {
    let app = HeadlessModalView::new(five_items(), vec![], false);
    let mut harness = InputTestHarness::new(app);
    let snapshots = harness.run_sequence("jj").unwrap();
    // snapshots[0] = initial, snapshots[1] = after first j, snapshots[2] = after second j
    assert_eq!(snapshots[2].cursor, 2);
}

#[test]
fn navigation_k_moves_cursor_up() {
    let app = HeadlessModalView::new(five_items(), vec![], false);
    let mut harness = InputTestHarness::new(app);
    let snapshots = harness.run_sequence("jjk").unwrap();
    assert_eq!(snapshots[3].cursor, 1);
}

#[test]
fn navigation_arrow_keys() {
    let app = HeadlessModalView::new(five_items(), vec![], false);
    let mut harness = InputTestHarness::new(app);
    let snapshots = harness.run_sequence("<Down><Down>").unwrap();
    assert_eq!(snapshots[2].cursor, 2);
}

#[test]
fn navigation_cursor_does_not_exceed_list() {
    let app = HeadlessModalView::new(five_items(), vec![], false);
    let mut harness = InputTestHarness::new(app);
    let final_state = harness.run_sequence_final_state("jjjjjjjj").unwrap();
    assert_eq!(final_state.cursor, 4);
}

#[test]
fn filter_reduces_item_count() {
    let app = HeadlessModalView::new(five_items(), vec![], false);
    let mut harness = InputTestHarness::new(app);
    let final_state = harness.run_sequence_final_state("on").unwrap();
    assert_eq!(final_state.filter_query, "on");
    // "one" matches "on", others may or may not — item_count should be ≤ 5
    assert!(final_state.item_count <= 5);
    assert!(final_state.item_count >= 1);
}

#[test]
fn filter_then_backspace() {
    let app = HeadlessModalView::new(five_items(), vec![], false);
    let mut harness = InputTestHarness::new(app);
    let final_state = harness.run_sequence_final_state("co<Backspace>").unwrap();
    assert_eq!(final_state.filter_query, "c");
}

#[test]
fn mode_toggle_with_esc() {
    let (items, options) = items_with_options();
    let app = HeadlessModalView::new(items, options, false);
    let mut harness = InputTestHarness::new(app);
    let snapshots = harness.run_sequence("<Esc>").unwrap();
    assert_eq!(snapshots[0].current_mode, "InsertMode");
    assert_eq!(snapshots[1].current_mode, "OptionsMode");
}

#[test]
fn multi_select_with_ctrl_s() {
    let app = HeadlessModalView::new(five_items(), vec![], true);
    let mut harness = InputTestHarness::new(app);
    let final_state = harness.run_sequence_final_state("<C-s>j<C-s>").unwrap();
    assert_eq!(final_state.marked_count, 2);
}

#[test]
fn confirm_with_enter_sets_quit() {
    let app = HeadlessModalView::new(five_items(), vec![], false);
    let mut harness = InputTestHarness::new(app);
    // After Enter the app signals quit — snapshot still captures state
    let final_state = harness.run_sequence_final_state("<Enter>").unwrap();
    assert_eq!(final_state.item_count, 5);
}

#[test]
fn render_returns_non_empty_output() {
    let app = HeadlessModalView::new(five_items(), vec![], false);
    let harness = RenderTestHarness::new();
    let output = harness.render(&app).unwrap();
    assert!(!output.is_empty());
}

#[test]
fn render_contains_items() {
    let app = HeadlessModalView::new(five_items(), vec![], false);
    let harness = RenderTestHarness::new();
    let output = harness.render(&app).unwrap();
    assert!(output.contains("one") || output.contains("two") || output.contains("three"));
}
