//! `patina-detect` — deterministic, LLM-free detectors that surface
//! review-worthy symptoms in agent-generated code. Consumes `diffviz-core`
//! and `lspkit`; never depends on `diffviz-review` (see
//! `.plans/plan-patina-detect/context-document.md`).
//!
//! This crate provides the entity model, `SymptomId` content addressing,
//! baseline persistence, symptom-log export, and a suite of deterministic
//! detectors under `detectors`.

pub mod detectors;
pub mod engines;
pub mod entities;
pub mod export;
pub mod persistence;
pub mod tui;

pub use export::export_symptom_log;
