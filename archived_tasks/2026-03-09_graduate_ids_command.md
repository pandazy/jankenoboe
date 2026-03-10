# Task: Add learning-song-graduate-ids Command

**Date:** 2026-03-09

## Summary

Added a new CLI command `learning-song-graduate-ids` that directly graduates specific learning records by setting level to max (19) and `graduated = 1` in a single atomic operation, regardless of current level.

## Changes

### Source Code
- **`src/commands/learning.rs`**: New `cmd_learning_song_graduate_ids` function
  - Validates IDs exist and aren't already graduated
  - Updates `level=19, graduated=1, last_level_up_at=now, updated_at=now`
  - All updates in a single transaction
- **`src/commands/mod.rs`**: Re-exported `cmd_learning_song_graduate_ids`
- **`src/main.rs`**: Added `LearningSongGraduateIds` subcommand variant and match arm

### Tests
- **`tests/test_learning.rs`**: 7 new tests:
  - `test_learning_song_graduate_ids_basic` — level 5 → graduated (level 19)
  - `test_learning_song_graduate_ids_from_level_zero` — level 0 → graduated
  - `test_learning_song_graduate_ids_from_max_level` — level 19 → graduated
  - `test_learning_song_graduate_ids_multiple` — batch graduate multiple records
  - `test_learning_song_graduate_ids_already_graduated` — error case
  - `test_learning_song_graduate_ids_not_found` — error case
  - `test_learning_song_graduate_ids_empty` — error case
- **`e2e/run_tests.sh`**: Added section 20 for graduate-ids e2e tests

### Documentation
- **`docs/cli-learning.md`**: Added full command reference section
- **`docs/cli.md`**: Added to command table and operations coverage
- **`AGENTS.md`**: Added to CLI commands table
- **`README.md`**: Added usage example
- **`.claude/skills/learning-with-jankenoboe/SKILL.md`**: Added "Directly Graduate by IDs" section

## Usage

```bash
jankenoboe learning-song-graduate-ids --ids learning-uuid-1,learning-uuid-2
```

**Output:**
```json
{
  "graduated_count": 2
}
```

## Error Cases
- Empty `--ids`: `{"error": "ids cannot be empty"}`
- ID not found: `{"error": "learning record(s) not found: <ids>"}`
- Already graduated: `{"error": "learning record already graduated: <id>"}`

## Status: ✅ Complete
