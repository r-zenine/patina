# Context Document for Decision-Based Review

## Behavioral Specification

Transform DiffViz from a file-based code review tool into a decision-based code review tool. Reviewers will explore code changes organized by the decisions that produced them, not just by file.

When a reviewer opens a contribution for review:
1. They see a list of decisions (from decision-log.md) instead of just files
2. Selecting a decision shows its rationale, alternatives considered, and all code impacts
3. Code that implements a decision appears with decision context (number, title)
4. If two decisions affect the same code, that code appears under both decisions (reviewed twice)
5. File-based navigation remains available as an alternative view

## Codebase Patterns to Follow

### Entity Design (diffviz-review)
- Entities live in `src/entities/` with dedicated files
- Entities indexed by `ReviewableDiffId` for consistency
- Use `BTreeMap` for ordered collections
- Follow existing patterns: Comment, Approval, Instruction

### State Management (diffviz-review)
- `ReviewState` is the centralized state container
- Update methods return `&mut Self` for chaining
- External mutations only through controlled methods

### TUI Architecture (diffviz-review-tui)
- `UiState` for pure UI state (navigation, scroll, modes)
- `ReviewEngine` for business operations
- Event system: UiEvent (navigation) → BusinessEvent (review ops)
- Clean separation between UI and business logic

### Navigation Pattern (diffviz-review-tui)
- `NavigationState` handles hierarchical selection
- Collapsible lists with keyboard-driven traversal
- View modes for different display contexts

## Technical Constraints

- **diffviz-core stays pure**: No decision concepts in core - it's semantic analysis only
- **Decisions in diffviz-review**: Model decisions as review workflow metadata
- **No fallbacks**: Fail-fast approach per CLAUDE.md guidelines
- **Function-level granularity**: Map decisions to function blocks, not exact diff lines
- **Three-level confidence**: high/medium/low for mapping certainty
- **No backward compatibility**: Decision-based review only works with new contributions that have mappings

## User Decisions Captured

| Decision | Choice |
|----------|--------|
| Overlapping code | Same code reviewed under both decisions |
| Range precision | Function-level |
| Confidence levels | Three-level (high/medium/low) |
| No-code decisions | Include with empty code_impacts |
| Mapping generation | Fully automatic by dev-contribute |
| Backward compat | No - decision-based only for new contributions |
| Comments | Keep current line-based visual selection |
| Decision layer | diffviz-review only (keep diffviz-core pure) |

## JSON Schema (Agreed)

```json
{
  "format_version": "1.0",
  "contribution_id": "001-phase-1-...",
  "decisions": [
    {
      "number": 1,
      "title": "Decision Title",
      "summary": "Brief description",
      "decision_log_line": 15,
      "code_impacts": [
        {
          "file": "path/to/file.rs",
          "line_ranges": [{"start": 10, "end": 25}],
          "change_type": "modification",
          "confidence": "high",
          "reasoning": "This function implements the decision"
        }
      ]
    }
  ]
}
```
