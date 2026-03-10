# Import Romaji Name Backfill for Shows

**Date:** 2026-02-28

## Summary

Updated the AMQ import workflow to handle romaji names (`name_romaji`) for shows — both backfilling existing shows with empty romaji and ensuring new shows always include romaji when available from the import JSON.

## Changes

### Import Script (`import_amq.py`)
- Renamed `get_show_id()` → `get_show()` — now returns the full show record (including `name_romaji`) instead of just the ID
- Added `update_show_romaji(show_id, romaji_name)` helper function
- Updated `resolve_entry()` to check if an existing show has an empty `name_romaji`; if the import JSON provides one, it's automatically filled
- Added `romaji_updated` flag to resolved entries for logging
- Import loop now logs "(filled romaji name)" when a backfill occurs

### Documentation
- `docs/import.md`: Added "Romaji backfill" bullet explaining that the import script auto-fills missing romaji names
- `.claude/skills/importing-amq-songs/SKILL.md`: 
  - "Not found" section: Added ⚠️ warning to always include `name_romaji` when creating new shows
  - "Found" section: Added instructions to check and backfill empty `name_romaji` manually, with note that automated script handles this

## Files Modified
- `.claude/skills/importing-amq-songs/scripts/import_amq.py`
- `.claude/skills/importing-amq-songs/SKILL.md`
- `docs/import.md`