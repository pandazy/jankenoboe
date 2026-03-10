# Task: songs-by-artist-ids command

**Date:** 2026-02-23
**Status:** ✅ Complete

## Summary

Created a new CLI command `jankenoboe songs-by-artist-ids` that returns all songs by given artists, traversing the `artist → song` relationship chain.

## Command

```bash
jankenoboe songs-by-artist-ids --artist-ids <comma-separated-artist-uuids>
```

**Output:** JSON with `count` and `results` array. Each result contains `song_id`, `song_name`, `artist_id`, `artist_name`. Ordered by artist name, then song name.

## Implementation Details

- Uses JankenSQLHub `:[artist_ids]` list parameter for safe IN-clause binding
- 2-table JOIN: `song → artist`
- Empty artist-ids returns error (exit code 1)
- Nonexistent artist IDs silently return zero results

## Files Modified

| File | Change |
|------|--------|
| `src/commands.rs` | Added `cmd_songs_by_artist_ids()` function |
| `src/main.rs` | Added `SongsByArtistIds` CLI subcommand variant and dispatch |
| `tests/test_querying.rs` | Added 7 unit tests |
| `e2e/run_tests.sh` | Added section 27 with 10 e2e assertions |
| `docs/cli-querying.md` | Full command documentation with query definition |
| `docs/cli.md` | Added to querying commands table |
| `README.md` | Added example to Querying section |
| `AGENTS.md` | Added to CLI commands table |
| `.clinerules` | Added to CLI commands list |
| `.claude/skills/querying-jankenoboe/SKILL.md` | Added "Songs by Artist IDs" section |

## Test Results

- **Unit tests:** 154/154 pass (7 new for this command)
- **E2E tests:** 184/184 pass (10 new for this command)

## Unit Test Coverage

- Single artist query (correct count, fields)
- Multiple artists query (correct count, both artist IDs present)
- Artist with no songs returns empty (count 0)
- Empty artist-ids returns error (exit code 1)
- Nonexistent artist returns empty (count 0)
- Correct fields returned (song_id, song_name, artist_id, artist_name)
- Ordering verification (by artist name, then song name)

## E2E Test Coverage

- Single artist query (correct count)
- Multiple artists query (correct count)
- Field validation (artist_name, song_name, song_id)
- Nonexistent artist returns empty (count 0)
- Empty artist-ids returns error (exit code 1)
