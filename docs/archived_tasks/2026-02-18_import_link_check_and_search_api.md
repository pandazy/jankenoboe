# Task: Import Link Check & Unified Search API

**Date:** 2026-02-18

## Summary

Added show–song link confirmation step to the import workflow and redesigned the `jankenoboe search` CLI API to use a unified `--term` JSON parameter.

## Changes

### 1. Import Workflow — Show–Song Link Check

Added Step 4 "Check Show–Song Link" between song check and play history creation in `docs/import.md`:
- If show and song both exist but aren't linked via `rel_show_song`, confirm linking before proceeding
- Updated the flow diagram with the new decision node
- Added `search rel_show_song` to Database Operations section

### 2. Unified `--term` JSON Search API

Replaced all flag-based search options (`--name`, `--vintage`, `--artist-id`, `--show-id`, `--song-id`, `--match`) with a single `--term` JSON parameter:

```bash
jankenoboe search <table> --fields <fields> --term '<json>'
```

**Term format:**
```json
{
  "<column>": { "value": "<text>", "match": "<mode>" }
}
```

- `match` defaults to `exact` (case-sensitive) when omitted
- Multiple keys combined with AND
- Match modes: `exact`, `exact-i`, `starts-with`, `ends-with`, `contains`

**Searchable columns per table:**
| Table | Columns |
|-------|---------|
| artist | name, name_context |
| show | name, vintage |
| song | name, name_context, artist_id |
| rel_show_song | show_id, song_id |

### 3. Local Reference Fix

Changed `../jankensqlhub` local path references to GitHub repo URL in:
- `AGENTS.md` — Prerequisites
- `docs/development.md` — Prerequisites

## Files Modified

- `docs/import.md` — Added Step 4 link check, updated flow diagram, updated all search commands to `--term`
- `docs/cli-querying.md` — Removed old per-table search sections, added unified `--term` section with match modes, default exact, ID columns, rel_show_song support
- `docs/cli.md` — Updated Operations Coverage (import workflow, fuzzy search sections)
- `skills/querying-jankenoboe/SKILL.md` — Rewrote Search section with exact/fuzzy examples using `--term`
- `skills/maintaining-jankenoboe-data/SKILL.md` — Updated merge workflow search commands
- `skills/reviewing-due-songs/SKILL.md` — Updated song lookup command
- `README.md` — Updated CLI search examples
- `AGENTS.md` — Fixed JankenSQLHub prerequisite reference
- `docs/development.md` — Fixed JankenSQLHub prerequisite reference

### 4. Data Definition Sync Fixes

Cross-checked all design docs against `init-db.sql` and each other. Found and fixed:

**`play_history.status` missing from `init-db.sql`:**
- All docs listed `status` as a field, but `init-db.sql` was missing it
- Added `"status" INTEGER NOT NULL DEFAULT 0` to `play_history` in `init-db.sql`

**`rel_show_song` incorrectly in `get` command:**
- `rel_show_song` has no `id` column (composite key: `show_id + song_id`)
- Removed from `read_by_id` table enum in `cli-querying.md`
- Removed from "Available fields per table" in `skills/querying-jankenoboe/SKILL.md`
- Changed `get rel_show_song` to `search rel_show_song --term` in `skills/reviewing-due-songs/SKILL.md`

**Additional files modified:**
- `docs/init-db.sql` — Added `status` column to `play_history`
- `skills/reviewing-due-songs/SKILL.md` — Fixed rel_show_song lookup to use search

## Design Decisions

1. **Single `--term` JSON parameter** — Avoids proliferation of per-column CLI flags; self-documenting (column names are explicit keys); naturally supports multi-condition AND queries
2. **Default `exact` match** — Case-sensitive by default for ID columns and precise lookups; use `exact-i` for case-insensitive name searches
3. **`enumif` for searchable column validation** — Leverages JankenSQLHub's existing constraint system to whitelist searchable columns per table, preventing SQL injection
4. **`rel_show_song` as searchable table** — Enables checking show–song links without a dedicated command; `get` not supported since it has no `id` column
