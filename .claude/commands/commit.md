# Commit Command

Generate a conventional commit message for staged changes.

## Instructions

When the user runs `/commit`, you should:

1. **Check staged changes**: Run `git diff --cached --stat` to see what's staged
2. **Analyze the changes**: Determine the appropriate commit type and scope
3. **Generate commit message** following the Conventional Commits specification from `.claude/COMMIT_CONVENTIONS.md`
4. **Present to user** for approval before committing

## Commit Message Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

## Common Types

- **feat**: New feature (triggers MINOR version bump)
- **fix**: Bug fix (triggers PATCH version bump)
- **docs**: Documentation only changes
- **style**: Formatting, missing semicolons, etc (no code change)
- **refactor**: Code change that neither fixes a bug nor adds a feature
- **perf**: Performance improvement
- **test**: Adding or updating tests
- **build**: Build system or external dependencies (webpack, npm, etc)
- **ci**: CI configuration files and scripts (GitHub Actions, etc)
- **chore**: Other changes that don't modify src or test files
- **revert**: Reverts a previous commit

## Breaking Changes

- Add `!` after type/scope: `feat(api)!: remove deprecated endpoint`
- Or use footer: `BREAKING CHANGE: description of what changed`

## Examples

```
feat(auth): add OAuth2 authentication

fix: correct null pointer in user profile
fixes #123

docs: update installation instructions

refactor(database)!: migrate from MongoDB to PostgreSQL

BREAKING CHANGE: Database layer completely rewritten
```

## Usage Notes

- Keep description under 72 characters
- Use imperative mood: "add" not "added" or "adds"
- Don't capitalize first letter of description
- No period at end of description
- Always add the Claude Code footer:

```
🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```
