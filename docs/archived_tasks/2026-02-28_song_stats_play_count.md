# Add Play History Count to Song Stats

**Date:** 2026-02-28

## Summary

Added `play_count` field to `learning-song-stats` output, showing the total number of play_history records for each song.

## Changes

### Query (`src/commands/learning.rs`)
- Added correlated subquery `(SELECT COUNT(*) FROM play_history ph WHERE ph.song_id = l.song_id) AS play_count` to the `learning_song_stats` SQL query
- Added `play_count` to the `returns` array in the query definition

### Tests (`tests/test_learning.rs`)
- Updated 5 existing stats tests to assert `play_count` = 0 (no play history)
- Added `test_learning_song_stats_with_play_history`: creates 3 play_history records and verifies `play_count` = 3

### Documentation
- `docs/cli-learning.md`: Added `play_count` field to the stats output fields table

## Files Modified
- `src/commands/learning.rs`
- `tests/test_learning.rs`
- `docs/cli-learning.md`