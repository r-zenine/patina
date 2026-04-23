//! Combined test harness: input + rendering in lockstep.
//!
//! After each key event, captures both the state snapshot and the visual output.

use crate::{
    Result, TuiError, input_parser::parse_input_sequence, render_test::RenderTestHarness,
    traits::ELMApp,
};

/// Combined test result: state snapshot + visual output at one step.
pub struct CombinedTestResult<S> {
    /// State snapshot after this event.
    pub state: S,
    /// Visual output for this state.
    pub visual: String,
}

/// Combined test harness for full integration testing.
pub struct CombinedTestHarness<M: ELMApp> {
    app: M,
    render_harness: RenderTestHarness,
}

impl<M: ELMApp> CombinedTestHarness<M> {
    /// Create a new combined test harness with default render size (80×24).
    pub fn new(app: M) -> Self {
        Self {
            app,
            render_harness: RenderTestHarness::new(),
        }
    }

    /// Create a combined test harness with custom render dimensions.
    pub fn with_render_size(app: M, width: u16, height: u16) -> Self {
        Self {
            app,
            render_harness: RenderTestHarness::with_size(width, height),
        }
    }

    /// Run an input sequence capturing both state and visual output at each step.
    ///
    /// The first result is the initial state (before any events).
    pub fn run_sequence_with_renders(
        &mut self,
        input: &str,
    ) -> Result<Vec<CombinedTestResult<M::Snapshot>>> {
        let events = parse_input_sequence(input)?;
        let mut results = Vec::new();

        results.push(CombinedTestResult {
            state: self.app.snapshot(),
            visual: self.render_harness.render(&self.app)?,
        });

        for event in events {
            self.app
                .dispatch_key(event)
                .map_err(|e| TuiError::App(Box::new(e)))?;

            results.push(CombinedTestResult {
                state: self.app.snapshot(),
                visual: self.render_harness.render(&self.app)?,
            });
        }

        Ok(results)
    }
}
