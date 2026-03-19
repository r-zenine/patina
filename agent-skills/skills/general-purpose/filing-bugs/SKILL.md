---
name: Filing Bugs
description: File a bug by adding a failing test case to the codebase marked as skipped with clear documentation. Use anytime the user reports a bug—create a test that reproduces it, mark it as skipped in the testing framework, and document the expected vs actual behavior.
---

## What Success Looks Like

A bug is successfully filed when:

1. **A failing test exists** in the codebase that reproduces the bug—the test fails with the current code
2. **Test is marked as skipped** using the testing framework's skip/ignore syntax (e.g., `@pytest.mark.skip`, `it.skip`, `#[ignore]`, etc.)
3. **Clear comments explain the bug** including:
   - What the bug is (concise description)
   - What behavior is expected
   - What behavior actually occurs
4. **Test is organized properly**:
   - Uses the project's existing bug test structure (e.g., `tests/bugs/`, `bugs/` directory) if one exists
   - If no structure exists, creates one (e.g., `tests/bugs/issue_name.py`)
5. **Test is minimal** and focuses only on reproducing the bug—no extraneous code

## Procedural Guidance

### 1. Identify the Testing Framework

- Examine the codebase for test files (look for patterns like `*_test.go`, `test_*.py`, `*.test.js`, etc.)
- Check `package.json`, `Cargo.toml`, `pyproject.toml`, or equivalent for testing dependencies
- Determine the framework: pytest, Jest, cargo test, Mocha, Go testing, JUnit, etc.

### 2. Understand the Bug

- Listen to the user's description of the bug
- Identify:
  - What the expected behavior should be
  - What the actual behavior is
  - The minimal steps to reproduce it

### 3. Locate or Propose Test Organization

- **If the project has an existing bug test structure** (e.g., `tests/bugs/`, `tests/regression/`): Use it
- **If no structure exists**: Propose creating one. Common patterns:
  - `tests/bugs/` (for active bugs)
  - `tests/regression/` (for fixed bugs)
  - Or inline in existing test files with clear comments
- Ask the user to confirm the location

### 4. Write a Minimal Failing Test

- Create a test function/case that:
  - Sets up the minimal state needed to trigger the bug
  - Performs the action that exposes the bug
  - Asserts the expected behavior (which should fail with current code)
- Keep the test focused—avoid testing other functionality
- Use clear, readable variable and function names

### 5. Mark Test as Skipped

- Use the testing framework's skip/ignore syntax to prevent the test from running by default
- Add a comment above the skip marker explaining the bug:
  ```
  # Bug: [Brief description of the bug]
  # Expected: [What should happen]
  # Actual: [What happens instead]
  @pytest.mark.skip
  def test_bug_something():
      ...
  ```

### 6. Verify the Test Fails

- Run the test without the skip marker to confirm it fails with the expected error
- This validates that the test actually reproduces the bug
- Re-apply the skip marker when confirmed
