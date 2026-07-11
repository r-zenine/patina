/// Entry point: the sole caller of `facade`.
pub fn entry_point() -> i32 {
    facade()
}

/// Middleman 1: body is a single delegating call, exactly one caller
/// (`entry_point`) — composes with `inner_helper` into a 2-link chain.
fn facade() -> i32 {
    inner_helper()
}

/// Middleman 2: body is a single delegating call, exactly one caller
/// (`facade`) — the chain's second link.
fn inner_helper() -> i32 {
    core_logic()
}

/// Terminal: does real work, not a single delegating call — the chain ends
/// here.
fn core_logic() -> i32 {
    42
}

/// Two independent callers of `shared_helper` below — proves the
/// exactly-one-caller gate holds even though `shared_helper`'s body has the
/// same single-delegating-call shape as `facade`/`inner_helper`.
pub fn caller_one() -> i32 {
    shared_helper()
}

pub fn caller_two() -> i32 {
    shared_helper()
}

/// Same body shape as a middleman, but called from two places above — must
/// not be reported.
fn shared_helper() -> i32 {
    core_logic()
}

pub trait Greeter {
    fn greet(&self) -> String;
}

pub struct Thing;

impl Thing {
    /// Deliberately not a single delegating call (2 statements) — only
    /// `Greeter::greet` below is meant to exercise the trait-impl
    /// exclusion; this must stay outside that shape so it isn't itself
    /// picked up as an unrelated (correctly excluded, but noisy) finding.
    fn raw_greet(&self) -> String {
        let greeting = "hi";
        greeting.to_string()
    }
}

/// Trait-impl method whose body is a single delegating call and which ends
/// up with exactly one caller (`use_greeter` below) — must still be
/// excluded, since it's referenced through the trait and may be satisfying
/// an interface rather than being a pointless wrapper.
impl Greeter for Thing {
    fn greet(&self) -> String {
        self.raw_greet()
    }
}

pub fn use_greeter(t: &Thing) -> String {
    t.greet()
}

/// Sole caller of `summarize` below — same single-caller shape as
/// `entry_point`/`facade`.
pub fn only_caller_of_summarizer(xs: &[i32]) -> Option<i32> {
    summarize(xs)
}

/// Combinator chain, not a real delegating call, despite having exactly one
/// caller and presenting a single top-level `call_expression`
/// (`.min()`) — the whole reason revision 005 exists (a real-repo run found
/// this exact shape, e.g. `determine_line_relevance_with_precedence`, being
/// misreported). Must not be reported.
fn summarize(xs: &[i32]) -> Option<i32> {
    xs.iter().filter(|x| **x > 0).map(|x| *x).min()
}

// ── Phase 5 (plan-patina-detect-fp-fixes) regression cases ──────────────────

/// Does real work (2 statements) so it is never itself a candidate.
pub struct LeaderState {
    active: bool,
}

impl LeaderState {
    pub fn activate(&mut self) {
        let next = true;
        self.active = next;
    }

    pub fn is_active(&self) -> bool {
        !self.active
    }
}

/// Composition facade over a *private* field — the audit's dominant FP
/// (`UiState::activate_leader` → `self.leader.activate()`). Must NOT be
/// reported even though `activate_leader` is a single delegating call with
/// exactly one caller.
pub struct Gadget {
    leader: LeaderState,
}

impl Gadget {
    pub fn new() -> Gadget {
        Gadget {
            leader: LeaderState { active: false },
        }
    }

    pub fn activate_leader(&mut self) {
        self.leader.activate();
    }
}

impl Default for Gadget {
    fn default() -> Self {
        Self::new()
    }
}

/// Sole caller of `Gadget::activate_leader`.
pub fn drive_gadget(g: &mut Gadget) {
    g.activate_leader();
}

/// Same wrapper shape over a `pub` field — the real `UiState::activate_leader`
/// FP sits over a pub field, so the facade exclusion is deliberately
/// visibility-independent and this must NOT be reported either.
pub struct OpenGadget {
    pub leader: LeaderState,
}

impl OpenGadget {
    pub fn activate_leader(&mut self) {
        self.leader.activate();
    }
}

/// Sole caller of `OpenGadget::activate_leader`.
pub fn drive_open_gadget(g: &mut OpenGadget) {
    g.activate_leader();
}

/// Trait-signature adapter — the audit's second FP family
/// (`TriageApp::process_key_event`): the trait impl has a fixed signature
/// and forwards to an inherent method that adapts `self` into field
/// borrows. `Machine::process` must NOT be reported even though its body is
/// a single call with exactly one caller (`dispatch`).
pub trait Dispatcher {
    fn dispatch(&mut self, key: u32) -> u32;
}

pub struct Machine {
    counter: u32,
    label: String,
}

impl Machine {
    pub fn process(&mut self, key: u32) -> u32 {
        process_impl(&mut self.counter, &self.label, key)
    }
}

impl Dispatcher for Machine {
    fn dispatch(&mut self, key: u32) -> u32 {
        self.process(key)
    }
}

/// Does real work; the adapter's target.
fn process_impl(counter: &mut u32, label: &str, key: u32) -> u32 {
    *counter += key;
    *counter + label.len() as u32
}
