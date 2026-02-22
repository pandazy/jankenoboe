# v2.2.1 — Skills reorganized to .claude/skills/

## Changes

### Skills file structure reorganized
- **Moved all skills from `skills/` to `.claude/skills/`** — follows the [Claude Code skills convention](https://code.claude.com/docs/en/skills) for automatic discovery from nested directories.
- **Import helper scripts moved to `scripts/` subdirectory** — `parse_amq_import.py`, `import_amq.py`, `check_artists.sh`, and `check_shows.sh` now live under `.claude/skills/importing-amq-songs/scripts/`.
- **Updated all path references** in skill files, scripts, and documentation (`AGENTS.md`, `README.md`, `docs/import.md`, `docs/structure.md`).

### New directory layout

```
.claude/skills/
├── querying-jankenoboe/SKILL.md
├── learning-with-jankenoboe/SKILL.md
├── maintaining-jankenoboe-data/SKILL.md
├── reviewing-due-songs/SKILL.md
└── importing-amq-songs/
    ├── SKILL.md
    └── scripts/
        ├── parse_amq_import.py
        ├── import_amq.py
        ├── check_artists.sh
        └── check_shows.sh
```

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/install.sh | sh
```

## Platform Support

Pre-built binaries for:
- Linux x86_64
- Linux ARM64 (aarch64)
- macOS Apple Silicon (aarch64)