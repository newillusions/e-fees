# KB Critical Rules

*Auto-generated from KB critical observations (priority 9-10)*
*Last updated: 2026-02-03*

---

## E Fees Tailwind Usage Rules

**Priority:** 10 | **Type:** pattern

CRITICAL RULE for E-Fees frontend development: Tailwind must NEVER be used with massive inline class strings. Extract repeated patterns into components or CSS classes. Use existing emittiv-* classes. Use semantic text sizes (text-xs, text-xxs) not arbitrary values.

*Source: observation:hfruicpknmxiuobmmjiq*

---

## E Fees Testing

**Priority:** 9 | **Type:** pattern

CRITICAL: E-Fees is a Tauri desktop app - Playwright/browser testing DOES NOT WORK. When testing via MCP, use Tauri MCP tools (get_dom, execute_js, take_screenshot). For screenshots, use Peekaboo MCP.

*Source: observation:nh56ye6stdh2t1l84169*

---

## Process Management Safety

**Priority:** 10 | **Type:** pattern

CRITICAL RULE: NEVER use pkill, killall, or broad pattern-matching kill commands without EXPLICIT user permission. Always ask first, use specific PIDs.

*Source: observation:yz1x62rdud27ejx4q83f*

---

## Claude Code Token Efficiency

**Priority:** 10 | **Type:** pattern

CRITICAL RULE: All Claude Code instances MUST maximize token usage efficiency and minimize costs. When exploring codebases, use Task tool with Explore agent. Use progressive KB loading (Layer 1 first).

*Source: observation:3uq1ed30fi5tb0qiswq1*

---

## Plugin Mcp Servers Require .mcp.json

**Priority:** 10 | **Type:** problem

CRITICAL: Claude Code's mcpServers field in plugin.json is SILENTLY IGNORED. MCP servers must be defined in project .mcp.json files.

*Source: observation:7j601cg9s5b78aqw165g*

---

## Plugin Hooks Location Confusion

**Priority:** 10 | **Type:** problem

CRITICAL WARNING: Confusion about where Claude Code plugin hooks should be defined. Two formats exist - verify correct location.

*Source: observation:bxkjww346sbmxmx5w4gi*

---

## Plugin Cache Orphaning Disables Everything

**Priority:** 10 | **Type:** problem

CRITICAL BUG: Claude Code plugin cache orphaning can disable an entire plugin silently. When ALL cache entries become orphaned, plugin stops loading.

*Source: observation:0q5xix9f6jtr8qj3vkc8*

---

## Claude Code Plugin Environment Variables

**Priority:** 10 | **Type:** pattern

CRITICAL: Claude Code's plugin system does NOT support bash-style variable expansion in plugin.json. Use env field or hardcode values.

*Source: observation:d8d73tr4jbllcd1nrj4n*

---

## Docker Network Preferences

**Priority:** 9 | **Type:** config

STRONG PREFERENCE: Use container hostname resolution, NOT static IPs. The system uses DHCP for containers.

*Source: observation:ewv42s8nzoayqiqi79z1*

---

## Unraid Container Management

**Priority:** 10 | **Type:** pattern

CRITICAL RULE: Unraid container deployment, configuration, and management MUST be done through the Unraid WebGUI or docker templates.

*Source: observation:gwysyzyl7amywblr8t6c*

---
