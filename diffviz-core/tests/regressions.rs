//! Regression tests for previously fixed bugs
//!
//! This module contains tests that verify previously fixed bugs remain fixed.
//! If any of these tests fail, it indicates a regression was introduced.

mod regressions {
    mod rust_parser_visibility_modifier_classification {
        include!("regressions/regression_rust_parser_visibility_modifier_classification.rs");
    }
}
