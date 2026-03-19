# Core-then-Integrate

## Description

Build pure business logic first (no external dependencies), then add external integrations layer by layer.

**Process**: Domain Logic → Interface Design → Adapter Implementation → Integration

## When to Choose

- Complex business rules that need to be tested in isolation
- Clean architecture goals (ports and adapters)
- Heavy external dependencies (multiple databases, APIs, queues)
- Business logic must be verifiable without infrastructure

## Execution Phases

### Phase 1: Domain Logic
- Implement pure business logic with zero external dependencies
- All logic is unit-testable in isolation
- No database calls, no API calls, no file system access
- Focus: "Does the business logic work correctly?"

### Phase 2: Interface / Port Design
- Define clear interface contracts between core and external systems
- Specify what the adapters must implement
- No concrete implementations yet — only contracts (traits, interfaces, types)
- Focus: "What shape does the integration need to have?"

### Phase 3: Adapter Implementation
- Implement specific external system adapters (database, API, queue, etc.)
- Each adapter is independently testable
- Adapters implement the interfaces defined in Phase 2
- Focus: "Does each adapter correctly implement the contract?"

### Phase 4: Integration
- Wire all components together
- End-to-end integration tests
- Handle cross-cutting concerns (error propagation, logging, transactions)
- Focus: "Does the whole system work together?"

## Contribution Folder Naming

```
NNN-phase-X-domain-logic-core-[agent]
NNN-phase-X-port-design-core-[agent]
NNN-phase-X-adapter-database-[agent]
NNN-phase-X-adapter-api-[agent]
NNN-phase-X-integration-orchestrator-[agent]
```

## Quality Checks

- Domain Modeler: Zero external dependencies; all logic unit-testable
- Port Designer: Interfaces are complete and minimal; no implementation detail leaks through
- Adapter Builder: Each adapter independently testable; implements contract fully
- Integration Orchestrator: All components wired; integration tests pass; error handling complete
