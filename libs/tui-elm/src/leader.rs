//! Leader-key (Spacemacs-style) state machine with a display timeout.

use std::time::{Duration, Instant};

use crate::registry::BindingScope;

/// Tracks whether the leader menu is open, which submenu is active, and how
/// long until it times out.
///
/// Timeouts do not fire by themselves: call [`LeaderState::is_timed_out`]
/// from the app's `on_tick` and deactivate — time stays an event, not a
/// side effect inside the state.
#[derive(Debug, Clone)]
pub struct LeaderState {
    active: bool,
    pressed_at: Option<Instant>,
    submenu: Option<char>,
    timeout: Duration,
}

impl LeaderState {
    /// A new, inactive leader with the given display timeout.
    pub fn new(timeout: Duration) -> Self {
        Self {
            active: false,
            pressed_at: None,
            submenu: None,
            timeout,
        }
    }

    /// Whether the leader menu is open.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// The open submenu, if any.
    pub fn submenu(&self) -> Option<char> {
        self.submenu
    }

    /// The active leader binding scope; `None` while inactive.
    pub fn scope(&self) -> Option<BindingScope> {
        match (self.active, self.submenu) {
            (false, _) => None,
            (true, None) => Some(BindingScope::LeaderRoot),
            (true, Some(c)) => Some(BindingScope::LeaderSubmenu(c)),
        }
    }

    /// Open the leader menu at its root and start the timeout.
    pub fn activate(&mut self) {
        self.active = true;
        self.pressed_at = Some(Instant::now());
        self.submenu = None;
    }

    /// Enter a submenu and restart the timeout.
    pub fn enter_submenu(&mut self, submenu: char) {
        self.submenu = Some(submenu);
        self.pressed_at = Some(Instant::now());
    }

    /// Close the leader menu.
    pub fn deactivate(&mut self) {
        self.active = false;
        self.pressed_at = None;
        self.submenu = None;
    }

    /// Whether the timeout has elapsed since the last (sub)menu opening.
    pub fn is_timed_out(&self) -> bool {
        match self.pressed_at {
            Some(pressed_at) => pressed_at.elapsed() > self.timeout,
            None => false,
        }
    }

    /// Remaining timeout for display; `None` when inactive or elapsed.
    pub fn timeout_remaining(&self) -> Option<Duration> {
        let elapsed = self.pressed_at?.elapsed();
        (elapsed < self.timeout).then(|| self.timeout - elapsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_inactive() {
        let leader = LeaderState::new(Duration::from_secs(2));
        assert!(!leader.is_active());
        assert_eq!(leader.scope(), None);
        assert!(!leader.is_timed_out());
        assert_eq!(leader.timeout_remaining(), None);
    }

    #[test]
    fn activate_enter_deactivate_walk_the_scopes() {
        let mut leader = LeaderState::new(Duration::from_secs(2));

        leader.activate();
        assert_eq!(leader.scope(), Some(BindingScope::LeaderRoot));
        assert!(leader.timeout_remaining().is_some());

        leader.enter_submenu('a');
        assert_eq!(leader.scope(), Some(BindingScope::LeaderSubmenu('a')));

        leader.deactivate();
        assert_eq!(leader.scope(), None);
        assert_eq!(leader.submenu(), None);
    }

    #[test]
    fn zero_timeout_times_out_immediately() {
        let mut leader = LeaderState::new(Duration::ZERO);
        leader.activate();
        std::thread::sleep(Duration::from_millis(1));
        assert!(leader.is_timed_out());
        assert_eq!(leader.timeout_remaining(), None);
    }
}
