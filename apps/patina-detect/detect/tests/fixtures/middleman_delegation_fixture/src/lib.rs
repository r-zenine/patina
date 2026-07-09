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
