//! Line diff engine backed by the `similar` crate, with a semantic-anchor
//! alignment post-pass.
//!
//! `diff_lines` computes the edit script between two line sequences using the
//! Patience algorithm (anchors on unique lines, producing human-readable diffs
//! and eliminating the need to special-case duplicate-line inputs the way the
//! hand-rolled Myers implementation had to). `align_by_anchors` then converts
//! adjacent Delete/Add runs whose lines share a semantic anchor (same function
//! signature, struct, etc.) into `Modify` ops — this replaces the old approach
//! of biasing the diff *pathfinding* itself with anchor equality, which made
//! the resulting edit script's correctness hard to verify independently.

use super::SemanticAnchor;
use similar::{Algorithm, capture_diff_slices};

/// A single diff operation, carrying indices into the original line slices
/// rather than owned/copied line content.
#[derive(Debug, Clone, PartialEq)]
pub enum DiffOp {
    /// Line `old_idx` in the old sequence equals line `new_idx` in the new sequence.
    Keep { old_idx: usize, new_idx: usize },
    /// Line `new_idx` in the new sequence was added.
    Add { new_idx: usize },
    /// Line `old_idx` in the old sequence was deleted.
    Delete { old_idx: usize },
    /// Line `old_idx` was replaced by line `new_idx` (semantically related, textually different).
    Modify { old_idx: usize, new_idx: usize },
}

/// Diff two line sequences with the Patience algorithm, expanding `similar`'s
/// range-based ops into one [`DiffOp`] per line.
pub fn diff_lines(old: &[&str], new: &[&str]) -> Vec<DiffOp> {
    capture_diff_slices(Algorithm::Patience, old, new)
        .into_iter()
        .flat_map(expand)
        .collect()
}

fn expand(op: similar::DiffOp) -> Vec<DiffOp> {
    match op {
        similar::DiffOp::Equal {
            old_index,
            new_index,
            len,
        } => (0..len)
            .map(|i| DiffOp::Keep {
                old_idx: old_index + i,
                new_idx: new_index + i,
            })
            .collect(),
        similar::DiffOp::Delete {
            old_index, old_len, ..
        } => (0..old_len)
            .map(|i| DiffOp::Delete {
                old_idx: old_index + i,
            })
            .collect(),
        similar::DiffOp::Insert {
            new_index, new_len, ..
        } => (0..new_len)
            .map(|i| DiffOp::Add {
                new_idx: new_index + i,
            })
            .collect(),
        similar::DiffOp::Replace {
            old_index,
            old_len,
            new_index,
            new_len,
        } => {
            let mut ops = Vec::with_capacity(old_len + new_len);
            ops.extend((0..old_len).map(|i| DiffOp::Delete {
                old_idx: old_index + i,
            }));
            ops.extend((0..new_len).map(|i| DiffOp::Add {
                new_idx: new_index + i,
            }));
            ops
        }
    }
}

/// Two lines are semantically related if they carry the same anchor type and identifier.
fn semantically_related(
    old_anchor: &Option<SemanticAnchor>,
    new_anchor: &Option<SemanticAnchor>,
) -> bool {
    match (old_anchor, new_anchor) {
        (Some(old), Some(new)) => {
            old.anchor_type == new.anchor_type && old.identifier == new.identifier
        }
        _ => false,
    }
}

