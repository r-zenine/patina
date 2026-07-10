/// Genuinely one production implementor, no test double anywhere — the
/// detector's target positive.
pub trait Greeter {
    fn greet(&self) -> String;
}

pub struct EnglishGreeter;

impl Greeter for EnglishGreeter {
    fn greet(&self) -> String {
        "hello".to_string()
    }
}

/// One production implementor plus a test-double implementor declared in
/// `#[cfg(test)]` code below — the Environment/DI pattern. Must be excluded
/// even though its production-impl count alone would look identical to
/// `Greeter`'s.
pub trait Clock {
    fn now(&self) -> u64;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> u64 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeClock;

    impl Clock for FakeClock {
        fn now(&self) -> u64 {
            42
        }
    }

    #[test]
    fn fake_clock_returns_a_fixed_time() {
        assert_eq!(FakeClock.now(), 42);
    }
}
