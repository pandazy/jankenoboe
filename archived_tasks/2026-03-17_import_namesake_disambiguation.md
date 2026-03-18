# Import Namesake Artist Disambiguation

**Date:** 2026-03-17

## Summary

Added interactive artist namesake disambiguation to the AMQ import script. When multiple artists share the same name during import, the script now pauses and prompts the user to choose.

## Changes

### `import_amq.py`
- Refactored `get_artist_id()` into `search_artists_by_name()` + `prompt_artist_disambiguation()`
- Added `get_songs_for_artist()` and `create_artist()` helper functions
- Single matches auto-resolve; multiple matches trigger interactive prompt
- Prompt shows each artist's ID and existing songs, plus context (song/show being resolved)
- Options: select existing artist, create NEW artist, or skip entry

### `SKILL.md`
- Updated Namesake Conflict Resolution section with new interactive behavior and example prompt

### `test_import_amq.py`
- Added namesake seed data (two "Minami" artists with distinct songs, Domestic Girlfriend show)
- Added Test 5 with three sub-tests:
  - 5a: Select an existing artist (stdin input "1")
  - 5b: Skip the entry (stdin input "4")
  - 5c: Create a new artist (stdin input "3")
- Updated seed count checks (6 artists, 4 shows, 4 songs)
- All 44/44 tests pass

### `docs/design/v1/import.md`
- Minor wording update for consistency with interactive prompt behavior

## Status: ✅ Complete
