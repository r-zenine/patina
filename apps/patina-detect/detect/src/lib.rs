//! `patina-detect` — deterministic, LLM-free detectors that surface
//! review-worthy symptoms in agent-generated code. Consumes `diffviz-core`
//! and `lspkit`; never depends on `diffviz-review` (see
//! `.plans/plan-patina-detect/context-document.md`).
//!
//! This crate provides the entity model, `SymptomId` content addressing,
//! baseline persistence, symptom-log export, and the first real detector:
//! house-rule violations (`detectors::house_rules`), via an embedded
//! ast-grep rule pack.

pub mod detectors;
pub mod engines;
pub mod entities;
pub mod export;
pub mod persistence;

pub use export::export_symptom_log;
