# Skills vs Sub Agents: Design Trade-off Analysis

**Research Date:** 2026-01-08
**Context:** Designing compliance/audit tools that leverage dev-strategy and dev-contribute context
**Decision:** Architecture pattern selection for new specialized tools

## Executive Summary

Analyzed the trade-offs between implementing compliance/audit tools as Claude Code Skills vs Task sub agents. Recommends a **hybrid approach** using skills for user-facing orchestration and sub agents for specialized analysis, mirroring the successful dev-strategy + onboarding-agent pattern.

## Research Methodology

1. **Current Architecture Analysis** - Examined existing skills and sub agent patterns
2. **Context Sharing Evaluation** - Analyzed how different approaches access dev-strategy artifacts
3. **User Experience Assessment** - Evaluated direct invocation vs. orchestrated execution
4. **Maintenance Overhead Comparison** - Development and operational complexity analysis

## Current Architecture Patterns

### Skills Architecture
- **dev-strategy**: High-level orchestration, user-facing (`/dev-strategy`), generates structured artifacts
- **dev-contribute**: Execution framework, works with dev-strategy context, maintains state through numbered contributions
- **read-contribution**: Specialized reader, lightweight, progressive disclosure navigation

**Key Characteristics:**
- User-invokable with slash commands
- Direct file system access to project artifacts
- State management through persistent artifact structure
- Can invoke other skills and sub agents

### Sub Agents Architecture
- **onboarding-agent**: Specialized code analysis, triggered by skills, independent operation
- Invoked via `Task` tool with specific `subagent_type`
- Focused, single-purpose operations with clear input/output

**Key Characteristics:**
- Skill/agent-invoked, not directly user-accessible
- Stateless operation on provided inputs
- Specialized expertise in narrow domains
- Lower development overhead for focused tasks

## Design Trade-offs Matrix

| Aspect | Skills | Sub Agents |
|--------|--------|------------|
| **Context Access** | Direct dev-strategy artifact access | Must receive context or re-read artifacts |
| **User Interface** | Direct invocation (`/skill-name`) | Indirect via orchestrating skill/agent |
| **Session Persistence** | Maintains state across user sessions | Single-execution lifecycle |
| **Composition** | Can orchestrate skills + sub agents | Leaf nodes - no orchestration capability |
| **Development Complexity** | Higher - skill metadata, templates, UX | Lower - focused logic implementation |
| **Reusability** | High - permanent user toolkit | Medium - reusable by other skills/agents |
| **State Management** | Project-wide state in artifact structure | Stateless - input → output |
| **Error Handling** | Must handle user interaction complexity | Simpler error propagation to caller |
| **Testing** | Complex - user scenarios + orchestration | Simpler - focused input/output testing |
| **Documentation** | User-facing docs + implementation guide | Implementation-focused documentation |

## Problem Context: Compliance/Audit Tools

**Proposed Tools:**
- Plan compliance verification (implementation vs. original strategy)
- Test quality assessment (coverage vs. functional context)
- Security gap auditing (code analysis with dev context)
- Architecture deviation detection
- Quality metrics reporting

**Context Requirements:**
- Access to dev-strategy planning artifacts
- Understanding of contribution history and decisions
- Integration with existing development workflow
- User-friendly invocation and reporting

## Recommendation: Hybrid Architecture

### User-Facing Skills (Orchestration Layer)

**1. Compliance Auditor Skill** (`/compliance-audit`)
- **Purpose:** Orchestrate comprehensive project compliance analysis
- **Responsibilities:**
  - Read dev-strategy context and contribution history
  - Invoke specialized audit sub agents
  - Generate compliance reports in standard artifact structure
  - Update project tracking with audit findings
- **Sub agents invoked:** plan-compliance-checker, security-gap-auditor

**2. Quality Reviewer Skill** (`/quality-review`)
- **Purpose:** Orchestrate code and test quality assessment
- **Responsibilities:**
  - Analyze test coverage against functional requirements
  - Coordinate architecture and code quality reviews
  - Generate quality improvement recommendations
  - Track quality metrics over time
- **Sub agents invoked:** test-coverage-analyzer, architecture-reviewer

### Specialized Sub Agents (Analysis Layer)

**1. Plan Compliance Checker**
- **Purpose:** Compare implementations against original dev-strategy plan
- **Input:** Dev-strategy artifacts, contribution history, current codebase
- **Output:** Compliance report with deviations and recommendations

**2. Test Coverage Analyzer**
- **Purpose:** Analyze test quality against functional context
- **Input:** Test suite, dev-strategy functional spec, implementation code
- **Output:** Coverage gaps and test quality assessment

**3. Security Gap Auditor**
- **Purpose:** Security-focused code analysis with development context
- **Input:** Codebase, dev-strategy security requirements, contribution decisions
- **Output:** Security findings with contextual risk assessment

**4. Architecture Reviewer**
- **Purpose:** Validate implementation architecture against planned design
- **Input:** Code structure, dev-strategy architecture decisions, patterns
- **Output:** Architecture compliance and deviation analysis

## Architecture Benefits

### User Experience
- **Direct Access:** Users invoke `/compliance-audit` or `/quality-review` directly
- **Consistent Interface:** Follows established skill invocation patterns
- **Comprehensive Reports:** Skills generate structured artifacts matching dev-strategy format

### Technical Excellence
- **Separation of Concerns:** Skills handle orchestration, sub agents focus on analysis
- **Reusability:** Sub agents can be shared across different orchestrating skills
- **Maintainability:** Clear boundaries between user interface and analysis logic
- **Extensibility:** New audit types add sub agents without changing skill interface

### Context Management
- **Efficient Access:** Skills read dev-strategy artifacts once, pass relevant context to sub agents
- **State Coordination:** Skills maintain audit history and cross-analysis coordination
- **Artifact Integration:** Audit results integrate with existing project documentation structure

## Implementation Pattern

```
User: /compliance-audit
├── Compliance Auditor Skill
    ├── Read dev-strategy planning artifacts
    ├── Read contribution history and decisions
    ├── Invoke plan-compliance-checker subagent
    ├── Invoke security-gap-auditor subagent
    ├── Correlate findings across analyses
    ├── Generate compliance-audit-report artifact
    └── Update project audit tracking
```

## Rationale

This hybrid approach:

1. **Follows Proven Patterns:** Mirrors successful dev-strategy + onboarding-agent architecture
2. **Maximizes User Convenience:** Direct skill invocation with comprehensive orchestration
3. **Optimizes Development Efficiency:** Sub agents handle focused analysis without UX complexity
4. **Enables Future Growth:** Easy addition of new audit types and analysis capabilities
5. **Maintains Context Integrity:** Skills manage complex artifact relationships and state
6. **Supports Specialized Expertise:** Sub agents can deep-dive into domain-specific analysis

## Next Steps

1. **Prototype Compliance Auditor Skill** - Start with plan-compliance-checker integration
2. **Develop Plan Compliance Checker Sub Agent** - Focus on dev-strategy artifact comparison
3. **Establish Audit Artifact Templates** - Define output formats for audit reports
4. **Extend to Quality Review Capabilities** - Add test and architecture analysis
5. **Create Audit Dashboard Integration** - Consider how audit results surface to users

## Decision Record

**Decision:** Implement compliance/audit tools using hybrid skills + sub agents architecture
**Alternatives Considered:** Pure skills approach, pure sub agents approach
**Key Factors:** User experience, context management complexity, development maintainability
**Review Date:** To be determined based on initial implementation feedback