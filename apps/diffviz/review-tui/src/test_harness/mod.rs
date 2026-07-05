//! Test harness for diffviz-review-tui
//!
//! Enables automated testing of TUI behavior through:
//! 1. **Input Test** - Validate logic without rendering
//! 2. **Render Test** - Validate rendering
//! 3. **Combined Test** - Full integration
//!
//! Feature-gated with `test-harness` to exclude from production builds.

pub mod combined;
pub mod input_test;
pub mod render_test;
pub mod snapshot;

pub use combined::{CombinedTestHarness, CombinedTestResult};
pub use input_test::InputTestHarness;
pub use render_test::RenderTestHarness;
pub use snapshot::StateSnapshot;
pub use tui_harness::parse_input_sequence;
