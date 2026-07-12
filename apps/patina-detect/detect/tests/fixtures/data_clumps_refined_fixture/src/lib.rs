//! Fixture for Phase 16 (Detector 8 revision — closed-recursion exclusion
//! via call hierarchy, decision D011).

/// Single entry point into the `visit_*` family below — mirrors
/// `cognitive_complexity::run_cognitive_complexity`'s one call into
/// `score_node` to kick off the visitor. This is the family's *only*
/// external caller.
pub fn run_visitor(root: i32) -> i32 {
    let mut max_depth = 0;
    visit_node(root, 0, &mut max_depth)
}

/// Member 1 of the closed clump (`value: i32, depth: usize, max_depth: &mut
/// usize`): forwards the clump intact to `visit_branch` — the promotion
/// gate's required forwarding occurrence.
fn visit_node(value: i32, depth: usize, max_depth: &mut usize) -> i32 {
    if value > 0 {
        visit_branch(value, depth, max_depth)
    } else {
        visit_leaf(value, depth, max_depth)
    }
}

/// Member 2: calls back into `visit_node`, but with a computed argument
/// (`value - 1`), not a bare identifier — recursion, not a second
/// forwarding occurrence. Only `visit_node` needs to forward for the
/// promotion gate.
fn visit_branch(value: i32, depth: usize, max_depth: &mut usize) -> i32 {
    *max_depth = (*max_depth).max(depth + 1);
    visit_node(value - 1, depth, max_depth)
}

/// Member 3: terminal case, no further calls into the family.
fn visit_leaf(value: i32, depth: usize, max_depth: &mut usize) -> i32 {
    value + depth as i32 + *max_depth as i32
}

/// First of two genuinely distinct external call sites into the
/// `contact_*` chain below.
pub fn caller_one() -> i32 {
    contact_a(1, "a".to_string(), "a@example.com".to_string())
}

/// Second, independent external call site — proves the group is reached
/// from >= 2 distinct places despite every `contact_*` function living in
/// this one file (decision D011: the rejected file/module heuristic would
/// have wrongly excluded this case).
pub fn caller_two() -> i32 {
    contact_a(2, "b".to_string(), "b@example.com".to_string())
}

/// Member 1 of the kept clump (`id: u64, name: String, email: String`):
/// forwards the clump intact to `contact_b`.
fn contact_a(id: u64, name: String, email: String) -> i32 {
    contact_b(id, name, email)
}

/// Member 2: forwards the clump intact to `contact_c`.
fn contact_b(id: u64, name: String, email: String) -> i32 {
    contact_c(id, name, email)
}

/// Member 3: terminal case.
fn contact_c(id: u64, name: String, email: String) -> i32 {
    id as i32 + name.len() as i32 + email.len() as i32
}

/// Single entry point into the `walk_*` family below, which also routes
/// through `walk_guard` — a helper whose signature carries the whole clump
/// plus one extra parameter (mirrors `cognitive_complexity::score_binary`/
/// `score_operand`). A caller whose parameters are a superset of the clump
/// is the traveling family itself, not an independent external call site,
/// so the family must still count as closed.
pub fn run_walker(item: i64) -> i64 {
    let mut budget = 0;
    walk_node(item, 0, &mut budget)
}

/// Member 1 of the superset-helper clump (`item: i64, level: usize,
/// budget: &mut usize`).
fn walk_node(item: i64, level: usize, budget: &mut usize) -> i64 {
    if item > 0 {
        walk_guard(item, level, budget, true)
    } else {
        walk_leaf(item, level, budget)
    }
}

/// The superset-signature helper: same clump plus `strict`. Not a group
/// member (different member set), but absorbed into the cluster by the
/// closed-cluster check.
fn walk_guard(item: i64, level: usize, budget: &mut usize, strict: bool) -> i64 {
    if strict {
        *budget += 1;
    }
    walk_branch(item, level, budget)
}

/// Member 2: forwards the clump intact to `walk_leaf` — the promotion
/// gate's required forwarding occurrence.
fn walk_branch(item: i64, level: usize, budget: &mut usize) -> i64 {
    if item > 5 {
        return walk_leaf(item, level, budget);
    }
    walk_node(item - 1, level, budget)
}

/// Member 3: terminal case.
fn walk_leaf(item: i64, level: usize, budget: &mut usize) -> i64 {
    item + level as i64 + *budget as i64
}

/// Single entry point into the `probe_*` family below; the only other
/// caller is the `#[test]` fn at the bottom of this file — a test driver
/// exercising the family, not an independent production call site, so the
/// family must still count as closed (mirrors
/// `cognitive_complexity::tests::score_of` calling `score_node`).
pub fn run_probe(key: u32) -> u32 {
    let mut trail = Vec::new();
    probe_head(key, "start", &mut trail)
}

/// Member 1 of the test-caller clump (`key: u32, label: &str, trail: &mut
/// Vec<String>`): forwards the clump intact to `probe_mid`.
fn probe_head(key: u32, label: &str, trail: &mut Vec<String>) -> u32 {
    probe_mid(key, label, trail)
}

/// Member 2: forwards the clump intact to `probe_tail`.
fn probe_mid(key: u32, label: &str, trail: &mut Vec<String>) -> u32 {
    trail.push(label.to_string());
    probe_tail(key, label, trail)
}

/// Member 3: terminal case.
fn probe_tail(key: u32, label: &str, trail: &mut Vec<String>) -> u32 {
    key + trail.len() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_head_records_the_label() {
        let mut trail = Vec::new();
        assert_eq!(probe_head(1, "t", &mut trail), 2);
    }
}
