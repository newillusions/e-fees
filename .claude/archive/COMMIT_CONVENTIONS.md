# Conventional Commits Specification v1.0.0

**Reference**: https://www.conventionalcommits.org/en/v1.0.0/

This project follows the Conventional Commits specification for all git commit messages. This document serves as a reference for Claude Code when generating commits.

## Summary

The Conventional Commits specification is a lightweight convention on top of commit messages. It provides an easy set of rules for creating an explicit commit history, which makes it easier to write automated tools on top of.

## Commit Message Structure

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Structure Rules

1. **MUST** be prefixed with a type, which consists of a noun (feat, fix, etc.)
2. **MAY** have an optional scope in parentheses after the type
3. **MUST** be followed by a colon and space
4. **MUST** have a description immediately following the type/scope prefix
5. **MAY** have a longer commit body providing additional context (after a blank line)
6. **MAY** have one or more footers (after another blank line)

## Commit Types

### Primary Types (Semantic Versioning)

- **feat**: A new feature (correlates with MINOR in SemVer)
- **fix**: A bug fix (correlates with PATCH in SemVer)
- **BREAKING CHANGE**: Breaking API change (correlates with MAJOR in SemVer)

### Additional Types (Recommended)

- **build**: Changes to build system or external dependencies (webpack, npm, cargo, etc)
- **chore**: Other changes that don't modify src or test files
- **ci**: Changes to CI configuration files and scripts (GitHub Actions, .gitlab-ci.yml, etc)
- **docs**: Documentation only changes
- **perf**: Performance improvement
- **refactor**: Code change that neither fixes a bug nor adds a feature
- **revert**: Reverts a previous commit
- **style**: Changes that don't affect code meaning (white-space, formatting, missing semi-colons)
- **test**: Adding missing tests or correcting existing tests

## Scope

A scope **MAY** be provided after a type. A scope is a phrase describing a section of the codebase enclosed in parentheses.

Examples:
- `feat(parser): add ability to parse arrays`
- `fix(auth): correct token expiration logic`
- `docs(api): update REST endpoint documentation`

## Description

A description **MUST** immediately follow the colon and space after the type/scope prefix. The description is a short summary of the code changes.

Guidelines:
- Use imperative, present tense: "change" not "changed" nor "changes"
- Don't capitalize first letter
- No period (.) at the end
- Keep under 72 characters

## Body

A longer commit body **MAY** be provided after the short description, providing additional contextual information about the code changes. The body **MUST** begin one blank line after the description.

## Footer

One or more footers **MAY** be provided one blank line after the body. Each footer **MUST** consist of a word token, followed by either a `:<space>` or `<space>#` separator, followed by a string value.

Common footers:
- `BREAKING CHANGE: <description>` - Describes breaking changes
- `Fixes #<issue>` - Links to issue tracker
- `Refs #<issue>` - References related issues
- `Reviewed-by: <name>` - Code review attribution
- `Acked-by: <name>` - Acknowledgment

## Breaking Changes

A commit that has a footer `BREAKING CHANGE:`, or appends a `!` after the type/scope, introduces a breaking API change (correlating with MAJOR in SemVer).

Two methods:

### Method 1: Exclamation Mark
```
feat(api)!: remove deprecated v1 endpoints
```

### Method 2: Footer
```
feat(api): remove deprecated endpoints

BREAKING CHANGE: The v1 API endpoints have been removed.
All clients must migrate to v2 endpoints.
```

## Examples

### Commit with description and breaking change footer
```
feat: allow provided config object to extend other configs

BREAKING CHANGE: `extends` key in config file is now used for extending other config files
```

### Commit with `!` to draw attention to breaking change
```
feat!: send an email to the customer when a product is shipped
```

### Commit with scope and `!`
```
feat(api)!: send an email to the customer when a product is shipped
```

### Commit with both `!` and BREAKING CHANGE footer
```
chore!: drop support for Node 6

BREAKING CHANGE: use JavaScript features not available in Node 6.
```

### Commit with no body
```
docs: correct spelling of CHANGELOG
```

### Commit with scope
```
feat(lang): add Polish language
```

### Commit with multi-paragraph body and multiple footers
```
fix: prevent racing of requests

Introduce a request id and a reference to latest request. Dismiss
incoming responses other than from latest request.

Remove timeouts which were used to mitigate the racing issue but are
obsolete now.

Reviewed-by: Z
Refs: #123
```

## Project-Specific Requirements

For this E-Fees project, all commits made by Claude Code should:

1. Follow the Conventional Commits specification exactly
2. Include the Claude Code footer:
   ```
   🤖 Generated with [Claude Code](https://claude.com/claude-code)

   Co-Authored-By: Claude <noreply@anthropic.com>
   ```
3. Use appropriate scopes when applicable:
   - `(auth)` - Authentication/authorization
   - `(api)` - Backend API changes
   - `(ui)` - Frontend UI changes
   - `(database)` - Database schema or queries
   - `(build)` - Build configuration
   - `(ci)` - CI/CD pipeline changes
   - `(deps)` - Dependency updates

## Benefits

- Automatically generating CHANGELOGs
- Automatically determining a semantic version bump
- Communicating the nature of changes to teammates, public, and stakeholders
- Triggering build and publish processes
- Making it easier for people to contribute by allowing them to explore a more structured commit history

## Why Use Conventional Commits?

1. **Automated Versioning**: Enables automatic semantic version bumps based on commit types
2. **Generated Changelogs**: Tools can automatically generate changelogs from commit messages
3. **Clear Communication**: Makes the nature of changes immediately obvious to team members
4. **Better History**: Creates a structured, searchable commit history
5. **CI/CD Integration**: Enables advanced CI/CD workflows based on commit types

## Reference

Full specification: https://www.conventionalcommits.org/en/v1.0.0/
