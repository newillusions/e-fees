#!/bin/bash
# E-Fees Project-Specific Claude Code Launcher
# Uses local MCP configuration with only relevant servers

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MCP_CONFIG="${PROJECT_DIR}/.mcp-config.json"

# Ensure we're in the project directory
cd "$PROJECT_DIR"

# Check if MCP config exists
if [ ! -f "$MCP_CONFIG" ]; then
    echo "❌ Error: MCP config not found at $MCP_CONFIG"
    exit 1
fi

# Display which MCP servers will be loaded
echo "🚀 Starting Claude Code for E-Fees Project"
echo "📁 Project: $PROJECT_DIR"
echo "⚙️  MCP Config: $MCP_CONFIG"
echo ""
echo "🔧 Active MCP Servers:"
echo "  ✅ SurrealDB (emittiv/projects)"
echo "  ✅ tauri-mcp (this project)"
echo "  ✅ filesystem (scoped to e-fees)"
echo "  ✅ git (scoped to e-fees)"
echo "  ✅ desktop-commander"
echo "  ✅ memory"
echo "  ✅ github"
echo "  ✅ sequential-thinking"
echo "  ✅ brave-search"
echo "  ✅ context7"
echo "  ✅ fetch"
echo ""

# Launch Claude Code with project-specific MCP config
exec claude --mcp-config "$MCP_CONFIG" "$@"
