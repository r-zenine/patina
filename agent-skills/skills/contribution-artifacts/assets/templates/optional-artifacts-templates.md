# Optional Artifacts Templates

Depending on contribution type, you may also create specialized documentation:

## Performance Contributions

**`performance-report.md`**
```markdown
# Performance Report - Phase X [Contribution Type]

**Baseline**: [Current metrics before changes]
**Results**: [Performance improvements achieved]
**Bottlenecks**: [Issues identified that need addressing]
**Recommendations**: [Next optimization opportunities]
```

**`optimization-recommendations.md`**
```markdown
# Optimization Recommendations - Phase X [Contribution Type]

**High Impact**: [Changes that will provide biggest performance gains]
**Quick Wins**: [Easy improvements that can be done immediately]
**Future Work**: [Larger architectural changes worth considering]
```

## Security Contributions

**`security-scan-results.json`**
Raw automated scan outputs with findings

**`vulnerability-report.md`**
```markdown
# Vulnerability Report - Phase X [Contribution Type]

**Critical**: [Issues requiring immediate attention]
**Medium**: [Security improvements recommended]
**Mitigated**: [Issues addressed in this contribution]
**Monitoring**: [Areas requiring ongoing attention]
```

**`threat-model.md`**
```markdown
# Threat Model - Phase X [Contribution Type]

**Attack Vectors**: [How system could be compromised]
**Mitigations**: [Security controls implemented]
**Residual Risk**: [Known vulnerabilities with accepted risk]
```

## Architecture Contributions

**`integration-map.md`**
```markdown
# Integration Map - Phase X [Contribution Type]

**Data Flow**: [How information moves through system]
**Dependencies**: [External systems and their interfaces]
**Failure Points**: [Where things can break and fallback strategies]
```

**`api-contracts.md`**
```markdown
# API Contracts - Phase X [Contribution Type]

**Endpoints**: [New or modified API endpoints]
**Request/Response**: [Data structures and validation rules]
**Breaking Changes**: [What existing integrations need to update]
```

## Documentation Contributions

**`user-guide.md`**
```markdown
# User Guide - Phase X [Contribution Type]

**Getting Started**: [Basic usage steps]
**Common Tasks**: [Most frequent user workflows]
**Troubleshooting**: [Common issues and solutions]
```

**`developer-guide.md`**
```markdown
# Developer Guide - Phase X [Contribution Type]

**Setup**: [How to work with this code]
**Extension Points**: [How to add new functionality]
**Testing**: [How to validate changes]
```