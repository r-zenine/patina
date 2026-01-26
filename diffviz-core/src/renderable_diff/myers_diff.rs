//! Myers diff algorithm implementation with semantic enhancements
//!
//! This module provides a clean implementation of the Myers diff algorithm
//! for computing the shortest edit script between two sequences of lines.
//! It includes semantic enhancements to treat lines with the same semantic
//! anchor as having edit distance 0.

use super::SemanticAnchor;

/// A single diff operation
#[derive(Debug, Clone, PartialEq)]
pub enum DiffOp {
    /// Keep a line (no change)
    Keep { line: String },
    /// Add a line
    Add { line: String },
    /// Delete a line
    Delete { line: String },
    /// Modify a line (old line changed to new line)
    Modify { old_line: String, new_line: String },
}

/// Result of Myers diff algorithm
#[derive(Debug, Clone)]
pub struct DiffResult {
    /// The sequence of diff operations
    pub ops: Vec<DiffOp>,
    /// Total number of additions
    #[allow(dead_code)]
    pub additions: usize,
    /// Total number of deletions
    #[allow(dead_code)]
    pub deletions: usize,
}

/// Apply Myers diff algorithm to two sequences of lines with semantic anchors
pub fn myers_diff_semantic(
    old_lines: &[(&str, Option<SemanticAnchor>)],
    new_lines: &[(&str, Option<SemanticAnchor>)],
) -> DiffResult {
    let n = old_lines.len();
    let m = new_lines.len();

    // Handle edge cases
    if n == 0 && m == 0 {
        return DiffResult {
            ops: Vec::new(),
            additions: 0,
            deletions: 0,
        };
    }

    if n == 0 {
        return DiffResult {
            ops: new_lines
                .iter()
                .map(|(line, _)| DiffOp::Add {
                    line: line.to_string(),
                })
                .collect(),
            additions: m,
            deletions: 0,
        };
    }

    if m == 0 {
        return DiffResult {
            ops: old_lines
                .iter()
                .map(|(line, _)| DiffOp::Delete {
                    line: line.to_string(),
                })
                .collect(),
            additions: 0,
            deletions: n,
        };
    }

    // Find the shortest edit script using Myers algorithm with semantic equality
    let trace = shortest_edit_script_semantic(old_lines, new_lines);
    let operations = backtrack_operations_semantic(&trace, old_lines, new_lines);

    // Post-process to merge identical add/delete pairs into keep operations
    let operations = merge_identical_add_delete_pairs(operations);

    // Count additions and deletions
    let mut additions = 0;
    let mut deletions = 0;
    for op in &operations {
        match op {
            DiffOp::Add { .. } => additions += 1,
            DiffOp::Delete { .. } => deletions += 1,
            DiffOp::Modify { .. } => {
                // Modify counts as both an addition and deletion from diff perspective
                additions += 1;
                deletions += 1;
            }
            DiffOp::Keep { .. } => {}
        }
    }

    DiffResult {
        ops: operations,
        additions,
        deletions,
    }
}

/// Apply Myers diff algorithm to two sequences of lines (backward compatibility)
#[allow(dead_code)]
pub fn myers_diff(old_lines: &[&str], new_lines: &[&str]) -> DiffResult {
    let n = old_lines.len();
    let m = new_lines.len();

    // Handle edge cases
    if n == 0 && m == 0 {
        return DiffResult {
            ops: Vec::new(),
            additions: 0,
            deletions: 0,
        };
    }

    if n == 0 {
        return DiffResult {
            ops: new_lines
                .iter()
                .map(|line| DiffOp::Add {
                    line: line.to_string(),
                })
                .collect(),
            additions: m,
            deletions: 0,
        };
    }

    if m == 0 {
        return DiffResult {
            ops: old_lines
                .iter()
                .map(|line| DiffOp::Delete {
                    line: line.to_string(),
                })
                .collect(),
            additions: 0,
            deletions: n,
        };
    }

    // Find the shortest edit script using Myers algorithm
    let trace = shortest_edit_script(old_lines, new_lines);
    let operations = backtrack_operations(&trace, old_lines, new_lines);

    // Count additions and deletions
    let mut additions = 0;
    let mut deletions = 0;
    for op in &operations {
        match op {
            DiffOp::Add { .. } => additions += 1,
            DiffOp::Delete { .. } => deletions += 1,
            DiffOp::Modify { .. } => {
                // Modify counts as both an addition and deletion from diff perspective
                additions += 1;
                deletions += 1;
            }
            DiffOp::Keep { .. } => {}
        }
    }

    DiffResult {
        ops: operations,
        additions,
        deletions,
    }
}

