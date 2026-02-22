# CLI Data Management Commands

Commands for creating, updating, deleting records and bulk operations. See [CLI Reference](cli.md) for an overview of all commands.

---

## jankenoboe create \<table\>

Create a new record in the specified table. The CLI generates the `id` (UUID) and timestamps (`created_at`, `updated_at`).

**Arguments:**
| Argument | Required | Description |
|----------|----------|-------------|
| `table` | Yes | Table name |

**Options:**
| Option | Required | Description |
|--------|----------|-------------|
| `--data` | Yes | JSON object with field values. String values are URL percent-decoded. |

**URL Percent-Encoding:** All string values in the `--data` JSON are automatically URL percent-decoded before use. This avoids shell quoting issues with special characters. Use `python3 tools/url_encode.py "<text>"` to encode values. Plain text (without `%` sequences) works unchanged. Keys and non-string values (numbers, booleans) are not decoded.

```bash
# Create artist with a name containing a single quote
jankenoboe create artist --data '{"name":"Ado%27s%20Music"}'
# Stored as: Ado's Music
```

### Artist
```bash
jankenoboe create artist --data '{"name": "ChoQMay"}'
```

### Show
```bash
jankenoboe create show --data '{"name": "A Sign of Affection", "name_romaji": "Yubisaki to Renren", "vintage": "Winter 2024", "s_type": "TV"}'
```

### Song
```bash
jankenoboe create song --data '{"name": "snowspring", "artist_id": "artist-uuid"}'
```

### Play History
```bash
jankenoboe create play_history --data '{"show_id": "show-uuid", "song_id": "song-uuid", "media_url": "https://..."}'
```

### Learning
```bash
jankenoboe create learning --data '{"song_id": "song-uuid", "level_up_path": "[1,1,1,1,1,1,1,2,3,5,7,13,19,32,52,84,135,220,355,574]"}'
```

### Rel Show Song
```bash
jankenoboe create rel_show_song --data '{"show_id": "show-uuid", "song_id": "song-uuid", "media_url": "https://..."}'
```

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
| `table` | Yes | Table name |
| `id` | Yes | Record UUID |

**Options:**
| Option | Required | Description |
|--------|----------|-------------|
| `--data` | Yes | JSON object with fields to update. String values are URL percent-decoded. |

### Update Song (e.g., reassign artist)
```bash
jankenoboe update song abc123 --data '{"artist_id": "new-artist-id"}'
```

### Update Learning - Level Up
After a successful review, increment the level. The `last_level_up_at` is updated to the current timestamp.
```bash
jankenoboe update learning def456 --data '{"level": 8}'
```

### Update Learning - Level Down
If the song is forgotten, set a lower level. The `last_level_up_at` is also updated to the current timestamp.
```bash
jankenoboe update learning def456 --data '{"level": 3}'
```

### Update Learning - Graduate
When a song reaches the end of its `level_up_path` and is fully memorized:
```bash
jankenoboe update learning def456 --data '{"graduated": 1}'
```

### Update Artist (e.g., soft delete)
```bash
jankenoboe update artist abc123 --data '{"status": 1}'
```

**Behavior Notes:**
- When `level` is changed on a learning record (up or down), `last_level_up_at` is also updated to the current timestamp
- The `updated_at` field is always set to the current timestamp on any update

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

**Example:**
```bash
jankenoboe delete artist abc123
```

**Output:**
```json
{
  "deleted": true
}
```

---

## jankenoboe bulk-reassign

Reassign multiple songs to a different artist atomically. Used for fixing import mistakes or merging duplicate artists.

There are two modes: reassign by specific song IDs, or reassign all songs from one artist to another.

### By Song IDs

**Options:**
| Option | Required | Description |
|--------|----------|-------------|
| `--song-ids` | Yes | Comma-separated song UUIDs |
| `--new-artist-id` | Yes | Target artist UUID |

```bash
jankenoboe bulk-reassign --song-ids song1,song2,song3 --new-artist-id correct-artist-id
```

### By Source Artist

Move all songs from one artist to another:

**Options:**
| Option | Required | Description |
|--------|----------|-------------|
| `--from-artist-id` | Yes | Source artist UUID (move all songs from) |
| `--to-artist-id` | Yes | Target artist UUID (move all songs to) |

```bash
jankenoboe bulk-reassign --from-artist-id artist-to-remove --to-artist-id artist-to-keep
```

**Output:**
```json
{
  "reassigned_count": 3
}