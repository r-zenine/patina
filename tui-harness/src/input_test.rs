//! Generic input test harness.
//!
//! Run compact input sequences through any ELMApp and capture state snapshots.

use crate::{Result, TuiError, input_parser::parse_input_sequence, traits::ELMApp};

/// Headless test harness for validating input → state transitions.
pub struct InputTestHarness<M: ELMApp> {
    app: M,
}

impl<M: ELMApp> InputTestHarness<M> {
    /// Create a new input test harness wrapping the given app.
    pub fn new(app: M) -> Self {
        Self { app }
    }

    /// Run an input sequence and return a snapshot after each event.
    ///
    /// The first snapshot is the initial state (before any events).
    pub fn run_sequence(&mut self, input: &str) -> Result<Vec<M::Snapshot>> {
        let events = parse_input_sequence(input)?;
        let mut snapshots = Vec::new();

        snapshots.push(self.app.snapshot());

        for event in events {
            self.app
                .dispatch_key(event)
                .map_err(|e| TuiError::App(Box::new(e)))?;
            snapshots.push(self.app.snapshot());
        }

        Ok(snapshots)
    }

    /// Run an input sequence and return only the final state snapshot.
    pub fn run_sequence_final_state(&mut self, input: &str) -> Result<M::Snapshot> {
        let events = parse_input_sequence(input)?;

        for event in events {
            self.app
                .dispatch_key(event)
                .map_err(|e| TuiError::App(Box::new(e)))?;
        }

        Ok(self.app.snapshot())
    }
}
