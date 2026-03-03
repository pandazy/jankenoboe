## v2.5.3

### Consolidated `.clinerules` into `AGENTS.md`

Merged all content from `.clinerules` into `AGENTS.md` and deleted the file, eliminating redundancy between the two configuration files.

**Key changes:**
- `AGENTS.md` is now the single source of truth for developer/agent guidelines (architecture, conventions, domain concepts, JankenSQLHub integration)
- `README.md` remains the operator-facing entry point (project intro, installation, CLI usage, agent skills)
- Added [JankenSQLHub usage skill](https://github.com/pandazy/jankensqlhub/blob/main/.claude/skills/using-jankensqlhub/SKILL.md) reference for developers
- Replaced outdated database setup instructions with a link to the initialize skill
- Removed "Non-technical users" phrasing

### Files Changed

| File | Change |
|------|--------|
| `.clinerules` | Deleted (merged into AGENTS.md) |
| `AGENTS.md` | Consolidated: developer-focused, no duplicated intro/skills table |
| `README.md` | Wording fix ("Non-technical users" → "Users") |
| `docs/design/v1/development.md` | Removed `.clinerules` from docs checklist |

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/install.sh | sh
```

## Platform Support

Pre-built binaries for:
- Linux x86_64
- Linux ARM64 (aarch64)
- macOS Apple Silicon (aarch64)