# Add display_level Field to Level Responses

**Date:** 2026-03-17

## Summary

Added `display_level` field (`level + 1`, 1-indexed) to the JSON output of `learning-due` and `learning-by-song-ids` commands, making the stored vs. displayed level convention explicit for agents.

## Changes

### Code
- `src/commands/learning.rs`: Added `display_level` computation (stored level + 1) to both `cmd_learning_due` and `cmd_learning_by_song_ids` result rows

### Tests
- `tests/test_learning.rs`: Added `display_level` assertions across 6 tests
- `e2e/run_tests.sh`: Added `display_level` assertions for `learning-by-song-ids`

### Documentation
- `docs/cli-learning.md`: Added "Output fields" tables for `learning-due` and `learning-by-song-ids` with `level` (stored, 0-indexed) and `display_level` (1-indexed) descriptions; updated SQL example
- `docs/design/v1/concept.md`: Updated Level Display Convention to note API outputs now include both fields
- `.claude/skills/learning-with-jankenoboe/SKILL.md`: Updated output examples with `display_level`
- `.claude/skills/reviewing-due-songs/SKILL.md`: Updated output examples with `display_level`

## Status

✅ Complete — All 68 tests pass, clippy and fmt clean.
