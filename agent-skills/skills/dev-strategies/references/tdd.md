# TDD — Test-Driven Development

## Description

Write sufficient tests first, then implement code to pass those tests.

**Process**: Test Design → Test Validation → Implementation

## Execution Phases

### Phase: Test Design
- Read the behavioral spec and phase objectives
- Create failing tests that prove requirements are met
- Follow Test Pyramid: many unit tests, some integration tests, few E2E tests
- Keep tests readable: express behavior, not implementation details

### Phase: Test Validation
- Review tests for sufficient coverage and edge cases
- Verify pyramid proportions are correct
- Ensure tests are readable and maintainable
- Confirm tests are actually failing (not accidentally passing)

### Phase: Implementation
- Write the minimum code necessary to make tests pass
- Refactor for clarity once tests are green
- Do not add functionality beyond what the tests require

## Test Pyramid (Martin Fowler)

**Unit Tests (Many — ~70–80%)**
- Test individual functions, components, and business logic in isolation
- Fast execution, no external dependencies
- Provide immediate feedback during development

**Integration Tests (Some — ~15–25%)**
- Test interactions between components, modules, or services
- Include database connections, API calls, file system operations
- Moderate execution speed, some external dependencies

**End-to-End Tests (Few — ~5–10%)**
- Test key user workflows and critical system paths
- Slow execution, full system dependencies
- Focus on high-value user scenarios

Reference: [Martin Fowler's Practical Test Pyramid](https://martinfowler.com/articles/practical-test-pyramid.html)

## Contribution Folder Naming

```
NNN-phase-X-test-design-tdd-[agent]
NNN-phase-X-test-validation-tdd-[agent]
NNN-phase-X-implementation-code-[agent]
```

## Quality Checks for TDD Contributions

- Test Designer: All tests are failing before implementation starts
- Test Validator: Tests cover requirements and edge cases; pyramid proportions maintained
- Implementation Agent: Minimum code written; no added functionality beyond tests; tests green
