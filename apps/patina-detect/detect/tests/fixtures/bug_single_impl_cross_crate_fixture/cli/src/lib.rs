use saver_core::Saver;

/// The sole production implementor of `saver_core::Saver` (mirrors
/// sam-cli's `SessionEngine`).
pub struct RealSaver;

impl Saver for RealSaver {
    fn save(&self) -> u32 {
        7
    }
}
