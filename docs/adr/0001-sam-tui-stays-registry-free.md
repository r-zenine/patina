# sam-tui keybindings stay registry-free

diffviz-review-tui dispatches keys through a declarative registry (now
`tui_elm::Registry`) so its five key-consuming surfaces cannot drift. sam-tui
deliberately does not: its modal picker has ~12 fixed keys in one match
(`key_transformer`), no modal scopes or leader menu for a registry to key on,
and a key surface gated by per-instance construction flags (`has_options`,
`allow_multi_select`) that static registry rows cannot express. Discovery
stays honest through a single hand-written const (`SAM_BINDINGS` in
`headless.rs`) that feeds both `describe()` and `affordances()`, filtered by
the same flags.

Originally decided as D006 of plan-tui-harness-agent-discovery (log since
deleted; recovered from git history). Reaffirmed after the registry was
extracted into the shared `tui-elm` crate: reuse got cheaper, but the fit
problem — config-gated bindings vs. static scope-keyed rows — is unchanged,
and supporting it would grow the framework for one consumer.

## Considered Options

- **Adopt `tui_elm::Registry`**: rejected — requires adding a runtime-gating
  concept (or non-`'static` registries) to the framework for sam alone, and
  sam has no scopes/leader for the registry's model to describe.
- **Hand-rolled but single-sourced const list**: chosen — sam's two discovery
  surfaces cannot disagree with each other, at ~12 rows of duplication risk
  against dispatch.

## Consequences

`SAM_BINDINGS` must be kept in sync with `key_transformer` by hand; there is
no structural guarantee. Consistency tests in `headless.rs` pin each
documented row (and its gating) to actual dispatch, so drift fails the build
instead of lying to agents. Revisit if sam's key surface grows, if
user-configurable rebinding arrives, or if a third TUI needs config-gated
bindings (then the framework concept pays for itself).
