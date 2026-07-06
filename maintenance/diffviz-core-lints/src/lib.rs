#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_hir;
extern crate rustc_span;

use clippy_utils::diagnostics::span_lint;
use rustc_hir::def_id::LOCAL_CRATE;
use rustc_hir::{Expr, ExprKind};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::sym;

dylint_linting::declare_late_lint! {
    /// ### What it does
    /// Flags use of the `regex` crate, or of `str`/`String` pattern-search
    /// methods (`split`, `find`, `matches`, `replace`, ...), inside the
    /// `diffviz-core` crate.
    ///
    /// ### Why is this bad?
    /// `diffviz-core` is the semantic analysis layer and is required to go
    /// through Tree-sitter for all code understanding. String/regex based
    /// parsing defeats that guarantee and silently reintroduces brittle,
    /// language-unaware logic. See CLAUDE.md: "string based or regexp
    /// operations to analyze code are forbidden in this module."
    ///
    /// ### Example
    /// ```rust
    /// let re = regex::Regex::new(r"fn\s+\w+").unwrap();
    /// ```
    /// Use instead: parse the source with Tree-sitter and query the AST.
    pub NO_STRING_PARSING_IN_CORE,
    Deny,
    "string/regex based parsing is forbidden in diffviz-core; use Tree-sitter"
}

const BANNED_STR_METHODS: &[&str] = &[
    "split",
    "splitn",
    "rsplit",
    "rsplitn",
    "split_terminator",
    "rsplit_terminator",
    "find",
    "rfind",
    "matches",
    "rmatches",
    "match_indices",
    "rmatch_indices",
    "replace",
    "replacen",
];

impl<'tcx> LateLintPass<'tcx> for NoStringParsingInCore {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
        if cx.tcx.crate_name(LOCAL_CRATE).as_str() != "diffviz_core" {
            return;
        }

        match &expr.kind {
            ExprKind::Call(callee, _) => {
                if let ExprKind::Path(qpath) = &callee.kind {
                    if let Some(def_id) = cx.qpath_res(qpath, callee.hir_id).opt_def_id() {
                        if !def_id.is_local() && cx.tcx.crate_name(def_id.krate).as_str() == "regex"
                        {
                            span_lint(
                                cx,
                                NO_STRING_PARSING_IN_CORE,
                                expr.span,
                                "use of the `regex` crate is forbidden in diffviz-core; parse with Tree-sitter instead",
                            );
                        }
                    }
                }
            }
            ExprKind::MethodCall(segment, receiver, _, _) => {
                if let Some(def_id) = cx.typeck_results().type_dependent_def_id(expr.hir_id) {
                    if !def_id.is_local() && cx.tcx.crate_name(def_id.krate).as_str() == "regex" {
                        span_lint(
                            cx,
                            NO_STRING_PARSING_IN_CORE,
                            expr.span,
                            "use of the `regex` crate is forbidden in diffviz-core; parse with Tree-sitter instead",
                        );
                        return;
                    }
                }

                let name = segment.ident.name.as_str();
                if BANNED_STR_METHODS.contains(&name) {
                    let receiver_ty = cx.typeck_results().expr_ty_adjusted(receiver).peel_refs();
                    let is_string = receiver_ty
                        .ty_adt_def()
                        .is_some_and(|adt| cx.tcx.is_diagnostic_item(sym::String, adt.did()));
                    if receiver_ty.is_str() || is_string {
                        span_lint(
                            cx,
                            NO_STRING_PARSING_IN_CORE,
                            expr.span,
                            format!(
                                "string-based parsing method `{name}` is forbidden in diffviz-core; use Tree-sitter instead"
                            ),
                        );
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    /// Runs `cargo dylint` against a nested fixture crate and returns its stderr.
    ///
    /// Fixtures are real crates under `ui_fixtures/` (not `ui_test_example`
    /// snapshots): the lint gates on the *crate name* `diffviz_core`, and
    /// `dylint_testing`'s example-based UI tests infer crate name from the
    /// fixture's file name, not its declared package name, so a snapshot-style
    /// fixture can't exercise that gate. Shelling out to a standalone crate
    /// sidesteps that, and matching on stderr substrings (rather than a full
    /// `.stderr` snapshot) avoids compiletest_rs's backslash-normalization
    /// quirk and stays robust to unrelated rustc/clippy message changes.
    fn run_dylint_on_fixture(fixture_dir: &str) -> String {
        let lint_crate_dir = env!("CARGO_MANIFEST_DIR");
        let manifest_path = Path::new(lint_crate_dir)
            .join("ui_fixtures")
            .join(fixture_dir)
            .join("Cargo.toml");

        let output = Command::new("cargo")
            .args(["dylint", "--path", lint_crate_dir, "--manifest-path"])
            .arg(&manifest_path)
            .arg("--all")
            .current_dir(lint_crate_dir)
            .output()
            .expect("failed to run `cargo dylint`");

        String::from_utf8_lossy(&output.stderr).into_owned()
    }

    #[test]
    fn fires_on_violations_in_diffviz_core() {
        let stderr = run_dylint_on_fixture("diffviz_core");

        assert!(
            stderr.contains("use of the `regex` crate is forbidden"),
            "missing regex-crate violation:\n{stderr}"
        );
        assert!(
            stderr.contains("string-based parsing method `split` is forbidden"),
            "missing split violation:\n{stderr}"
        );
        assert!(
            stderr.contains("string-based parsing method `find` is forbidden"),
            "missing find violation:\n{stderr}"
        );
        assert!(
            !stderr.contains("allowed_ast_like_usage"),
            "lint fired on allowed str methods (starts_with/ends_with/contains):\n{stderr}"
        );
    }

    #[test]
    fn stays_silent_outside_diffviz_core() {
        let stderr = run_dylint_on_fixture("other_crate");

        assert!(
            !stderr.contains("no_string_parsing_in_core"),
            "lint fired outside diffviz_core:\n{stderr}"
        );
    }
}
