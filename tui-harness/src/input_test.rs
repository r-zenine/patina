//! Generic input test harness.
//!
//! Run compact input sequences through any ELMApp and capture state snapshots.

use crate::{Result, input_parser::parse_input_sequence, traits::ELMApp};

/// Headless test harness for validating input → state transitions.
pub struct InputTestHarness<M: ELMApp> {
    app: M,
}

impl<M: ELMApp> InputTestHarness<M> {
    /// Create a new input test harness wrapping the given app.
    pub fn new(app: M) -> Self {
        Self { app }
    }

    /// Run an input sequence and return a snapshot after each step.
    ///
    /// The first snapshot is the initial state (before any steps).
    /// `<Wait:N>` steps sleep N milliseconds and run `on_tick`, then
    /// snapshot like any other step.
    pub fn run_sequence(&mut self, input: &str) -> Result<Vec<M::Snapshot>> {
        let steps = parse_input_sequence(input)?;
        let mut snapshots = Vec::new();

        snapshots.push(self.app.snapshot());

        for step in steps {
            step.apply(&mut self.app)?;
            snapshots.push(self.app.snapshot());
        }

        Ok(snapshots)
    }

    /// Run an input sequence and return only the final state snapshot.
    pub fn run_sequence_final_state(&mut self, input: &str) -> Result<M::Snapshot> {
        let steps = parse_input_sequence(input)?;

        for step in steps {
            step.apply(&mut self.app)?;
        }

        Ok(self.app.snapshot())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    /// Minimal app that records key and tick counts.
    struct TickCounter {
        keys: u32,
        ticks: u32,
        created_at: Instant,
        elapsed_ms_at_last_tick: u128,
    }

    impl TickCounter {
        fn new() -> Self {
            Self {
                keys: 0,
                ticks: 0,
                created_at: Instant::now(),
                elapsed_ms_at_last_tick: 0,
            }
        }
    }

    #[derive(serde::Serialize)]
    struct TickSnapshot {
        keys: u32,
        ticks: u32,
        elapsed_ms_at_last_tick: u128,
    }

    impl ELMApp for TickCounter {
        type Snapshot = TickSnapshot;
        type Error = std::convert::Infallible;

        fn dispatch_key(
            &mut self,
            _key: crossterm::event::KeyEvent,
        ) -> std::result::Result<(), Self::Error> {
            self.keys += 1;
            Ok(())
        }

        fn draw(&self, _frame: &mut ratatui::Frame) {}

        fn should_quit(&self) -> bool {
            false
        }

        fn snapshot(&self) -> TickSnapshot {
            TickSnapshot {
                keys: self.keys,
                ticks: self.ticks,
                elapsed_ms_at_last_tick: self.elapsed_ms_at_last_tick,
            }
        }

        fn on_tick(&mut self) {
            self.ticks += 1;
            self.elapsed_ms_at_last_tick = self.created_at.elapsed().as_millis();
        }
    }

    #[test]
    fn test_wait_sleeps_and_runs_on_tick() {
        let mut harness = InputTestHarness::new(TickCounter::new());

        let snapshot = harness.run_sequence_final_state("j<Wait:50>k").unwrap();

        assert_eq!(snapshot.keys, 2);
        assert_eq!(snapshot.ticks, 1);
        assert!(
            snapshot.elapsed_ms_at_last_tick >= 50,
            "on_tick fired after only {}ms — Wait did not sleep",
            snapshot.elapsed_ms_at_last_tick
        );
    }

    #[test]
    fn test_wait_produces_snapshot_step() {
        let mut harness = InputTestHarness::new(TickCounter::new());

        // Initial + 'j' + Wait + 'k' = 4 snapshots.
        let snapshots = harness.run_sequence("j<Wait:10>k").unwrap();

        assert_eq!(snapshots.len(), 4);
        assert_eq!(snapshots[2].ticks, 1);
        assert_eq!(snapshots[2].keys, 1);
    }
}
