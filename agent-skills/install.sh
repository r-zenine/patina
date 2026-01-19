#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
CLAUDE_DIR="$HOME/.claude"
REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo -e "${GREEN}Installing Claude Code skills and agents...${NC}"
echo "Repository: $REPO_DIR"
echo "Target: $CLAUDE_DIR"
echo

# Create .claude directory if it doesn't exist
if [[ ! -d "$CLAUDE_DIR" ]]; then
    echo -e "${YELLOW}Creating $CLAUDE_DIR directory...${NC}"
    mkdir -p "$CLAUDE_DIR"
fi

# Create skills directory if it doesn't exist
if [[ ! -d "$CLAUDE_DIR/skills" ]]; then
    echo -e "${YELLOW}Creating $CLAUDE_DIR/skills directory...${NC}"
    mkdir -p "$CLAUDE_DIR/skills"
fi

# Create agents directory if it doesn't exist
if [[ ! -d "$CLAUDE_DIR/agents" ]]; then
    echo -e "${YELLOW}Creating $CLAUDE_DIR/agents directory...${NC}"
    mkdir -p "$CLAUDE_DIR/agents"
fi

# Function to create symlink with backup
create_symlink() {
    local source="$1"
    local target="$2"
    local name="$3"
    
    if [[ -L "$target" ]]; then
        echo -e "${YELLOW}Removing existing symlink: $target${NC}"
        rm "$target"
    elif [[ -e "$target" ]]; then
        echo -e "${YELLOW}Backing up existing $name: ${target}.backup${NC}"
        mv "$target" "${target}.backup"
    fi
    
    echo "Linking $name..."
    ln -sf "$source" "$target"
}

# Install skills
if [[ -d "$REPO_DIR/skills" ]]; then
    for skill_dir in "$REPO_DIR"/skills/*/; do
        if [[ -d "$skill_dir" ]]; then
            skill_name=$(basename "$skill_dir")
            echo -e "${GREEN}Installing skill: $skill_name${NC}"
            create_symlink "$skill_dir" "$CLAUDE_DIR/skills/$skill_name" "$skill_name skill"
        fi
    done
else
    echo -e "${YELLOW}No skills directory found in repository${NC}"
fi

# Install agents  
if [[ -d "$REPO_DIR/agents" ]]; then
    for agent_file in "$REPO_DIR"/agents/*.md; do
        if [[ -f "$agent_file" ]]; then
            agent_name=$(basename "$agent_file")
            echo -e "${GREEN}Installing agent: $agent_name${NC}"
            create_symlink "$agent_file" "$CLAUDE_DIR/agents/$agent_name" "$agent_name agent"
        fi
    done
else
    echo -e "${YELLOW}No agents directory found in repository${NC}"
fi

echo
echo -e "${GREEN}Installation complete!${NC}"
echo
echo "Installed to: $CLAUDE_DIR"
echo "Skills:"
ls -la "$CLAUDE_DIR/skills" 2>/dev/null | grep "^l" | awk '{print "  - " $9}' || echo "  No skills installed"
echo "Agents:"
ls -la "$CLAUDE_DIR/agents" 2>/dev/null | grep "^l" | awk '{print "  - " $9}' || echo "  No agents installed"
echo
echo -e "${YELLOW}Note: Restart Claude Code to use the new skills and agents.${NC}"