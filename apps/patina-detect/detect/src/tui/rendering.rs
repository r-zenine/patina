//! Builds a `diffviz-core` `ReviewableDiff` for each `Site`, audit-mode
//! (`old_source: None`, decision D006 in `.plans/plan-patina-detect/decision-log.yaml`
//! — a single-unit range with no "old" side is a real, tested path in
//! `create_reviewable_diff_from_range`).
//!
//! Built once at startup (mirrors `diffviz-review-tui`'s precomputed
//! `DrillIndex`) rather than per-frame: reading the file and reparsing on
//! every render would make scrolling/paging redo tree-sitter work for
//! content that never changes during a triage session.

use crate::entities::Site;
use diffviz_core::{
    ProgrammingLanguage, ReviewableDiff, SourceCode, create_reviewable_diff_from_range,
    parsers::parser_for_language,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderingError {
    #[error("failed to read {path}: {source}")]
    ReadFile {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("unsupported file type for tree-sitter rendering: {path}")]
    UnsupportedLanguage { path: String },

    #[error("failed to build a reviewable diff for {path}:{start}-{end}: {source}")]
    Build {
        path: String,
        start: usize,
        end: usize,
        #[source]
        source: diffviz_core::DecisionDiffError,
    },

    #[error("{path}:{start}-{end} produced no reviewable unit")]
    NoUnit {
        path: String,
        start: usize,
        end: usize,
    },
}

/// Build a `ReviewableDiff` covering a site's full line-range span (the
/// smallest range that contains every range the detector reported for this
/// site). Only the first resulting unit is kept — `create_reviewable_diff_from_range`
/// can decompose a range into several units, but a `Site` is meant to be one
/// leaf card in the Drill view, not a group of its own.
pub fn render_site(root: &std::path::Path, site: &Site) -> Result<ReviewableDiff, RenderingError> {
    // Detectors disagree on whether `Site::file` is root-joined or
    // root-relative. Prefer the path as given — it resolves for detectors
    // that already root-join — and fall back to joining `root` for the
    // ones that strip it.
    let resolved_file = if site.file.exists() {
        site.file.clone()
    } else {
        root.join(&site.file)
    };
    let path_str = site.file.to_string_lossy().to_string();

    let start = site.line_ranges.iter().map(|r| r.start).min().unwrap_or(1);
    let end = site
        .line_ranges
        .iter()
        .map(|r| r.end)
        .max()
        .unwrap_or(start);

    let language = ProgrammingLanguage::from_file_path(&path_str);
    let parser =
        parser_for_language(language).ok_or_else(|| RenderingError::UnsupportedLanguage {
            path: path_str.clone(),
        })?;

    let content =
        std::fs::read_to_string(&resolved_file).map_err(|source| RenderingError::ReadFile {
            path: path_str.clone(),
            source,
        })?;
    let new_source = SourceCode::new(content);

    let mut diffs = create_reviewable_diff_from_range(
        &path_str,
        start,
        end,
        None,
        &new_source,
        language,
        parser.as_ref(),
    )
    .map_err(|source| RenderingError::Build {
        path: path_str.clone(),
        start,
        end,
        source,
    })?;

    if diffs.is_empty() {
        return Err(RenderingError::NoUnit {
            path: path_str,
            start,
            end,
        });
    }
    Ok(diffs.remove(0))
}

/// Render every site of every symptom, index-aligned with `symptoms` (i.e.
/// `core_diffs[i][j]` is the rendering of `symptoms[i].sites[j]`).
pub fn render_all<'a>(
    root: &std::path::Path,
    symptoms: impl Iterator<Item = &'a crate::entities::Symptom>,
) -> Result<Vec<Vec<ReviewableDiff>>, RenderingError> {
    symptoms
        .map(|symptom| {
            symptom
                .sites
                .iter()
                .map(|site| render_site(root, site))
                .collect()
        })
        .collect()
}
