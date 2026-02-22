# E-Fees Slash Commands

Custom slash commands for the E-Fees project. Type `/command-name` in Claude Code to invoke.

## Available Commands

| Command | Purpose | When to Use |
|---------|---------|-------------|
| `/commands` | Command reference guide | Learning available commands |
| `/release` | Full release pipeline | Version bump → build → publish → update manifest |
| `/commit` | Git commit workflow | Committing changes |

## Using Commands

```
You: /release patch
Claude: [Runs full release pipeline: bump, tag, CI build, Forgejo publish, update.json sync]
```

## File Structure

```
.claude/commands/
  ├── README.md        # This file
  ├── commands.md      # Command reference (/commands)
  ├── commit.md        # Git commit (/commit)
  └── release.md       # Release pipeline (/release)
```
