## v2.5.2

### New: `initialize` Agent Skill

Added a dedicated `initialize` skill (`.claude/skills/initialize/SKILL.md`) that all other skills now reference. It provides a 4-step setup flow:

1. Verify CLI installation
2. Check `JANKENOBOE_DB` (only prompts if unset)
3. Verify database file exists
4. First-time database creation (with safety warning against running on existing databases)

This prevents agents from unnecessarily setting `JANKENOBOE_DB` when it's already configured.

### Documentation Reorganization

- Moved design docs (`concept.md`, `development.md`, `import.md`, `structure.md`) into versioned `docs/design/v1/` folder
- Renamed `amq_song_export-sample.json` → `amq_song_export-small.json`
- Simplified README: removed redundant setup walkthrough, linking to the initialize skill instead
- All 5 existing skills now use a single-line setup reference instead of inline DB path instructions

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/install.sh | sh
```

## Platform Support

Pre-built binaries for:
- Linux x86_64
- Linux ARM64 (aarch64)
- macOS Apple Silicon (aarch64)