//! Generic file-backed data structures on top of rustbreak: an append/prune
//! sequential log and a key/value store with optional TTL eviction. No
//! dependency on any domain layer.

mod associative_state;
mod sequential_state;

pub use associative_state::{AssociativeStateWithTTL, ErrorAssociativeState};
pub use sequential_state::{ErrorSequentialState, ModResult, SequentialState};
