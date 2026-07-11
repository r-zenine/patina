/// Name of the gitignore-syntax file, checked at every directory level
/// alongside `.gitignore`, that lets a project exclude files from
/// patina-detect's scans without excluding them from git.
pub const IGNORE_FILE_NAME: &str = ".patina_detect_ignore";

pub mod cognitive_complexity;
pub mod data_clumps;
pub mod dead_exports;
pub mod house_rules;
pub mod middleman_delegation;
pub mod near_duplicate_structs;
pub mod parallel_dispatch;
pub mod single_impl_traits;
pub mod type2_clones;
