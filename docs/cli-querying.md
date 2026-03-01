# CLI Querying Commands

Commands for reading and searching data. See [CLI Reference](cli.md) for an overview of all commands.

> **Usage examples and workflows:** See [querying-jankenoboe skill](../.claude/skills/querying-jankenoboe/SKILL.md) for comprehensive examples including search patterns, match modes, and output formats.

---

## jankenoboe get \<table\> \<id\>

Retrieve a single record by its ID.

**Arguments:**
| Argument | Required | Description |
|----------|----------|-------------|
| `table` | Yes | Table name |
| `id` | Yes | Record UUID |

**Options:**
| Option | Required | Description |
|--------|----------|-------------|
| `--fields` | Yes | Comma-separated list of field names to return |

**Selectable fields per table:**
| Table | Fields |
|-------|--------|
| `artist` | `id`, `name`, `name_context`, `created_at`, `updated_at`, `status` |
| `show` | `id`, `name`, `name_romaji`, `vintage`, `s_type`, `created_at`, `updated_at`, `status` |
| `song` | `id`, `name`, `name_context`, `artist_id`, `created_at`, `updated_at`, `status` |
| `play_history` | `id`, `show_id`, `song_id`, `created_at`, `media_url`, `status` |
| `learning` | `id`, `song_id`, `level`, `created_at`, `updated_at`, `last_level_up_at`, `level_up_path`, `graduated` |

**JankenSQLHub Query Definition:**
```json
{
  "read_by_id": {
    "query": "SELECT ~[fields] FROM #[table] WHERE id=@id",
    "returns": "~[fields]",
    "args": {
      "table": {"enum": ["artist", "show", "song", "play_history", "learning"]},
      "fields": {
        "enumif": {
          "table": {
            "artist": ["id", "name", "name_context", "created_at", "updated_at", "status"],
            "show": ["id", "name", "name_romaji", "vintage", "s_type", "created_at", "updated_at", "status"],
            "song": ["id", "name", "name_context", "artist_id", "created_at", "updated_at", "status"],
            "play_history": ["id", "show_id", "song_id", "created_at", "media_url", "status"],
            "learning": ["id", "song_id", "level", "created_at", "updated_at", "last_level_up_at", "level_up_path", "graduated"]
          }
        }
      }
    }
  }
}
```

---

## jankenoboe search \<table\> --term

Search records using a structured `--term` JSON parameter. Each key in the term map is a column name, and its value specifies the search value and match mode. Multiple keys are combined with AND.

**Arguments:**
| Argument | Required | Description |
|----------|----------|-------------|
| `table` | Yes | Table name (`artist`, `show`, `song`, `play_history`, `rel_show_song`, or `learning`) |

**Options:**
| Option | Required | Description |
|--------|----------|-------------|
| `--fields` | Yes | Comma-separated list of field names to return |
| `--term` | Yes | JSON object mapping column names to `{value, match}` pairs |

**Term JSON format:**
```json
{
  "<column>": { "value": "<search_text>", "match": "<mode>" }
}
```

The `match` field is optional and defaults to `exact` (case-sensitive).

**URL Percent-Encoding:** The `value` field is automatically URL percent-decoded. Use `python3 tools/url_encode.py "<text>"` to encode values containing shell-problematic characters.

**Match modes:**
| Mode | Default | SQL Pattern | Description |
|------|---------|-------------|-------------|
| `exact` | ✅ | `= value` | Exact match (case-sensitive) |
| `exact-i` | | `LOWER() = LOWER(value)` | Exact match (case-insensitive) |
| `starts-with` | | `LIKE value%` | Column starts with value (case-insensitive) |
| `ends-with` | | `LIKE %value` | Column ends with value (case-insensitive) |
| `contains` | | `LIKE %value%` | Column contains value (case-insensitive) |

**Searchable columns per table:**
| Table | Columns |
|-------|---------|
| `artist` | `name`, `name_context` |
| `show` | `name`, `name_romaji`, `vintage` |
| `song` | `name`, `name_context`, `artist_id` |
| `play_history` | `show_id`, `song_id` |
| `rel_show_song` | `show_id`, `song_id` |

**Implementation:** The CLI validates column names against the searchable whitelist, dynamically builds the WHERE clause, and uses JankenSQLHub `#[table]`/`~[fields]` with `enumif` for field validation, preventing SQL injection via column names.

**Searchable column validation (JankenSQLHub enumif):**
```json
{
  "searchable_columns": {
    "enumif": {
      "table": {
        "artist": ["name", "name_context"],
        "show": ["name", "name_romaji", "vintage"],
        "song": ["name", "name_context", "artist_id"],
        "play_history": ["show_id", "song_id"],
        "rel_show_song": ["show_id", "song_id"]
      }
    }
  }
}
```

---

## jankenoboe duplicates \<table\>

Find records with case-insensitive matching names for data quality review.

**Arguments:**
| Argument | Required | Description |
|----------|----------|-------------|
| `table` | Yes | Table name (`artist`, `show`, or `song`) |

**Behavior:**
- For `artist` and `song` tables: includes a `song_count` subquery
- For `show` table: `song_count` is always `0`
- Only includes records with `status = 0` (non-deleted)
- Duplicates may be legitimate (e.g., two real artists with the same name)

---

## jankenoboe shows-by-artist-ids --artist-ids

Get all shows where the given artists have song performances. Traverses `artist → song → rel_show_song → show`.

**Options:**
| Option | Required | Description |
|--------|----------|-------------|
| `--artist-ids` | Yes | Comma-separated artist UUIDs |

**Returns:** `show_id`, `show_name`, `vintage`, `song_id`, `song_name`, `artist_id`, `artist_name`

**JankenSQLHub Query Definition:**
```json
{
  "shows_by_artists": {
    "query": "SELECT DISTINCT sh.id as show_id, sh.name as show_name, sh.vintage, s.id as song_id, s.name as song_name, a.id as artist_id, a.name as artist_name FROM show sh JOIN rel_show_song rs ON rs.show_id = sh.id JOIN song s ON rs.song_id = s.id JOIN artist a ON s.artist_id = a.id WHERE a.id IN :[artist_ids] ORDER BY a.name, sh.name, s.name",
    "returns": ["show_id", "show_name", "vintage", "song_id", "song_name", "artist_id", "artist_name"],
    "args": {
      "artist_ids": {"itemtype": "string"}
    }
  }
}
```

**Behavior:**
- One row per artist-show-song combination, ordered by artist name → show name → song name
- Artists with no linked shows return zero results
- Nonexistent artist IDs are silently ignored

---

## jankenoboe songs-by-artist-ids --artist-ids

Get all songs by the given artists. Traverses `artist → song`.

**Options:**
| Option | Required | Description |
|--------|----------|-------------|
| `--artist-ids` | Yes | Comma-separated artist UUIDs |

**Returns:** `song_id`, `song_name`, `artist_id`, `artist_name`

**JankenSQLHub Query Definition:**
```json
{
  "songs_by_artists": {
    "query": "SELECT s.id as song_id, s.name as song_name, a.id as artist_id, a.name as artist_name FROM song s JOIN artist a ON s.artist_id = a.id WHERE a.id IN :[artist_ids] ORDER BY a.name, s.name",
    "returns": ["song_id", "song_name", "artist_id", "artist_name"],
    "args": {
      "artist_ids": {"itemtype": "string"}
    }
  }
}
```

**Behavior:**
- One row per song with artist details, ordered by artist name → song name
- Artists with no songs return zero results
- Nonexistent artist IDs are silently ignored