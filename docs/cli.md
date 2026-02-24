# CLI Reference

## Overview

Jankenoboe provides a set of CLI commands for interacting with the local SQLite database. All commands output JSON to stdout for easy piping and agent consumption. All timestamps are Unix timestamps (seconds since epoch).

The CLI uses [JankenSQLHub](https://github.com/pandazy/jankensqlhub)'s `#[table]`, `~[fields]`, `enum`, and `enumif` features to maintain full security through parameter validation.

**Database path:** Set the `JANKENOBOE_DB` environment variable to your SQLite database file path (e.g., `export JANKENOBOE_DB=~/db/datasource.db`).

## Tables

| Table | Description |
|-------|-------------|
| `artist` | Singers/bands |
| `show` | Anime series/movies |
| `song` | Theme songs |
| `play_history` | Records of song encounters in quizzes |
| `learning` | Spaced repetition tracking |
| `rel_show_song` | Many-to-many link between shows and songs |

---

## Commands by Category

### [Querying](cli-querying.md)

| Command | Description |
|---------|-------------|
| `jankenoboe get <table> <id>` | Get record by ID |
| `jankenoboe search <table>` | Search records with table-specific filters (exact or fuzzy match) |
| `jankenoboe duplicates <table>` | Find duplicate records by name |
| `jankenoboe shows-by-artist-ids` | Get all shows where given artists have song performances |
| `jankenoboe songs-by-artist-ids` | Get all songs by given artists |

### [Learning (Spaced Repetition)](cli-learning.md)

| Command | Description |
|---------|-------------|
| `jankenoboe learning-due` | Get songs due for review |
| `jankenoboe learning-batch` | Add one or many songs to learning |
| `jankenoboe learning-song-review` | Generate HTML report of due songs with enriched data |
| `jankenoboe learning-song-levelup-ids` | Level up specific learning records by their IDs |
| `jankenoboe learning-by-song-ids` | Get learning records by song IDs |

### [Data Management](cli-data-management.md)

| Command | Description |
|---------|-------------|
| `jankenoboe create <table>` | Create a new record |
| `jankenoboe update <table> <id>` | Update a record |
| `jankenoboe delete <table> <id>` | Delete a record |
| `jankenoboe bulk-reassign` | Reassign multiple songs to a new artist |

---

## Operations Coverage

This table shows how every required operation maps to the CLI commands.

### Import Workflow
| Operation | Command |
|-----------|---------|
| Find artist by name | `jankenoboe search artist --term '{"name":{"value":"X","match":"exact-i"}}' --fields id,name` |
| Find show by name + vintage | `jankenoboe search show --term '{"name":{"value":"X","match":"exact-i"},"vintage":{"value":"Y"}}' --fields id,name` |
| Find song by name + artist | `jankenoboe search song --term '{"name":{"value":"X","match":"exact-i"},"artist_id":{"value":"Y"}}' --fields id,name` |
| List songs by artist (disambiguation) | `jankenoboe search song --term '{"artist_id":{"value":"X"}}' --fields id,name` |
| Create artist | `jankenoboe create artist --data '{"name":"..."}'` |
| Create show | `jankenoboe create show --data '{"name":"...","vintage":"..."}'` |
| Create song | `jankenoboe create song --data '{"name":"...","artist_id":"..."}'` |
| Create play history | `jankenoboe create play_history --data '{"show_id":"...","song_id":"..."}'` |
| Check show–song link | `jankenoboe search rel_show_song --term '{"show_id":{"value":"X"},"song_id":{"value":"Y"}}' --fields show_id,song_id` |
| Link song to show | `jankenoboe create rel_show_song --data '{"show_id":"...","song_id":"..."}'` |

### Learning / Spaced Repetition
| Operation | Command |
|-----------|---------|
| Get songs due for review | `jankenoboe learning-due` |
| Create learning record(s) | `jankenoboe learning-batch --song-ids ...` |
| Level up | `jankenoboe update learning <id> --data '{"level": N}'` |
| Level down | `jankenoboe update learning <id> --data '{"level": N}'` |
| Graduate | `jankenoboe update learning <id> --data '{"graduated": 1}'` |
| Generate due songs HTML report | `jankenoboe learning-song-review` |
| Level up specific songs by ID | `jankenoboe learning-song-levelup-ids --ids ...` |
| Get learning records by song IDs | `jankenoboe learning-by-song-ids --song-ids ...` |

### Data Quality
| Operation | Command |
|-----------|---------|
| Find duplicate artists/shows/songs | `jankenoboe duplicates <table>` |
| Reassign single song | `jankenoboe update song <id> --data '{"artist_id":"..."}'` |
| Bulk reassign songs | `jankenoboe bulk-reassign --song-ids ... --new-artist-id ...` |
| Soft-delete artist | `jankenoboe update artist <id> --data '{"status": 1}'` |
| Hard-delete artist or song | `jankenoboe delete <table> <id>` |

### Fuzzy Search (--term)
| Operation | Command |
|-----------|---------|
| Search with term conditions | `jankenoboe search <table> --term '{"<col>":{"value":"...","match":"<mode>"}}' --fields ...` |
| Match modes | `exact` (case-sensitive), `exact-i` (case-insensitive), `starts-with`, `ends-with`, `contains` |
| Multiple AND conditions | `jankenoboe search show --term '{"name":{"value":"sign","match":"contains"},"vintage":{"value":"2024","match":"ends-with"}}' --fields ...` |

### General
| Operation | Command |
|-----------|---------|
| Read any record by ID | `jankenoboe get <table> <id> --fields ...` |
| Update any record | `jankenoboe update <table> <id> --data '{"field":"value"}'` |

---

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | Error (invalid parameters, validation failure, record not found, etc.) |

## Error Output Format

All errors are written to stderr as JSON:
```json
{
  "error": "Description of the error"
}