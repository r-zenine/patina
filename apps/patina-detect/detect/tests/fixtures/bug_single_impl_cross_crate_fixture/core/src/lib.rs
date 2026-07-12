//! Mirror of the sam-core/sam-cli `SessionSaver` shape: the trait lives
//! here, its test double is declared *inside a `#[test]` fn body* below,
//! and the sole production impl lives in the sibling `cli` crate that
//! depends on this one.

/// The DI trait: one production impl (`saver_cli::RealSaver`) plus the
/// fn-body-local `MockSaver` test double below — the Environment/DI
/// pattern the detector must exclude.
pub trait Saver {
    fn save(&self) -> u32;
}

/// Indexing-settled anchor — genuinely single-impl, no test double
/// anywhere, always reported.
pub trait Beacon {
    fn shine(&self) -> u32;
}

pub struct SteadyBeacon;

impl Beacon for SteadyBeacon {
    fn shine(&self) -> u32 {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_saver_saves() {
        // The mock and its impl are block-local items inside the #[test]
        // fn, exactly like sam_engine.rs's MockSessionSaver —
        // rust-analyzer's implementations() does not surface impls
        // declared inside function bodies.
        struct MockSaver;

        impl Saver for MockSaver {
            fn save(&self) -> u32 {
                42
            }
        }

        assert_eq!(MockSaver.save(), 42);
    }
}
