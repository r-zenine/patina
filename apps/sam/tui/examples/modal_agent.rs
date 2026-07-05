//! Agent CLI entry for sam's modal view (fixture-driven, never ships).
//!
//! Usage:
//!   cargo run -p sam-tui --example modal_agent -- --describe
//!   cargo run -p sam-tui --example modal_agent -- --test-input "jj"
//!   cargo run -p sam-tui --example modal_agent -- --agent-repl

use sam_tui::modal_view::{HeadlessModalView, MockValue, OptionToggle};
use tui_harness::run_agent_cli;

fn main() {
    let items = vec![
        MockValue::new(1, "alpha"),
        MockValue::new(2, "beta"),
        MockValue::new(3, "gamma"),
        MockValue::new(4, "delta"),
        MockValue::new(5, "epsilon"),
    ];
    let options = vec![OptionToggle {
        text: "verbose".into(),
        key: 'v',
        active: false,
    }];

    let app = HeadlessModalView::new(items, options, true);
    run_agent_cli(app, std::env::args().skip(1)).expect("agent CLI failed");
}