/// Find the shortest edit script using Myers algorithm
#[allow(dead_code)]
fn shortest_edit_script(old_lines: &[&str], new_lines: &[&str]) -> Vec<Vec<i32>> {
    let n = old_lines.len() as i32;
    let m = new_lines.len() as i32;
    let max = n + m;

    let mut v = vec![0i32; (2 * max + 1) as usize];
    let mut trace = Vec::new();

    for d in 0..=max {
        trace.push(v.clone());

        let mut k = -d;
        while k <= d {
            let k_idx = (k + max) as usize;

            let x = if k == -d || (k != d && v[(k - 1 + max) as usize] < v[(k + 1 + max) as usize])
            {
                v[(k + 1 + max) as usize]
            } else {
                v[(k - 1 + max) as usize] + 1
            };

            let mut y = x - k;

            while x < n && y < m && old_lines[x as usize] == new_lines[y as usize] {
                y += 1;
            }
            let x = x + (y - (x - k));

            v[k_idx] = x;

            if x >= n && y >= m {
                return trace;
            }

            k += 2;
        }
    }

    trace
}

/// Check if two lines are semantically related (for alignment preference)
fn semantically_related(
    old_anchor: &Option<SemanticAnchor>,
    new_anchor: &Option<SemanticAnchor>,
) -> bool {
    match (old_anchor, new_anchor) {
        (Some(old), Some(new)) => {
            // Same anchor type and identifier means these lines are semantically related
            old.anchor_type == new.anchor_type && old.identifier == new.identifier
        }
        _ => false,
    }
}

/// Check if two lines should be treated as equal for Myers algorithm
/// This combines exact equality with semantic alignment preference
fn lines_should_align(
    old_line: &str,
    old_anchor: &Option<SemanticAnchor>,
    new_line: &str,
    new_anchor: &Option<SemanticAnchor>,
) -> bool {
    // Exact match always aligns
    if old_line == new_line {
        return true;
    }

    // Semantic relationship creates alignment preference for Myers pathfinding,
    // but we'll still generate proper diff operations based on actual content
    semantically_related(old_anchor, new_anchor)
}

/// Find the shortest edit script using Myers algorithm with semantic equality
fn shortest_edit_script_semantic(
    old_lines: &[(&str, Option<SemanticAnchor>)],
    new_lines: &[(&str, Option<SemanticAnchor>)],
) -> Vec<Vec<i32>> {
    let n = old_lines.len() as i32;
    let m = new_lines.len() as i32;
    let max = n + m;

    let mut v = vec![0i32; (2 * max + 1) as usize];
    let mut trace = Vec::new();

    for d in 0..=max {
        trace.push(v.clone());

        let mut k = -d;
        while k <= d {
            let k_idx = (k + max) as usize;

            let x = if k == -d || (k != d && v[(k - 1 + max) as usize] < v[(k + 1 + max) as usize])
            {
                v[(k + 1 + max) as usize]
            } else {
                v[(k - 1 + max) as usize] + 1
            };

            let mut y = x - k;

            // Greedy extension with semantic alignment
            while x < n && y < m {
                let (old_line, old_anchor) = &old_lines[x as usize];
                let (new_line, new_anchor) = &new_lines[y as usize];
                if lines_should_align(old_line, old_anchor, new_line, new_anchor) {
                    y += 1;
                } else {
                    break;
                }
            }
            let x = x + (y - (x - k));

            v[k_idx] = x;

            if x >= n && y >= m {
                return trace;
            }

            k += 2;
        }
    }

    trace
}

