# Context Document for Parser Refactor

## Behavioral Specification

Refactor the diffviz-core parser subsystem so that language-specific parsing is driven by declarative language descriptors rather than 7 independent hand-written parsers. A generic semantic tree builder consumes these descriptors and guarantees complete byte coverage (every source byte maps to a semantic node). This eliminates ~50-60% code duplication, resolves 4 open bugs as side-effects of correct coverage, and reduces the cost of adding a new language from ~800 lines to ~50 lines. Genuinely unique language logic (Rust impl blocks, C preprocessor, Go naming-convention visibility) is expressed as targeted overrides. JavaScript is promoted from stub to full parser.

## Codebase Patterns to Follow

- **Domain Modeling**: Typed enums with `thiserror` errors, `SemanticUnitType` variants for universal categories
- **Testing**: TDD for bugs (ignored tests as failing specs), fixture-based testing with JSON files
- **Architecture**: Clean layered — core has no dependencies on review/git layers
- **Tree-sitter Only**: No string/regex for code analysis (enforced rule in diffviz-core)
- **Fail Fast**: No defensive programming or fallbacks in diffviz-core
- **Zero Warnings**: All compiler and clippy warnings must be resolved after every change

## Technical Constraints

- **No backward compatibility required** — callers in diffviz-review/diffviz-cli will be ported
- **One-shot migration** — all 7+1 parsers converted together, no transitional period
- **JavaScript promoted** to full parser in this effort
- **CSS/JSON/TOML remain stubs** — data/markup languages with no meaningful semantic structure
- **Existing 44 passing tests + fixtures = correctness oracle** — refactored parsers must produce identical SemanticTree output
- **9 ignored bug tests should pass** after refactor (impl blocks, struct range, TS classification, JS error message)

## Local Repository Skills

- **dev-contribute** — For implementing each phase of the roadmap
- **design-contribute** — For design objectives requiring collaborative refinement
- **contribution-system** — For artifact schemas and folder conventions
- **filing-bugs** — For any new bugs discovered during refactoring

## Research Findings

No research needed — all technologies (tree-sitter, Rust traits, data-driven dispatch) are well-understood and already in use in the codebase.
