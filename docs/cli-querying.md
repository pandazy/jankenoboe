# CLI Querying Commands

Commands for reading and searching data. See [CLI Reference](cli.md) for an overview of all commands.

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

**Example:**
```bash
jankenoboe get song 3b105bd4-c437-4720-a373-660bd5d68532 --fields id,name,artist_id
```

**Output:**
```json
{
  "results": [
    {
      "id": "3b105bd4-c437-4720-a373-660bd5d68532",
      "name": "Fuwa Fuwa Time (5-nin Ver.)",
      "artist_id": "2196b222-ed04-4260-90c8-d18382bf8900"
    }
  ]
}
```

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
| `table` | Yes | Table name (`artist`, `show`, `song`, `play_history`, or `rel_show_song`) |

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

The `match` field is optional and defaults to `exact` (case-sensitive). When only `value` is needed with the default match, you can omit `match`:
```json
{
  "<column>": { "value": "<search_text>" }
}
```

**URL Percent-Encoding:** The `value` field is automatically URL percent-decoded before use. This avoids shell quoting issues with special characters like quotes, spaces, and parentheses. Use Python's `urllib.parse.quote(text, safe="")` or the included helper:

```bash
# Encode a value
python3 tools/url_encode.py "it's a test"
# Output: it%27s%20a%20test

# Use in search
jankenoboe search artist --fields id,name --term '{"name":{"value":"it%27s%20a%20test"}}'
```

Plain text values (without `%` sequences) work unchanged — encoding is only needed for values containing shell-problematic characters like `'`, `"`, `(`, `)`, `&`, `!`, spaces, etc.

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
| `show` | `name`, `vintage` |
| `song` | `name`, `name_context`, `artist_id` |
| `play_history` | `show_id`, `song_id` |
| `rel_show_song` | `show_id`, `song_id` |

**Examples:**
```bash
# Find artist by exact name (case-sensitive, default match mode)
jankenoboe search artist --fields id,name --term '{"name": {"value": "ChoQMay"}}'

# Find artist by name (case-insensitive)
jankenoboe search artist --fields id,name --term '{"name": {"value": "minami", "match": "exact-i"}}'

# Find show by name + vintage (both exact)
jankenoboe search show --fields id,name,vintage --term '{"name": {"value": "K-On!"}, "vintage": {"value": "Spring 2009"}}'

# Find song by name + artist_id
jankenoboe search song --fields id,name,artist_id --term '{"name": {"value": "snowspring", "match": "exact-i"}, "artist_id": {"value": "abc123"}}'

# List all songs by artist
jankenoboe search song --fields id,name,artist_id --term '{"artist_id": {"value": "abc123"}}'

# Check if show and song are linked
jankenoboe search rel_show_song --fields show_id,song_id,media_url --term '{"show_id": {"value": "<show-uuid>"}, "song_id": {"value": "<song-uuid>"}}'

# Find play history records by song
jankenoboe search play_history --fields id,show_id,song_id,media_url --term '{"song_id": {"value": "<song-uuid>"}}'

# Find play history records by show and song
jankenoboe search play_history --fields id,media_url --term '{"show_id": {"value": "<show-uuid>"}, "song_id": {"value": "<song-uuid>"}}'

# Find artists whose name starts with "min"
jankenoboe search artist --fields id,name --term '{"name": {"value": "min", "match": "starts-with"}}'

# Find shows from 2024 with "sign" in the name (multiple AND conditions)
jankenoboe search show --fields id,name,vintage --term '{"name": {"value": "sign", "match": "contains"}, "vintage": {"value": "2024", "match": "ends-with"}}'

# Find songs whose name contains "love"
jankenoboe search song --fields id,name,artist_id --term '{"name": {"value": "love", "match": "contains"}}'
```

**Implementation:**

The CLI parses the `--term` JSON, validates column names against the table's allowed searchable columns, and dynamically builds the WHERE clause. Fuzzy modes (`starts-with`, `ends-with`, `contains`) and `exact-i` use `LOWER()` for case-insensitive matching; `exact` is case-sensitive. The query is executed using JankenSQLHub with `#[table]` and `~[fields]` with `enumif` for field validation.

**Searchable column validation** uses the same `enumif` pattern as other queries to ensure only allowed columns can be searched per table, preventing SQL injection via column names.

```json
{
  "searchable_columns": {
    "enumif": {
      "table": {
        "artist": ["name", "name_context"],
        "show": ["name", "vintage"],
        "song": ["name", "name_context", "artist_id"],
        "play_history": ["show_id", "song_id"],
        "rel_show_song": ["show_id", "song_id"]
      }
    }
  }
}
```

**Output (all search commands):**
```json
{
  "results": [
    {"id": "...", "name": "snowspring", "artist_id": "..."}
  ]
}
```

---

## jankenoboe duplicates \<table\>

Find records with case-insensitive matching names for data quality review.

**Arguments:**
| Argument | Required | Description |
|----------|----------|-------------|
| `table` | Yes | Table name (`artist`, `show`, or `song`) |

**Example:**
```bash
jankenoboe duplicates artist
```

**Output:**
```json
{
  "duplicates": [
    {
      "name": "minami",
      "records": [
        {"id": "14b7393a-...", "name": "Minami", "song_count": 5},
        {"id": "6136d7b3-...", "name": "Minami", "song_count": 3}
      ]
    }
  ]
}
```

**Note:** Duplicates may be legitimate (e.g., two real artists with the same name). This command surfaces them for human review.