/// Backtrack through the trace to construct the sequence of operations (semantic version)
fn backtrack_operations_semantic(
    trace: &[Vec<i32>],
    old_lines: &[(&str, Option<SemanticAnchor>)],
    new_lines: &[(&str, Option<SemanticAnchor>)],
) -> Vec<DiffOp> {
    let n = old_lines.len() as i32;
    let m = new_lines.len() as i32;
    let max = n + m;

    let mut x = n;
    let mut y = m;
    let mut operations = Vec::new();

    for d in (0..trace.len()).rev() {
        let v = &trace[d];
        let k = x - y;
        let k_idx = (k + max) as usize;

        let prev_k = if k == -(d as i32)
            || (k != d as i32 && k_idx > 0 && k_idx < v.len() - 1 && v[k_idx - 1] < v[k_idx + 1])
        {
            k + 1
        } else {
            k - 1
        };

        let prev_x = if k_idx > 0 && k_idx < v.len() && ((prev_k + max) as usize) < v.len() {
            v[(prev_k + max) as usize]
        } else {
            0
        };
        let prev_y = prev_x - prev_k;

        // Process aligned lines - but only Keep if they're actually identical
        while x > prev_x && y > prev_y {
            let (old_line, old_anchor) = &old_lines[(x - 1) as usize];
            let (new_line, new_anchor) = &new_lines[(y - 1) as usize];

            if lines_should_align(old_line, old_anchor, new_line, new_anchor) {
                if old_line == new_line {
                    // Actually identical - generate Keep operation
                    operations.push(DiffOp::Keep {
                        line: old_lines[(x - 1) as usize].0.to_string(),
                    });
                } else {
                    // Semantically related but different - generate Modify operation
                    operations.push(DiffOp::Modify {
                        old_line: old_lines[(x - 1) as usize].0.to_string(),
                        new_line: new_lines[(y - 1) as usize].0.to_string(),
                    });
                }
                x -= 1;
                y -= 1;
            } else {
                break;
            }
        }

        if d > 0 {
            if x > prev_x {
                operations.push(DiffOp::Delete {
                    line: old_lines[(x - 1) as usize].0.to_string(),
                });
                x -= 1;
            } else if y > prev_y {
                operations.push(DiffOp::Add {
                    line: new_lines[(y - 1) as usize].0.to_string(),
                });
                y -= 1;
            }
        }
    }

    operations.reverse();
    operations
}

/// Backtrack through the trace to construct the sequence of operations
#[allow(dead_code)]
fn backtrack_operations(trace: &[Vec<i32>], old_lines: &[&str], new_lines: &[&str]) -> Vec<DiffOp> {
    let n = old_lines.len() as i32;
    let m = new_lines.len() as i32;
    let max = n + m;

    let mut x = n;
    let mut y = m;
    let mut operations = Vec::new();

    for d in (0..trace.len()).rev() {
        let v = &trace[d];
        let k = x - y;
        let k_idx = (k + max) as usize;

        let prev_k = if k == -(d as i32)
            || (k != d as i32 && k_idx > 0 && k_idx < v.len() - 1 && v[k_idx - 1] < v[k_idx + 1])
        {
            k + 1
        } else {
            k - 1
        };

        let prev_x = if k_idx > 0 && k_idx < v.len() && ((prev_k + max) as usize) < v.len() {
            v[(prev_k + max) as usize]
        } else {
            0
        };
        let prev_y = prev_x - prev_k;

        while x > prev_x && y > prev_y {
            operations.push(DiffOp::Keep {
                line: old_lines[(x - 1) as usize].to_string(),
            });
            x -= 1;
            y -= 1;
        }

        if d > 0 {
            if x > prev_x {
                operations.push(DiffOp::Delete {
                    line: old_lines[(x - 1) as usize].to_string(),
                });
                x -= 1;
            } else if y > prev_y {
                operations.push(DiffOp::Add {
                    line: new_lines[(y - 1) as usize].to_string(),
                });
                y -= 1;
            }
        }
    }

    operations.reverse();
    operations
}

/// Post-process Myers diff operations to merge identical add/delete pairs into keep operations
///
/// This handles cases where Myers algorithm treats moved identical lines as separate delete+add
/// operations instead of recognizing them as unchanged content in a different position.
fn merge_identical_add_delete_pairs(mut operations: Vec<DiffOp>) -> Vec<DiffOp> {
    let mut i = 0;
    while i < operations.len() {
        if i + 1 < operations.len() {
            match (&operations[i], &operations[i + 1]) {
                // Case 1: Delete followed by Add of identical content
                (DiffOp::Delete { line: delete_line }, DiffOp::Add { line: add_line }) => {
                    if delete_line == add_line {
                        // Replace delete+add pair with keep operation
                        operations[i] = DiffOp::Keep {
                            line: delete_line.clone(),
                        };
                        operations.remove(i + 1);
                        // Don't increment i, check the next operation at the same position
                        continue;
                    }
                }
                // Case 2: Add followed by Delete of identical content (less common)
                (DiffOp::Add { line: add_line }, DiffOp::Delete { line: delete_line }) => {
                    if add_line == delete_line {
                        // Replace add+delete pair with keep operation
                        operations[i] = DiffOp::Keep {
                            line: add_line.clone(),
                        };
                        operations.remove(i + 1);
                        // Don't increment i, check the next operation at the same position
                        continue;
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
    operations
}
