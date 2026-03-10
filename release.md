## v2.6.0

### New: `batch-get` command

Batch version of `get` — retrieve multiple records by IDs in a single call. Accepts `--ids` (comma-separated UUIDs) and `--fields`, returns `{"count": N, "results": [...]}`. Nonexistent IDs are silently ignored.

```bash
jankenoboe batch-get artist --ids uuid-1,uuid-2 --fields id,name
```

### New: `learning-song-graduate-ids` command

Directly graduate specific learning records by their IDs. Sets level to max (19) and `graduated = 1` in a single atomic transaction, regardless of current level. Validates that records exist and aren't already graduated.

```bash
jankenoboe learning-song-graduate-ids --ids learning-uuid-1,learning-uuid-2
```

### Improved: Import missing report with resolved IDs

The AMQ import script's Missing Entities Report now groups entries by their missing pattern and includes already-resolved IDs. For example, if only the song is missing, the report shows the resolved `artist_id` and `show_id` — follow-up procedures can directly create the song and link it without re-fetching.

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/install.sh | sh
```

## Platform Support

Pre-built binaries for:
- Linux x86_64
- Linux ARM64 (aarch64)
- macOS Apple Silicon (aarch64)
