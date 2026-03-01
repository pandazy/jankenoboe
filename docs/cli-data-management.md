# CLI Data Management Commands

Commands for creating, updating, deleting records and bulk operations. See [CLI Reference](cli.md) for an overview of all commands.

> **Usage examples and workflows:** See [maintaining-jankenoboe-data skill](../.claude/skills/maintaining-jankenoboe-data/SKILL.md) for comprehensive examples, merge workflows, and output formats.

---

## jankenoboe create \<table\>

Create a new record in the specified table. The CLI generates the `id` (UUID) and timestamps (`created_at`, `updated_at`).

**Arguments:**
| Argument | Required | Description |
|----------|----------|-------------|
| `table` | Yes | Table name (`artist`, `show`, `song`, `play_history`, `learning`, `rel_show_song`) |

**Options:**
| Option | Required | Description |
|--------|----------|-------------|
| `--data` | Yes | JSON object with field values. String values are URL percent-decoded. |

**URL Percent-Encoding:** All string values in `--data` are automatically URL percent-decoded before use. Use `python3 tools/url_encode.py "<text>"` to encode values containing shell-problematic characters. Keys and non-string values (numbers, booleans) are not decoded.

**Creatable fields per table:**
| Table | Fields |
|-------|--------|
| `artist` | `name`, `name_context` |
| `show` | `name`, `name_romaji`, `vintage`, `s_type` |
| `song` | `name`, `name_context`, `artist_id` |
| `play_history` | `show_id`, `song_id`, `media_url` |
| `learning` | `song_id`, `level_up_path` |
| `rel_show_song` | `show_id`, `song_id`, `media_url` |

**Output:**
```json
{
  "id": "generated-uuid"
}
```

---

## jankenoboe update \<table\> \<id\>

Update specific fields of a record. Only the provided fields are updated; `updated_at` is set to the current timestamp automatically.

**Arguments:**
| Argument | Required | Description |
|----------|----------|-------------|
| `table` | Yes | Table name (`artist`, `show`, `song`, `play_history`, `learning`) |
| `id` | Yes | Record UUID |

**Options:**
| Option | Required | Description |
|--------|----------|-------------|
| `--data` | Yes | JSON object with fields to update. String values are URL percent-decoded. |

**Updatable fields per table:**
| Table | Fields |
|-------|--------|
| `artist` | `name`, `name_context`, `status` |
| `show` | `name`, `name_romaji`, `vintage`, `s_type`, `status` |
| `song` | `name`, `name_context`, `artist_id`, `status` |
| `play_history` | `show_id`, `song_id`, `media_url`, `status` |
| `learning` | `level`, `graduated` |

**Behavior Notes:**
- When `level` is changed on a learning record, `last_level_up_at` is also updated to the current timestamp
- The `updated_at` field is always set to the current timestamp

**Output:**
```json
{
  "updated": true
}
```

---

## jankenoboe delete \<table\> \<id\>

Hard delete a record from the database.

**Arguments:**
| Argument | Required | Description |
|----------|----------|-------------|
| `table` | Yes | Table name (`artist` or `song` only) |
| `id` | Yes | Record UUID |

**Output:**
```json
{
  "deleted": true
}
```

---

## jankenoboe bulk-reassign

Reassign multiple songs to a different artist atomically. Two modes:

### By Song IDs

| Option | Required | Description |
|--------|----------|-------------|
| `--song-ids` | Yes | Comma-separated song UUIDs |
| `--new-artist-id` | Yes | Target artist UUID |

### By Source Artist

| Option | Required | Description |
|--------|----------|-------------|
| `--from-artist-id` | Yes | Source artist UUID (move all songs from) |
| `--to-artist-id` | Yes | Target artist UUID (move all songs to) |

**Output:**
```json
{
  "reassigned_count": 3
}