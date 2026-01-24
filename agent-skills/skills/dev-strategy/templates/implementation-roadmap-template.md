# Implementation Roadmap for [FEATURE NAME]

## Execution Strategy

**Strategy**: [Selected strategy name, e.g., "Feature Flag Incremental"]
**Approach**: [Brief description of the overall approach]

---

## Phase 1: [Phase Title]

**Description**: [What will be implemented in this phase - 1-2 sentences]

**Objectives**:
- **Implementation**: [Clear execution step following existing patterns]
- **Design**: [Design decision needed - to be refined based on learnings]
- **Implementation**: [Another clear execution step]

**Testing Criteria**:
- [How to verify objective 1 is complete]
- [How to verify objective 2 is complete]
- [How to verify objective 3 is complete]

**Dependencies**: [None, or "Must complete Phase X first"]

**Relevant Local Skills**: [List applicable skills from context document for this phase, or "None"]

**Files to Modify**:
- `path/to/file1.js` - [What changes will be made]
- `path/to/file2.js` - [What changes will be made]
- `tests/test-file.js` - [Test additions needed]

---

## Phase 2: [Phase Title]

**Description**: [What will be implemented in this phase]

**Objectives**:
- **Implementation**: [Clear execution step following existing patterns]
- **Design**: [Design decision needed - to be refined based on learnings]

**Testing Criteria**:
- [How to verify objective 1 is complete]
- [How to verify objective 2 is complete]

**Dependencies**: [Phase numbers that must complete first]

**Relevant Local Skills**: [List applicable skills from context document for this phase, or "None"]

**Files to Modify**:
- `path/to/file3.js` - [What changes will be made]
- `path/to/file4.js` - [What changes will be made]

---

## Phase 3: [Phase Title]

**Description**: [What will be implemented in this phase]

**Objectives**:
- **Implementation**: [Clear execution step following existing patterns]
- **Design**: [Design decision needed - to be refined based on learnings]

**Testing Criteria**:
- [How to verify objective 1 is complete]
- [How to verify objective 2 is complete]

**Dependencies**: [Phase numbers that must complete first]

**Relevant Local Skills**: [List applicable skills from context document for this phase, or "None"]

**Files to Modify**:
- `path/to/file5.js` - [What changes will be made]
- `path/to/file6.js` - [What changes will be made]

---

## Implementation Notes

### Common Patterns for Phases:

**For New Features**:
1. Setup/Infrastructure - Dependencies, config, database setup
2. Core Logic - Main business logic implementation
3. Integration - Connect to existing systems
4. UI/API - User-facing interfaces
5. Testing & Polish - Sufficient testing, error handling

**For Refactoring**:
1. Preparation - Add tests for existing behavior
2. Extract - Move code to new structure
3. Replace - Update callers to use new structure
4. Cleanup - Remove old code
5. Validation - Ensure no regressions

### Writing Good Objectives:

**Implementation Objectives** (clear execution following existing patterns):
- ✅ "Add JWT refresh token endpoint to AuthController"
- ✅ "Update frontend login form to handle refresh tokens"
- ❌ "Improve authentication"

**Design Objectives** (require design decisions to be refined later):
- ✅ "Design error handling strategy for auth failures"
- ✅ "Determine optimal session management approach"
- ❌ "Make it better"

### Essential Testing Criteria:
- ✅ "User can log in and receive both access and refresh tokens"
- ✅ "Expired access token automatically refreshes without user action"
- ❌ "Authentication works"
- ❌ "No bugs"

### File Modification Guidelines:
- Be specific about what changes (add function, update class, etc.)
- Include test files that need updates
- Consider configuration files that might need changes
- Include migration files if database changes are needed