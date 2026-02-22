---
name: querying-jankenoboe
description: Search and read anime song learning data from a Jankenoboe SQLite database. Finds artists, shows, songs, learning records, and play history. Use when the user asks to look up, search, find, or list anime songs, artists, shows, or learning progress.
---

## Setup

The `jankenoboe` CLI must be installed. Set the `JANKENOBOE_DB` environment variable to the SQLite database path:
```bash
export JANKENOBOE_DB=~/db/datasource.db
```

**Always ask the user for the database path first** if `JANKENOBOE_DB` is not already set.

## Tables

| Table | Key Fields |
|-------|-----------|
| artist | id, name, name_context, status |
| show | id, name, name_romaji, vintage, s_type, status |
| song | id, name, name_context, artist_id, status |
| learning | id, song_id, level, graduated, level_up_path, last_level_up_at |
| play_history | id, show_id, song_id, media_url |
| rel_show_song | show_id, song_id, media_url |

---

## Get by ID

Retrieve a single record by its UUID.

```bash
jankenoboe get <table> <id> --fields <comma-separated-fields>
```

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

### Available fields per table

| Table | Fields |
|-------|--------|
| artist | id, name, name_context, created_at, updated_at, status |
| show | id, name, name_romaji, vintage, s_type, created_at, updated_at, status |
| song | id, name, name_context, artist_id, created_at, updated_at, status |
| play_history | id, show_id, song_id, created_at, media_url, status |
| learning | id, song_id, level, created_at, updated_at, last_level_up_at, level_up_path, graduated |

---

## Search (--term)

All searches use the `--term` JSON parameter. Each key is a column name with `{value, match}`. Multiple keys are combined with AND. The `match` field defaults to `exact` (case-sensitive) when omitted.

**Match modes:** `exact` (default, case-sensitive), `exact-i` (case-insensitive), `starts-with`, `ends-with`, `contains`

**URL Percent-Encoding:** The `value` field is automatically URL percent-decoded. Use `python3 tools/url_encode.py "<text>"` to encode values containing quotes, spaces, or other shell-problematic characters (e.g., `it%27s` → `it's`, `%20` → space). Plain text without `%` works unchanged.

### Exact searches

```bash
# Find artist by name (case-insensitive)
jankenoboe search artist --fields id,name --term '{"name": {"value": "minami", "match": "exact-i"}}'

# Find show by name + vintage
jankenoboe search show --fields id,name,vintage --term '{"name": {"value": "K-On!", "match": "exact-i"}, "vintage": {"value": "Spring 2009"}}'

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
```

### Fuzzy searches

```bash
# Artists whose name starts with "min"
jankenoboe search artist --fields id,name --term '{"name": {"value": "min", "match": "starts-with"}}'

# Shows from 2024 with "sign" in the name (AND)
jankenoboe search show --fields id,name,vintage --term '{"name": {"value": "sign", "match": "contains"}, "vintage": {"value": "2024", "match": "ends-with"}}'

# Songs whose name contains "love"
jankenoboe search song --fields id,name,artist_id --term '{"name": {"value": "love", "match": "contains"}}'
```

**Searchable columns per table:**

| Table | Columns |
|-------|---------|
| artist | name, name_context |
| show | name, vintage |
| song | name, name_context, artist_id |
| play_history | show_id, song_id |
| rel_show_song | show_id, song_id |

**Output (all search commands):**
```json
{
  "results": [
    {"id": "...", "name": "...", ...}
  ]
}
```

---

## Learning Due for Review

```bash
jankenoboe learning-due
jankenoboe learning-due --limit 20
```

**Output:**
```json
{
  "count": 13,
  "results": [
    {
      "id": "...",
      "song_id": "...",
      "song_name": "1-nichi wa 25-jikan.",
      "level": 16,
      "wait_days": 135
    }
  ]
}
```

---

## Find Duplicates

```bash
jankenoboe duplicates <table>
```

Allowed tables: `artist`, `show`, `song`

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

Duplicates may be legitimate (e.g., two real artists with the same name). This command surfaces them for human review.

---

## Response Format

All CLI output is JSON to stdout:
- Reads/searches: `{"results": [...]}`
- Due items: `{"count": N, "results": [...]}`
- Duplicates: `{"duplicates": [...]}`
- Errors: `{"error": "..."}` on stderr, exit code 1