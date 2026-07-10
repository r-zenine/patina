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
