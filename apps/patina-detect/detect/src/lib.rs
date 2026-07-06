//! `patina-detect` — deterministic, LLM-free detectors that surface
//! review-worthy symptoms in agent-generated code. Consumes `diffviz-core`
//! and `lspkit`; never depends on `diffviz-review` (see
//! `.plans/plan-patina-detect/context-document.md`).
//!
//! This crate currently provides the entity model, `SymptomId` content
//! addressing, baseline persistence, and symptom-log export. No detector
//! ships yet — that starts with house-rule violations (ast-grep).

pub mod engines;
pub mod entities;
pub mod export;
pub mod persistence;

pub use export::export_symptom_log;
