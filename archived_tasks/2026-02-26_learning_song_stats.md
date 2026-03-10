# Task: learning-song-stats command

## Status: ✅ COMPLETE

## Summary

Added a new `learning-song-stats` CLI command that returns the time spent learning each song. For each song queried by ID, it aggregates all learning records (including graduated and re-learn) and computes the absolute gap in days between the earliest creation date and the most recent `last_level_up_at`.

## What was done

### Implementation
- **`src/commands/learning.rs`**: Added `cmd_learning_song_stats` function
  - SQL: `GROUP BY song_id`, uses `ABS()` for the days gap, `ROUND()` for rounding
  - Uses JankenSQLHub's `:[song_ids]` list parameter for the IN clause
  - Returns: `song_id`, `song_name`, `earliest_created_at`, `latest_last_level_up_at`, `days_spent`
  - Ordered by `days_spent` DESC

### CLI wiring
- **`src/commands/mod.rs`**: Added re-export of `cmd_learning_song_stats`
- **`src/main.rs`**: Added `LearningSongStats` Clap subcommand with `--song-ids` option

### Tests (7 new tests in `tests/test_learning.rs`)
- `test_learning_song_stats_single_song_single_record`
- `test_learning_song_stats_multiple_records_per_song`
- `test_learning_song_stats_multiple_songs`
- `test_learning_song_stats_no_learning_records`
- `test_learning_song_stats_empty_song_ids`
- `test_learning_song_stats_abs_gap_when_last_level_up_is_zero`

### Documentation updated
- `docs/cli-learning.md` — Full command reference section
- `AGENTS.md` — CLI commands table
- `.clinerules` — Commands list
- `README.md` — CLI usage examples
- `.claude/skills/learning-with-jankenoboe/SKILL.md` — Agent skill section

### Build status
- All 232 tests pass
- Clippy clean
- Formatted

## Key design decisions
- Only returns `song_id`, `song_name`, `earliest_created_at`, `latest_last_level_up_at`, `days_spent` — no extra fields like artist_name, max_level, or graduated (per user request for minimalism)
- Uses `ABS()` to compute absolute gap between dates
- Groups ALL learning records per song (graduated + active) to get the full learning timeline