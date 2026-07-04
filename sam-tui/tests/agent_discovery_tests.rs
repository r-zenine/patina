//! Phase 5 contract tests: sam's modal view is discoverable through the
//! same generic surface as diffviz-review-tui (plan-tui-harness-agent-discovery).

use sam_tui::modal_view::{HeadlessModalView, MockValue, OptionToggle};
use tui_harness::{build_manifest, describe_output, ELMApp};

fn items() -> Vec<MockValue> {
    vec![
        MockValue::new(1, "alpha"),
        MockValue::new(2, "beta"),
        MockValue::new(3, "gamma"),
    ]
}

fn one_option() -> Vec<OptionToggle> {
    vec![OptionToggle {
        text: "verbose".into(),
        key: 'v',
        active: false,
    }]
}

#[test]
fn describe_carries_the_sam_snapshot_schema() {
    let app = HeadlessModalView::new(items(), one_option(), true);
    let manifest: serde_json::Value =
        serde_json::from_str(&describe_output(&app).unwrap()).unwrap();

    assert_eq!(manifest["manifest_version"], 1);
    assert_eq!(manifest["described"], true);
    assert_eq!(manifest["app"], "sam-tui-modal");

    let properties = manifest["snapshot_schema"]["properties"]
        .as_object()
        .unwrap();
    for field in [
        "current_mode",
        "cursor",
        "filter_query",
        "item_count",
        "marked_count",
    ] {
        assert!(properties.contains_key(field), "schema missing {field}");
    }
}

#[test]
fn describe_lists_both_view_modes() {
    let app = HeadlessModalView::new(items(), one_option(), false);
    let manifest = build_manifest(&app);
    let names: Vec<&str> = manifest.modes.iter().map(|m| m.name.as_str()).collect();
    assert!(names.contains(&"InsertMode"));
    assert!(names.contains(&"OptionsMode"));
}

#[test]
fn affordances_gate_multi_select_bindings() {
    let with_multi = HeadlessModalView::new(items(), vec![], true);
    let without_multi = HeadlessModalView::new(items(), vec![], false);

    let has_mark =
        |app: &HeadlessModalView<MockValue>| app.affordances().iter().any(|a| a.event == "Mark");

    assert!(has_mark(&with_multi), "multi-select advertises Mark");
    assert!(
        !has_mark(&without_multi),
        "Mark must not be advertised when multi-select is off"
    );
}

#[test]
fn affordances_gate_the_options_toggle() {
    let with_options = HeadlessModalView::new(items(), one_option(), false);
    let without_options = HeadlessModalView::new(items(), vec![], false);

    let has_toggle = |app: &HeadlessModalView<MockValue>| {
        app.affordances()
            .iter()
            .any(|a| a.event == "ToggleViewMode")
    };

    assert!(has_toggle(&with_options));
    assert!(
        !has_toggle(&without_options),
        "Esc/ToggleViewMode must not be advertised without options"
    );
}

#[test]
fn affordances_always_include_navigation_and_the_filter_catch_all() {
    let app = HeadlessModalView::new(items(), vec![], false);
    let affordances = app.affordances();

    assert!(affordances.iter().any(|a| a.event == "Down"));
    assert!(affordances.iter().any(|a| a.event == "Up"));
    assert!(
        affordances.iter().any(|a| a.event == "InputChar"),
        "fuzzy-filter typing must be advertised as a catch-all"
    );
}
