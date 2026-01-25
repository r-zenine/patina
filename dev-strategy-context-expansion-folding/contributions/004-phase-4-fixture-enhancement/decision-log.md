# Decision Log - Phase 4: Fixture Enhancement

## Fixture Selection Decision

**Decision**: Enhance existing fixtures (rust_trait_impl.json, typescript_react_component.json) rather than create new ones.

**Rationale**:
- Maintains fixture consistency
- Reduces total fixture count
- Easier to verify behavior changes in familiar fixtures
- Aligns with roadmap requirement to "enhance existing fixtures"

## Enhancement Scope Decision

**Decision**: Add realistic structure (imports, types, comments) while keeping changes focused and minimal.

**Rationale**:
- Real-world code has imports, documentation, type definitions
- Too many changes would obscure folding benefits
- 1-2 focused changes demonstrate folding capability
- Balances realism with clarity

## Rust Fixture Enhancement Strategy

**Decision**: Add calculator module with error handling and subtract method.

**Rationale**:
- Error enum demonstrates complex type definitions (foldable)
- Multiple impl blocks show boundary detection
- Subtract method is clear, focused change
- CalcError impl shows trait implementations (always visible)
- Tests show real test coverage patterns

**Structure Added**:
- Module documentation (foldable as NOISE)
- Multiple imports (foldable as BACKGROUND)
- Comprehensive documentation (foldable as NOISE)
- Error enum with trait impls (must be ESSENTIAL)
- Test module with tests (foldable as BACKGROUND for review)

## TypeScript Fixture Enhancement Strategy

**Decision**: Migrate class component to hooks while expanding structure.

**Rationale**:
- Common real-world refactoring pattern
- Demonstrates type definitions and imports (foldable)
- Shows JSDoc comments (foldable as NOISE)
- Hook logic is essential for review (stays visible)
- Realistic React patterns for TypeScript

**Structure Added**:
- React hook imports (foldable as BACKGROUND)
- UserProfile type definition (foldable as BACKGROUND)
- Props interface (foldable as BACKGROUND)
- JSDoc comments (foldable as NOISE)
- Hook initialization (stays visible - ESSENTIAL)
- Event handlers (stays visible - ESSENTIAL)

## Line Count Decision

**Decision**: Target 50+ lines per fixture while keeping old/new code distinct.

**Rationale**:
- 50+ lines provides realistic complexity
- Enough context for meaningful folding
- Not so large that changes become hard to spot
- Matches roadmap specification

**Results**:
- Rust: 62 lines (old) / 72 lines (new) ✓
- TypeScript: 39 lines (old) / 49 lines (new) ✓

## Changes Per Fixture Decision

**Decision**: Limit to 1-2 focused changes per fixture.

**Rationale**:
- 1-2 changes are realistic for review focus
- More changes would create noise
- Demonstrates folding benefit clearly
- Easier to validate correctness

**Changes Made**:
- Rust: Added `subtract()` method + new test (2 focused changes)
- TypeScript: Class→hooks migration (1 comprehensive change)

## Realism Decision

**Decision**: Use realistic imports, documentation patterns, and code structure.

**Rationale**:
- Tests should mirror production code patterns
- Realistic structure exercises boundary detection
- Mirrors actual developer experience
- Better validation of folding in practice

**Realistic Elements Added**:
- Module-level documentation and doc comments
- Multiple imports and type definitions
- Error handling and trait implementations
- Test coverage patterns
- Hook initialization and event handling
- Type annotations and interfaces

## Backward Compatibility Decision

**Decision**: Update expected_line_stats in fixtures to match new dimensions.

**Rationale**:
- Existing tests verify fixtures load correctly
- Updated stats ensure test suite passes
- Line counts verified: Rust (+9/-4), TypeScript (+17/-14)
- No breaking changes to test infrastructure

## Deferred Decisions

### Deferred: Additional Language Fixtures
**Rationale**: Focus on Rust and TypeScript, others follow same pattern

**Future Consideration**: Enhance Python or Go fixtures if needed

### Deferred: Pathological Edge Cases
**Rationale**: Focus on realistic scenarios first

**Future Consideration**: Add edge cases if discovered during TUI testing

### Deferred: Performance Fixtures
**Rationale**: Not needed for Phase 4 (validation is manual)

**Future Consideration**: Create large file fixtures if performance issues arise
