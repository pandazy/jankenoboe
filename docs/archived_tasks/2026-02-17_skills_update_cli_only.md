# Task: Update Skills to CLI-Only Interface

**Date:** 2026-02-17
**Status:** Completed

## Objective

Update all Claude Agent Skills to use the `jankenoboe` CLI as the sole interface, removing the previous HTTP API and SQL fallback modes. Align skills with the CLI design spec documented in `docs/cli*.md`.

## Changes Made

### Deleted
- `skills/_shared/` — entire directory removed:
  - `sql-mode-guide.md` — SQL execution guide for zsh
  - `sql/` — 20 SQL template files (read-by-id, search-*, insert-*, update, delete, find-duplicates, learning-*, bulk-reassign-*, due-songs-with-shows, media-urls-for-song)
  - `scripts/generate-review-html.py` — Python HTML generation script
  - `templates/review-due-songs.html` — HTML template for due song reviews

### Rewritten Skills (HTTP API + SQL mode → CLI-only)
- **`skills/querying-jankenoboe/SKILL.md`** — `jankenoboe get`, `search`, `duplicates`, `learning-due` with full field tables and output examples
- **`skills/learning-with-jankenoboe/SKILL.md`** — `learning-due`, `learning-batch`, `update learning` (level up/down/graduate), re-learn workflow with two-step flow
- **`skills/maintaining-jankenoboe-data/SKILL.md`** — `create`, `update`, `delete`, `bulk-reassign` (by song IDs and by artist), merge workflow
- **`skills/reviewing-due-songs/SKILL.md`** — `learning-due` + enrichment via `get`/`search`, plain text output format

### Updated References
- **`AGENTS.md`** — removed "Reference SQL Templates" section
- **`.clinerules`** — removed "Reference SQL Templates" section
- **`docs/structure.md`** — replaced `_shared/` directory tree with flat skill listing

### Not Changed (by design)
- `docs/archived_tasks/2026-02-16_agent_skills_and_agents_md.md` — historical reference, contains old HTTP/SQL mentions which are accurate for that point in time
- `skills/_shared/scripts/generate-review-html.py` — deleted (not updated to CLI); may be reimplemented later if needed
- `README.md` — already clean, no stale references found

## Key Design Decisions

1. **CLI-only**: All skills now exclusively use `jankenoboe` CLI commands. No HTTP or SQL fallback.
2. **Setup section standardized**: Each skill has a consistent setup section requiring `JANKENOBOE_DB` env var.
3. **Reviewing skill simplified**: Instead of SQL-based HTML generation, the reviewing skill uses CLI commands (`learning-due` + `get`/`search`) and presents plain text output.