/// Post-pass: within each adjacent Delete-run/Add-run pair, convert
/// positionally-matching lines whose anchors are semantically related into
/// `Modify` ops. Unmatched pairs stay as an adjacent Delete followed by Add;
/// runs of unequal length leave the excess as plain Delete/Add.
pub fn align_by_anchors(
    ops: Vec<DiffOp>,
    old_anchors: &[Option<SemanticAnchor>],
    new_anchors: &[Option<SemanticAnchor>],
) -> Vec<DiffOp> {
    let mut result = Vec::with_capacity(ops.len());
    let mut i = 0;
    while i < ops.len() {
        if !matches!(ops[i], DiffOp::Delete { .. }) {
            result.push(ops[i].clone());
            i += 1;
            continue;
        }

        let delete_start = i;
        let mut mid = i;
        while mid < ops.len() && matches!(ops[mid], DiffOp::Delete { .. }) {
            mid += 1;
        }
        let mut add_end = mid;
        while add_end < ops.len() && matches!(ops[add_end], DiffOp::Add { .. }) {
            add_end += 1;
        }

        let delete_run = &ops[delete_start..mid];
        let add_run = &ops[mid..add_end];
        let paired = delete_run.len().min(add_run.len());

        for k in 0..paired {
            let old_idx = match delete_run[k] {
                DiffOp::Delete { old_idx } => old_idx,
                _ => unreachable!("delete_run only contains Delete ops"),
            };
            let new_idx = match add_run[k] {
                DiffOp::Add { new_idx } => new_idx,
                _ => unreachable!("add_run only contains Add ops"),
            };

            if semantically_related(&old_anchors[old_idx], &new_anchors[new_idx]) {
                result.push(DiffOp::Modify { old_idx, new_idx });
            } else {
                result.push(DiffOp::Delete { old_idx });
                result.push(DiffOp::Add { new_idx });
            }
        }
        result.extend(delete_run[paired..].iter().cloned());
        result.extend(add_run[paired..].iter().cloned());

        i = add_end;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderable_diff::SemanticAnchorType;

    fn anchor(identifier: &str) -> Option<SemanticAnchor> {
        Some(SemanticAnchor {
            anchor_type: SemanticAnchorType::FunctionSignature,
            identifier: identifier.to_string(),
        })
    }

    #[test]
    fn diff_lines_reconstructs_via_indices() {
        let old = vec!["a", "b", "c"];
        let new = vec!["a", "x", "c"];
        let ops = diff_lines(&old, &new);

        let reconstructed_old: Vec<&str> = ops
            .iter()
            .filter_map(|op| match op {
                DiffOp::Keep { old_idx, .. } | DiffOp::Delete { old_idx } => Some(old[*old_idx]),
                DiffOp::Modify { old_idx, .. } => Some(old[*old_idx]),
                DiffOp::Add { .. } => None,
            })
            .collect();
        assert_eq!(reconstructed_old, old);

        let reconstructed_new: Vec<&str> = ops
            .iter()
            .filter_map(|op| match op {
                DiffOp::Keep { new_idx, .. } | DiffOp::Add { new_idx } => Some(new[*new_idx]),
                DiffOp::Modify { new_idx, .. } => Some(new[*new_idx]),
                DiffOp::Delete { .. } => None,
            })
            .collect();
        assert_eq!(reconstructed_new, new);
    }

    #[test]
    fn diff_lines_handles_duplicate_lines_without_dropping() {
        // The exact shape that broke the hand-rolled Myers snake loop: one old
        // line equal to several consecutive new lines.
        let old = vec!["fn f() {", "    x();", "}"];
        let new = vec!["fn f() {", "    x();", "    x();", "    x();", "}"];
        let ops = diff_lines(&old, &new);

        let kept_and_added_new: Vec<&str> = ops
            .iter()
            .filter_map(|op| match op {
                DiffOp::Keep { new_idx, .. } | DiffOp::Add { new_idx } => Some(new[*new_idx]),
                _ => None,
            })
            .collect();
        assert_eq!(kept_and_added_new, new);
    }

    #[test]
    fn align_by_anchors_converts_matching_pair_to_modify() {
        let ops = vec![DiffOp::Delete { old_idx: 0 }, DiffOp::Add { new_idx: 0 }];
        let old_anchors = vec![anchor("f")];
        let new_anchors = vec![anchor("f")];

        let aligned = align_by_anchors(ops, &old_anchors, &new_anchors);
        assert_eq!(
            aligned,
            vec![DiffOp::Modify {
                old_idx: 0,
                new_idx: 0
            }]
        );
    }

    #[test]
    fn align_by_anchors_leaves_mismatched_anchors_untouched() {
        let ops = vec![DiffOp::Delete { old_idx: 0 }, DiffOp::Add { new_idx: 0 }];
        let old_anchors = vec![anchor("f")];
        let new_anchors = vec![anchor("g")];

        let aligned = align_by_anchors(ops, &old_anchors, &new_anchors);
        assert_eq!(
            aligned,
            vec![DiffOp::Delete { old_idx: 0 }, DiffOp::Add { new_idx: 0 }]
        );
    }

    #[test]
    fn align_by_anchors_leaves_no_anchor_pair_untouched() {
        let ops = vec![DiffOp::Delete { old_idx: 0 }, DiffOp::Add { new_idx: 0 }];
        let old_anchors = vec![None];
        let new_anchors = vec![None];

        let aligned = align_by_anchors(ops, &old_anchors, &new_anchors);
        assert_eq!(
            aligned,
            vec![DiffOp::Delete { old_idx: 0 }, DiffOp::Add { new_idx: 0 }]
        );
    }
}
