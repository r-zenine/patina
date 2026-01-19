# Claude Code Tools: Skills vs Agents vs Commands

A comprehensive guide to understanding when and how to use Claude Code's three primary extensibility mechanisms.

## Overview

Claude Code provides three distinct types of tools for extending functionality:

- **Skills**: User-defined, reusable workflows for specific domains
- **Agents**: Built-in specialized sub-agents for complex multi-step tasks  
- **Commands**: Interactive CLI commands and built-in tool functions

## Skills

### What They Are
Skills are user-defined YAML configurations that create reusable workflows for specific domains. They act as specialized modes that Claude can enter to handle particular types of tasks with domain expertise.

### Structure
```yaml
---
name: skill-name
description: When and how to use this skill
allowed-tools: ["Tool1", "Tool2", "Tool3"]
---

# Skill documentation in markdown
```

### Key Characteristics
- **User-defined**: You create and maintain them
- **Domain-specific**: Focused on particular problem areas
- **Tool-restricted**: Limited to specified allowed-tools
- **Process-oriented**: Define structured workflows
- **Reusable**: Can be invoked across different sessions

### Best Use Cases
- **Complex multi-step workflows** requiring domain expertise
- **Strategic planning tasks** like dev-strategy planning
- **Specialized analysis** with specific methodologies
- **Repetitive processes** that benefit from standardization
- **Knowledge-intensive tasks** requiring specific approaches

### Current Examples
- `dev-strategy`: Creates comprehensive implementation plans for complex coding projects
- `dev-contribute`: Enables structured contributions to dev-strategy plans with proper documentation

### When to Use Skills
✅ **Use for:**
- Processes requiring 5+ coordinated steps
- Domain expertise that spans multiple projects
- Workflows needing consistent methodology
- Tasks requiring specific tool combinations
- Reusable problem-solving approaches

❌ **Don't use for:**
- One-off tasks or simple operations
- General programming questions
- Tasks requiring tools not in allowed-tools list

## Agents

### What They Are
Agents are built-in specialized sub-agents that Claude can spawn to handle complex, autonomous tasks. They run independently and return results.

### Key Characteristics
- **Built-in**: Provided by Claude Code system
- **Autonomous**: Operate independently once launched
- **Tool-rich**: Have access to comprehensive tool sets
- **Specialized**: Each type optimized for specific task categories
- **Stateless**: Each invocation is independent

### Available Agent Types
- **general-purpose**: Research, code search, multi-step tasks (Tools: *)
- **statusline-setup**: Configure Claude Code status line (Tools: Read, Edit)  
- **output-style-setup**: Create Claude Code output styles (Tools: Read, Write, Edit, Glob, LS, Grep)
- **onboarding-agent**: Generate standardized onboarding documentation (Tools: Bash, Glob, Grep, Read, WebFetch, TodoWrite, WebSearch, BashOutput, Write, KillShell)

### When to Use Agents
✅ **Use for:**
- Complex searches requiring multiple attempts
- Tasks needing autonomous exploration
- System configuration that requires expertise
- Documentation generation from code analysis
- Multi-step research with uncertain scope

❌ **Don't use for:**
- Simple file reads or searches
- Tasks you can complete directly
- When you know exactly what files to examine

### Usage Pattern
```
Task {
    subagent_type: "agent-type",
    description: "Brief task description", 
    prompt: "Detailed autonomous task instructions"
}
```

## Commands

### What They Are
Commands encompass both interactive CLI commands and Claude's built-in tool functions for direct system interaction.

### Types

#### CLI Commands
- Interactive shell-style commands
- Direct system operations  
- Built-in Claude Code features

#### Built-in Tools
- **File Operations**: Read, Write, Edit, MultiEdit
- **Search**: Grep, Glob, LS
- **Development**: Bash, Git operations
- **Web**: WebFetch for external content
- **Notebooks**: NotebookEdit for Jupyter
- **Task Management**: TodoWrite

### Key Characteristics
- **Immediate**: Direct execution with instant feedback
- **Granular**: Single-purpose operations
- **System-integrated**: Direct OS and tool interaction
- **Stateful**: Maintain session context

### When to Use Commands
✅ **Use for:**
- Direct file operations
- Simple searches with known targets
- Immediate system interactions
- Single-step operations
- Quick explorations

❌ **Don't use for:**
- Complex multi-step processes
- Tasks requiring domain methodology
- Repetitive workflows needing standardization

## Decision Matrix

| Task Type | Tool Choice | Reasoning |
|-----------|-------------|-----------|
| Create implementation roadmap | **Skill** (dev-strategy) | Complex planning requiring structured methodology |
| Find a specific function | **Command** (Grep/Read) | Direct search with known target |
| Generate project documentation | **Agent** (onboarding-agent) | Autonomous analysis requiring comprehensive exploration |
| Configure Claude Code settings | **Agent** (statusline-setup) | Specialized system configuration |
| Fix a simple bug | **Commands** (Read, Edit, Bash) | Direct operations sufficient |
| Implement a feature phase | **Skill** (dev-contribute) | Structured workflow with documentation requirements |
| Search codebase uncertainly | **Agent** (general-purpose) | May require multiple search rounds |
| Read specific file | **Command** (Read) | Single direct operation |

## Best Practices

### For Skills
- Create skills for workflows you'll repeat across projects
- Define clear allowed-tools based on actual needs
- Include comprehensive documentation and examples
- Use structured processes (numbered steps, phases)
- Focus on methodology over implementation details

### For Agents  
- Use when task scope is uncertain
- Provide detailed autonomous instructions
- Leverage when built-in expertise is valuable
- Remember they're stateless - include all context

### For Commands
- Use for immediate, direct operations
- Batch related commands when possible
- Prefer specific tools over general ones
- Use for interactive exploration and testing

## Evolution Path

1. **Start with Commands** for immediate needs
2. **Identify patterns** that could benefit from structure  
3. **Create Skills** for recurring complex workflows
4. **Use Agents** when autonomous expertise adds value

The key is matching the tool's strengths to your task's characteristics: Skills for structured domain workflows, Agents for autonomous complex tasks, and Commands for direct immediate operations.
