# Import Skill Simplification

**Date:** 2026-03-17

## Goal

Simplify the importing-amq-songs skill to use only `import_amq.py` (remove the "Analyze and Verify" step) and improve the missing data feedback with actionable CLI commands including `name_romaji` for shows.

## Changes

### SKILL.md Simplified
- Removed "Step 1: Analyze and Verify" section (references to `parse_amq_import.py`, `check_artists.sh`, `check_shows.sh`)
- Removed verbose "Option B: Manual Import" flow with all its sub-steps
- Single workflow: run `import_amq.py`, use the missing report to create entities, re-run with `--missing-only`
- Document went from ~180 lines to ~90 lines

### import_amq.py — Actionable Missing Report
- Added `_build_create_cmd()` helper to generate ready-to-run `jankenoboe create` commands
- Missing artists: shows `jankenoboe create artist --data '{"name": "..."}'`
- Missing shows: shows `jankenoboe create show --data '{"name": "...", "name_romaji": "...", "vintage": "...", "s_type": "..."}'` — always includes `name_romaji` from AMQ export
- Missing songs with resolved `artist_id`: shows complete create command; unresolved: flagged with "(create artist first)"
- Added `romaji_name` and `s_type` fields to resolved entries for use in report
- Deduplicated missing entities section now uses dict (keyed by name/composite key) instead of set, to preserve entry data for command generation

### Deleted Unused Scripts
- `.claude/skills/importing-amq-songs/scripts/parse_amq_import.py`
- `.claude/skills/importing-amq-songs/scripts/check_artists.sh`
- `.claude/skills/importing-amq-songs/scripts/check_shows.sh`

### Tests Updated
- Added 3 new assertions in `test_import_amq.py`:
  - CLI command for creating ChoQMay artist present
  - CLI command for creating Kashimashi show includes `name_romaji`
  - CLI command for creating show includes vintage and s_type
- All 32/32 tests pass

## Files Modified
- `.claude/skills/importing-amq-songs/SKILL.md` — simplified
- `.claude/skills/importing-amq-songs/scripts/import_amq.py` — actionable missing report
- `.claude/skills/importing-amq-songs/scripts/test_import_amq.py` — new assertions

### Replaced `tools/url_encode.py` with Inline Python

The `url_encode.py` script could get stuck in Python's REPL mode when called by agents. Replaced all references across the project with inline Python one-liners:

```bash
python3 -c "from urllib.parse import quote; print(quote('<text>', safe=''))"
```

Updated files:
- `.claude/skills/importing-amq-songs/SKILL.md`
- `.claude/skills/maintaining-jankenoboe-data/SKILL.md`
- `.claude/skills/querying-jankenoboe/SKILL.md`
- `docs/cli-querying.md`
- `docs/cli-data-management.md`
- `README.md`
- `AGENTS.md`
- `docs/design/v1/structure.md`

## Files Deleted
- `.claude/skills/importing-amq-songs/scripts/parse_amq_import.py`
- `.claude/skills/importing-amq-songs/scripts/check_artists.sh`
- `.claude/skills/importing-amq-songs/scripts/check_shows.sh`
- `tools/url_encode.py`
