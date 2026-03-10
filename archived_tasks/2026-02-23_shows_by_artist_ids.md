# Task: shows-by-artist-ids command

**Date:** 2026-02-23
**Status:** ✅ Complete

## Summary

Created a new CLI command `jankenoboe shows-by-artist-ids` that returns all shows where given artists have song performances, traversing the `artist → song → rel_show_song → show` relationship chain.

## Command

```bash
jankenoboe shows-by-artist-ids --artist-ids <comma-separated-artist-uuids>
```

**Output:** JSON with `count` and `results` array. Each result contains `show_id`, `show_name`, `vintage`, `song_id`, `song_name`, `artist_id`, `artist_name`. Ordered by artist name, show name, song name.

## Implementation Details

- Uses JankenSQLHub `:[artist_ids]` list parameter for safe IN-clause binding
- 4-table JOIN: `show → rel_show_song → song → artist`
- SELECT DISTINCT to avoid duplicates
- Empty artist-ids returns error (exit code 1)
- Nonexistent artist IDs silently return zero results

## Files Modified

| File | Change |
|------|--------|
| `src/commands.rs` | Added `cmd_shows_by_artist_ids()` function |
| `src/main.rs` | Added `ShowsByArtistIds` CLI subcommand variant and dispatch |
| `tests/test_querying.rs` | Added 7 unit tests |
| `e2e/run_tests.sh` | Added section 26 with 10 e2e assertions |
| `docs/cli-querying.md` | Full command documentation with query definition |
| `docs/cli.md` | Added to querying commands table |
| `README.md` | Added example to Querying section |
| `AGENTS.md` | Added to CLI commands table |
| `.clinerules` | Added to CLI commands list |
| `.claude/skills/querying-jankenoboe/SKILL.md` | Added "Shows by Artist IDs" section |

## Test Results

- **Unit tests:** 147/147 pass
- **E2E tests:** 174/174 pass (10 new for this command)

## E2E Test Coverage

- Single artist query (correct count)
- Multiple artists query (correct count)
- Field validation (show_name, artist_name, vintage)
- Nonexistent artist returns empty (count 0)
- Empty artist-ids returns error (exit code 1)