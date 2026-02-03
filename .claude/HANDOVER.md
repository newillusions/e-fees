# E-Fees Project Handover

## Current Status
Fee Proposal Management desktop app (Tauri v2 + Svelte 5) with SurrealDB backend.
- **Version**: 0.10.27
- **Branch**: `feat/fee-pricing-calculator`
- **App Status**: ✅ Running, dev server active
- **Pricing Module**: ✅ CSS refactoring complete, drag-and-drop implemented, **READY FOR TESTING**

## Last Session (2026-02-01)
**Summary**: Short session investigating kb-agent plugin - confirmed it's working correctly. User attempted to run `/plugin` command which doesn't exist.

### Key Finding
- **kb-agent plugin IS working** - Confirmed via installed_plugins.json and startup hook
- **No `/plugin` command** - User tried running this, but it's not a valid kb-agent command
- **Available commands**: /lamp-on, /lamp-off, /kb, /kb-save, /kb-pause, /kb-feedback, /kb-pin, /kb-setup, /kb-errors, /handoff
- **Plugin location**: `/Users/martin/.claude/plugins/cache/mms-plugins/kb-agent/e7bdda4b520b/`
- **Version**: v3.4.1 (commit f8fb3e7)
- **Session ID**: session:7i79ro3c5kwf9iqxlnwb

## Previous Session (2026-01-27)
**Summary**: Completed CSS refactoring of all pricing panels and implemented drag-and-drop reordering for disciplines using event delegation pattern.

### Completed
1. **Drag-and-drop implementation** ✅
   - Event delegation pattern (container handles drops, rows track hover)
   - Visual feedback (orange border on hover target)
   - Ready for testing (see `DRAG_DROP_READY_FOR_TESTING.md`)

2. **CSS refactoring** ✅
   - Created `IconButton.svelte` and `PanelCard.svelte` components
   - Added CSS classes: `.emittiv-btn`, `.emittiv-inline-input`, `.emittiv-table-input`
   - Fixed icon sizes to 16px, text sizes: text-sm (14px) body, text-base (16px) titles

3. **Pricing panel standardization** ✅
   - All 7 components use consistent compact spacing
   - Tab order: Disciplines → Stages → Costs → Calculator → Payments
   - Default tab changed to Disciplines

## Next Steps
1. **Test drag-and-drop** - Open pricing modal and verify discipline reordering works
2. **Test pricing workflow** - Verify new tab order makes sense for users
3. **Test spacing changes** - Verify all panels look correct after CSS refactoring
4. Consider releasing v0.10.28 with pricing calculator and UI refinements

## Key Technical Context

### Tech Stack
- **Frontend**: Svelte 5 with TypeScript, TailwindCSS
- **Backend**: Tauri v2 (Rust)
- **Database**: SurrealDB @ ws://surreal-dev.internal:8000 (emittiv/projects)
- **Design**: Emittiv brand palette (black/orange theme)

### Critical Rules
1. **Testing**: Use Tauri MCP tools (get_dom, execute_js) - browser tools don't work
2. **Screenshots**: Use Peekaboo MCP - captures modals, background windows
3. **UI Automation**: NEVER use cliclick/osascript - interrupts user's work
4. **Dev command**: Use `npm run tauri:dev` (not `npm run dev`)
5. **MCP socket**: Located at `/tmp/tauri-mcp.sock` - delete if stale
6. **Process safety**: NEVER pkill without permission - ask first, use specific PIDs

### MCP Setup
```json
"tauri-mcp": {
  "command": "node",
  "args": ["/Volumes/base/dev/app/e-fees/tauri-plugin-mcp/mcp-server-ts/build/index.js"]
},
"peekaboo": {
  "command": "npx",
  "args": ["-y", "@steipete/peekaboo-mcp"]
}
```

### Standard Emittiv Spacing
All modal components use:
- Border radius: `rounded` (4px)
- Header padding: `px-3 py-2`
- Text size: `text-xs`
- Icon size: `w-3 h-3`
- Table cells: `px-2 py-1` / `py-1.5`
- Empty states: `p-4`
- Footer sections: `px-3 py-1.5`

## Key Files
| Purpose | Location |
|---------|----------|
| Pricing modal | `src/lib/components/pricing/FeePricingModal.svelte` |
| Calculator panel | `src/lib/components/pricing/PricingCalculatorPanel.svelte` |
| Costs panel | `src/lib/components/pricing/CostsPanel.svelte` |
| Payment schedule | `src/lib/components/pricing/PaymentSchedulePanel.svelte` |
| MCP config | `.mcp.json` |
| App logs | `~/Library/Logs/com.emittiv.e-fees/E-Fees.log` |
| KB plugin | `/Users/martin/.claude/plugins/cache/mms-plugins/kb-agent/e7bdda4b520b/` |

## Troubleshooting

### KB Plugin Commands
- `/lamp-on` - Load context at session start
- `/lamp-off` - Save learnings at session end
- `/kb <query>` - Search knowledge base
- `/kb-save` - Save to KB
- `/kb-feedback` - Report issues/suggestions
- `/kb-setup` - Configure KB connection
- **Note**: `/plugin` command does NOT exist

### If tauri-mcp tools not available
1. Check server running: `ps aux | grep mcp-server-ts`
2. Test server directly: `echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | node build/index.js`
3. Restart Claude Code to reconnect MCP servers
4. Should expose: execute_js, take_screenshot, get_dom, get_element_position, etc.

### If MCP socket fails
1. Check socket: `ls -la /tmp/tauri-mcp.sock`
2. If stale: `rm /tmp/tauri-mcp.sock`
3. Restart app (ask user first, don't pkill blindly)

### Svelte input reactivity
When setting input values via JS:
```javascript
input.value = '100000';
input.dispatchEvent(new Event('input', { bubbles: true }));
input.dispatchEvent(new Event('change', { bubbles: true }));
```

---
*Updated: 2026-02-01*
