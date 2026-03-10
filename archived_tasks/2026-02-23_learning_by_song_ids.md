# Task: Add `learning-by-song-ids` Command

**Date:** 2026-02-23
**Status:** ✅ Completed

## Summary

Created a new CLI command `learning-by-song-ids` that returns learning records for given song IDs, leveraging JankenSQLHub's `:[song_ids]` list parameter for a safe `IN` clause.

## CLI Usage

```bash
jankenoboe learning-by-song-ids --song-ids song-uuid-1,song-uuid-2
```

**Output:**
```json
{
  "count": 2,
  "results": [
    {
      "id": "learning-uuid-1",
      "song_id": "song-uuid-1",
      "song_name": "Crossing Field",
      "level": 10,
      "graduated": 0,
      "last_level_up_at": 1708900000,
      "wait_days": 7
    }
  ]
}
```

## Implementation Details

### JankenSQLHub Integration

Used `:[song_ids]` list parameter with `{"itemtype": "string"}` for the `IN` clause:

```json
{
  "learning_by_songs": {
    "query": "SELECT l.id, l.song_id, s.name as song_name, l.level, l.graduated, l.last_level_up_at, json_extract(l.level_up_path, '$[' || l.level || ']') as wait_days FROM learning l JOIN song s ON l.song_id = s.id WHERE l.song_id IN :[song_ids] ORDER BY l.level DESC",
    "returns": ["id", "song_id", "song_name", "level", "graduated", "last_level_up_at", "wait_days"],
    "args": {
      "song_ids": {"itemtype": "string"}
    }
  }
}
```

### Behavior

- Returns all learning records (active and graduated) for the given song IDs
- A single song may have multiple records (e.g., graduated + active re-learn)
- Results ordered by level descending
- Songs with no learning records are absent from results (no error)
- Empty `--song-ids` returns an error

## Files Changed

| File | Change |
|------|--------|
| `src/commands.rs` | Added `cmd_learning_by_song_ids()` function |
| `src/main.rs` | Added `LearningBySongIds` subcommand |
| `tests/test_learning.rs` | Added 7 tests |
| `docs/cli-learning.md` | Full command reference |
| `docs/cli.md` | Added to command table and operations coverage |
| `AGENTS.md` | Added to CLI commands list |
| `.clinerules` | Added to CLI commands list |
| `README.md` | Added usage example |
| `.claude/skills/querying-jankenoboe/SKILL.md` | Added section |
| `.claude/skills/learning-with-jankenoboe/SKILL.md` | Added section |

## Tests Added

7 new tests in `tests/test_learning.rs`:

1. `test_learning_by_song_ids_single` — Single song with all field assertions
2. `test_learning_by_song_ids_multiple_songs` — Multiple songs ordered by level DESC
3. `test_learning_by_song_ids_includes_graduated` — Graduated records included
4. `test_learning_by_song_ids_multiple_records_per_song` — Graduated + re-learn records
5. `test_learning_by_song_ids_no_learning_records` — Returns empty results
6. `test_learning_by_song_ids_empty` — Error on empty input
7. `test_learning_by_song_ids_nonexistent_song` — Nonexistent song returns empty

All 186 tests pass (38 unit + 148 integration).

### E2E Test

Added section 21 ("Learning by Song IDs") in `e2e/run_tests.sh`:
- Creates 2 songs, adds to learning, levels up one to level 3
- Queries via `learning-by-song-ids` with both song IDs
- Asserts count, level ordering (DESC), and song names
- Error case: empty `--song-ids` exits with code 1

E2E verified: all 164 e2e tests pass (via `make e2e` with Docker/Colima).
