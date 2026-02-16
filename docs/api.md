# API Reference

## Overview

Jankenoboe is a REST API service running on `http://localhost:3000`. All timestamps are Unix timestamps (seconds since epoch).

The API is designed around **generic CRUD endpoints** that leverage [JankenSQLHub](https://github.com/pandazy/jankensqlhub)'s `#[table]`, `~[fields]`, `enum`, and `enumif` features to minimize the number of endpoints while maintaining full security through parameter validation.

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

## Endpoints Summary

| # | Method | Endpoint | Description |
|---|--------|----------|-------------|
| 1 | GET | `/:table/:id` | Get record by ID |
| 2 | GET | `/:table/search` | Search records with table-specific filters |
| 3 | POST | `/:table` | Create a new record |
| 4 | PATCH | `/:table/:id` | Update a record |
| 5 | DELETE | `/:table/:id` | Delete a record |
| 6 | GET | `/learning/due` | Get songs due for review |
| 7 | GET | `/:table/duplicates` | Find duplicate records by name |
| 8 | POST | `/song/bulk-reassign` | Reassign multiple songs to a new artist |
| 9 | POST | `/learning/batch` | Add one or many songs to learning |

---

## 1. GET /:table/:id

Retrieve a single record by its ID.

**Query Parameters:**
| Parameter | Required | Description |
|-----------|----------|-------------|
| `fields` | Yes | Comma-separated list of field names to return |

**Example:**
```bash
curl "http://localhost:3000/song/3b105bd4-c437-4720-a373-660bd5d68532?fields=id,name,artist_id"
```

**Response:**
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
      "table": {"enum": ["artist", "show", "song", "play_history", "learning", "rel_show_song"]},
      "fields": {
        "enumif": {
          "table": {
            "artist": ["id", "name", "name_context", "created_at", "updated_at", "status"],
            "show": ["id", "name", "name_romaji", "vintage", "s_type", "created_at", "updated_at", "status"],
            "song": ["id", "name", "name_context", "artist_id", "created_at", "updated_at", "status"],
            "play_history": ["id", "show_id", "song_id", "created_at", "media_url", "status"],
            "learning": ["id", "song_id", "level", "created_at", "updated_at", "last_level_up_at", "level_up_path", "graduated"],
            "rel_show_song": ["show_id", "song_id", "media_url", "created_at"]
          }
        }
      }
    }
  }
}
```

---

## 2. GET /:table/search

Search records using table-specific filters. The handler selects the appropriate query definition based on the table and provided query parameters.

### Artist Search

**Find by name (case-insensitive):**
```bash
curl "http://localhost:3000/artist/search?fields=id,name&name=minami"
```

**Query Definition:**
```json
{
  "search_artist_by_name": {
    "query": "SELECT ~[fields] FROM artist WHERE LOWER(name) = LOWER(@name)",
    "returns": "~[fields]",
    "args": {
      "fields": {"enum": ["id", "name", "name_context", "created_at", "updated_at", "status"]},
      "name": {"range": [1, 800]}
    }
  }
}
```

### Show Search

**Find by name and vintage:**
```bash
curl "http://localhost:3000/show/search?fields=id,name,vintage&name=K-On!&vintage=Spring%202009"
```

**Query Definition:**
```json
{
  "search_show_by_name_vintage": {
    "query": "SELECT ~[fields] FROM show WHERE LOWER(name) = LOWER(@name) AND vintage=@vintage",
    "returns": "~[fields]",
    "args": {
      "fields": {"enum": ["id", "name", "name_romaji", "vintage", "s_type", "created_at", "updated_at", "status"]},
      "name": {"range": [1, 300]},
      "vintage": {"range": [1, 50]}
    }
  }
}
```

### Song Search

**Find by name and artist_id:**
```bash
curl "http://localhost:3000/song/search?fields=id,name,artist_id&name=snowspring&artist_id=abc123"
```

**List all songs by artist:**
```bash
curl "http://localhost:3000/song/search?fields=id,name,artist_id&artist_id=abc123"
```

**Query Definitions:**
```json
{
  "search_song_by_name_artist": {
    "query": "SELECT ~[fields] FROM song WHERE LOWER(name) = LOWER(@name) AND artist_id=@artist_id",
    "returns": "~[fields]",
    "args": {
      "fields": {"enum": ["id", "name", "name_context", "artist_id", "created_at", "updated_at", "status"]},
      "name": {"range": [1, 300]}
    }
  },
  "search_song_by_artist": {
    "query": "SELECT ~[fields] FROM song WHERE artist_id=@artist_id",
    "returns": "~[fields]",
    "args": {
      "fields": {"enum": ["id", "name", "name_context", "artist_id", "created_at", "updated_at", "status"]}
    }
  }
}
```

**Response (all search endpoints):**
```json
{
  "results": [
    {"id": "...", "name": "snowspring", "artist_id": "..."}
  ]
}
```

---

## 3. POST /:table

Create a new record in the specified table. The server generates the `id` (UUID) and timestamps (`created_at`, `updated_at`).

### Artist
```bash
curl -X POST "http://localhost:3000/artist" \
  -H "Content-Type: application/json" \
  -d '{"name": "ChoQMay"}'
```

### Show
```bash
curl -X POST "http://localhost:3000/show" \
  -H "Content-Type: application/json" \
  -d '{"name": "A Sign of Affection", "name_romaji": "Yubisaki to Renren", "vintage": "Winter 2024", "s_type": "TV"}'
```

### Song
```bash
curl -X POST "http://localhost:3000/song" \
  -H "Content-Type: application/json" \
  -d '{"name": "snowspring", "artist_id": "artist-uuid"}'
```

### Play History
```bash
curl -X POST "http://localhost:3000/play_history" \
  -H "Content-Type: application/json" \
  -d '{"show_id": "show-uuid", "song_id": "song-uuid", "media_url": "https://..."}'
```

### Learning
```bash
curl -X POST "http://localhost:3000/learning" \
  -H "Content-Type: application/json" \
  -d '{"song_id": "song-uuid", "level_up_path": "[1,1,1,1,1,1,1,2,3,5,7,13,19,32,52,84,135,220,355,574]"}'
```

### Rel Show Song
```bash
curl -X POST "http://localhost:3000/rel_show_song" \
  -H "Content-Type: application/json" \
  -d '{"show_id": "show-uuid", "song_id": "song-uuid", "media_url": "https://..."}'
```

**Response:**
```json
{
  "id": "generated-uuid"
}
```

---

## 4. PATCH /:table/:id

Update specific fields of a record. Only the provided fields are updated; `updated_at` is set to the current timestamp automatically.

### Update Song (e.g., reassign artist)
```bash
curl -X PATCH "http://localhost:3000/song/abc123" \
  -H "Content-Type: application/json" \
  -d '{"artist_id": "new-artist-id"}'
```

### Update Learning - Level Up
After a successful review, increment the level. The server updates `last_level_up_at` to the current timestamp.

```bash
curl -X PATCH "http://localhost:3000/learning/def456" \
  -H "Content-Type: application/json" \
  -d '{"level": 8}'
```

### Update Learning - Level Down
If the song is forgotten, set a lower level. The server also updates `last_level_up_at` to the current timestamp, resetting the review timer for the new level's wait period.

```bash
curl -X PATCH "http://localhost:3000/learning/def456" \
  -H "Content-Type: application/json" \
  -d '{"level": 3}'
```

### Update Learning - Graduate
When a song reaches the end of its `level_up_path` (level 19) and is fully memorized:

```bash
curl -X PATCH "http://localhost:3000/learning/def456" \
  -H "Content-Type: application/json" \
  -d '{"graduated": 1}'
```

### Update Artist (e.g., soft delete)
```bash
curl -X PATCH "http://localhost:3000/artist/abc123" \
  -H "Content-Type: application/json" \
  -d '{"status": 1}'
```

**Behavior Notes:**
- When `level` is changed on a learning record (up or down), the server must also update `last_level_up_at` to the current timestamp
- The `updated_at` field is always set to the current timestamp on any update

---

## 5. DELETE /:table/:id

Hard delete a record from the database.

```bash
curl -X DELETE "http://localhost:3000/artist/abc123"
```

**Allowed tables:** `artist`, `song`

**Response:**
```json
{
  "deleted": true
}
```

---

## 6. GET /learning/due

Get all songs due for review based on spaced repetition rules. This is a special endpoint because the due-for-review filter involves computed conditions that can't be expressed as a simple generic query.

**Query Parameters:**
| Parameter | Required | Description |
|-----------|----------|-------------|
| `limit` | No | Maximum number of results (default: 100) |

**Due Filter Logic:**
```sql
graduated = 0 AND (
    -- Level 0 with last_level_up_at set: wait 300 seconds (5 minutes)
    (last_level_up_at > 0 AND level = 0 AND now >= last_level_up_at + 300)
    OR
    -- Level 0 newly created (last_level_up_at not yet set): use updated_at + 300 seconds
    (last_level_up_at = 0 AND level = 0 AND now >= updated_at + 300)
    OR
    -- Level > 0: use level_up_path[level] days
    (level > 0 AND (level_up_path[level] * 86400 + last_level_up_at) <= now)
)
```

**Response:**
```json
{
  "count": 13,
  "results": [
    {
      "id": "5a1af77e-5d26-4b21-92f5-79f4d1332fef",
      "song_id": "...",
      "song_name": "1-nichi wa 25-jikan.",
      "level": 16,
      "wait_days": 135
    }
  ]
}
```

**Full SQL:**
```sql
SELECT l.*, s.name as song_name
FROM learning l
JOIN song s ON l.song_id = s.id
WHERE l.graduated = 0
  AND (
    (l.last_level_up_at > 0 AND l.level = 0
     AND CAST(strftime('%s', 'now') AS INTEGER) >= (l.last_level_up_at + 300))
    OR
    (l.last_level_up_at = 0 AND l.level = 0
     AND CAST(strftime('%s', 'now') AS INTEGER) >= (l.updated_at + 300))
    OR
    (l.level > 0
     AND (json_extract(l.level_up_path, '$[' || l.level || ']') * 86400 + l.last_level_up_at)
         <= CAST(strftime('%s', 'now') AS INTEGER))
  )
ORDER BY l.level DESC;
```

---

## 7. GET /:table/duplicates

Find records with case-insensitive matching names for data quality review.

**Allowed tables:** `artist`, `show`, `song`

```bash
curl "http://localhost:3000/artist/duplicates"
```

**Response:**
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

**Note:** Duplicates may be legitimate (e.g., two real artists with the same name). This endpoint surfaces them for human review.

---

## 8. POST /song/bulk-reassign

Reassign multiple songs to a different artist atomically. Used for fixing import mistakes or merging duplicate artists.

**By song IDs:**
```bash
curl -X POST "http://localhost:3000/song/bulk-reassign" \
  -H "Content-Type: application/json" \
  -d '{
    "song_ids": ["song1", "song2", "song3"],
    "new_artist_id": "correct-artist-id"
  }'
```

**By source artist (move all songs from one artist to another):**
```bash
curl -X POST "http://localhost:3000/song/bulk-reassign" \
  -H "Content-Type: application/json" \
  -d '{
    "from_artist_id": "artist-to-remove",
    "to_artist_id": "artist-to-keep"
  }'
```

**Response:**
```json
{
  "reassigned_count": 3
}
```

---

## 9. POST /learning/batch

Add one or many songs to the learning system. Each song gets a new learning record with `level = 0`, `last_level_up_at = 0`, `graduated = 0`, and a server-generated `level_up_path` based on a Fibonacci easing curve.

**Request Body:**
| Field | Required | Type | Description |
|-------|----------|------|-------------|
| `song_ids` | Yes | string[] | Array of song UUIDs to add to learning |
| `relearn_song_ids` | No | string[] | Song UUIDs of graduated songs to re-learn (see graduated song handling below) |
| `relearn_start_level` | No | integer | Starting level for re-learned songs (default: `7`, stored as 0-indexed). Since these songs were previously learned, they skip the early warm-up levels. |

**Example (single song):**
```bash
curl -X POST "http://localhost:3000/learning/batch" \
  -H "Content-Type: application/json" \
  -d '{"song_ids": ["3b105bd4-c437-4720-a373-660bd5d68532"]}'
```

**Example (multiple songs):**
```bash
curl -X POST "http://localhost:3000/learning/batch" \
  -H "Content-Type: application/json" \
  -d '{"song_ids": ["song-uuid-1", "song-uuid-2", "song-uuid-3"]}'
```

**Response:**
```json
{
  "created_ids": [
    "generated-learning-uuid-1",
    "generated-learning-uuid-2"
  ],
  "skipped_song_ids": [
    "song-uuid-3"
  ],
  "already_graduated_song_ids": [
    "song-uuid-4"
  ]
}
```

- `created_ids`: UUIDs of newly created learning records (includes both new songs and re-learned graduated songs)
- `skipped_song_ids`: song UUIDs that were skipped because they already have an active (non-graduated) learning record
- `already_graduated_song_ids`: song UUIDs that have a graduated learning record but were NOT included in `relearn_song_ids`, so they were not re-added. The caller should review these and decide whether to re-learn them in a follow-up request.

**Behavior:**
- Each learning record is created with:
  - `id`: server-generated UUID
  - `song_id`: from the request
  - `level`: `0`
  - `created_at`: current Unix timestamp
  - `updated_at`: current Unix timestamp
  - `last_level_up_at`: `0` (indicates the song has never been leveled up)
  - `level_up_path`: server-generated JSON array (see [Level-Up Path Generation](#level-up-path-generation) below)
  - `graduated`: `0`
- All inserts are performed in a single transaction (all succeed or all fail)

### Skip and Re-learn Rules

1. **Active (non-graduated) learning record exists → skip**: If a `song_id` already has a learning record with `graduated = 0`, the song is skipped and appears in `skipped_song_ids`. This prevents duplicate active learning records.
2. **Graduated learning record exists → requires confirmation**: If a `song_id` has only graduated learning records (`graduated = 1`), the song is NOT automatically re-added. Instead, it appears in `already_graduated_song_ids`. The caller should review these and explicitly include them in `relearn_song_ids` in a follow-up request to confirm re-learning.
3. **Graduated song explicitly confirmed for re-learning**: If a `song_id` appears in both `song_ids` and `relearn_song_ids`, and the song only has graduated records, a new learning record is created starting from `relearn_start_level` (default: `7`, which is display level 8). Since the song was previously memorized, it skips the early warm-up levels. The old graduated record is preserved.

**Typical two-step flow for graduated songs:**

```bash
# Step 1: Attempt to add songs (some may be graduated)
curl -X POST "http://localhost:3000/learning/batch" \
  -H "Content-Type: application/json" \
  -d '{"song_ids": ["song-new", "song-active", "song-graduated"]}'
# Response: created_ids: ["..."], skipped_song_ids: ["song-active"], already_graduated_song_ids: ["song-graduated"]

# Step 2: After user confirms, re-learn the graduated songs (default start level 7)
curl -X POST "http://localhost:3000/learning/batch" \
  -H "Content-Type: application/json" \
  -d '{"song_ids": ["song-graduated"], "relearn_song_ids": ["song-graduated"]}'
# Response: created_ids: ["..."], skipped_song_ids: [], already_graduated_song_ids: []

# Step 2 (alternative): Re-learn with custom start level (e.g., start at stored level 5 = display level 6)
curl -X POST "http://localhost:3000/learning/batch" \
  -H "Content-Type: application/json" \
  -d '{"song_ids": ["song-graduated"], "relearn_song_ids": ["song-graduated"], "relearn_start_level": 5}'
# Response: created_ids: ["..."], skipped_song_ids: [], already_graduated_song_ids: []
```

**Error Cases:**
| Condition | Status | Response |
|-----------|--------|----------|
| `song_ids` is empty | 400 | `{"error": "song_ids cannot be empty"}` |
| `song_id` not found in song table | 400 | `{"error": "song not found: <id>"}` |

### Level-Up Path Generation

The `level_up_path` is a JSON array of wait-days generated using a **Fibonacci-based easing curve**. The algorithm produces gradually increasing review intervals:

**Algorithm:**
1. Compute the Fibonacci number for each level: `fibo(n)` where `fibo(0)=0, fibo(1)=1, fibo(n)=fibo(n-1)+fibo(n-2)`
2. Shrink each value: `shrink(x) = x * 2 / 9` (integer division)
3. For each level, the wait-days = difference between consecutive shrunk values: `shrink(fibo(n+1)) - shrink(fibo(n))`
4. If the difference is 0, use 1 (minimum 1-day wait)

**Default path (20 levels):**
```json
[1, 1, 1, 1, 1, 1, 1, 2, 3, 5, 7, 13, 19, 32, 52, 84, 135, 220, 355, 574]
```

| Stored Level | Display Level | Wait (days) | Cumulative (days) |
|--------------|---------------|-------------|-------------------|
| 0 | 1 | 1 | 1 |
| 1 | 2 | 1 | 2 |
| 2 | 3 | 1 | 3 |
| 3 | 4 | 1 | 4 |
| 4 | 5 | 1 | 5 |
| 5 | 6 | 1 | 6 |
| 6 | 7 | 1 | 7 |
| 7 | 8 | 2 | 9 |
| 8 | 9 | 3 | 12 |
| 9 | 10 | 5 | 17 |
| 10 | 11 | 7 | 24 |
| 11 | 12 | 13 | 37 |
| 12 | 13 | 19 | 56 |
| 13 | 14 | 32 | 88 |
| 14 | 15 | 52 | 140 |
| 15 | 16 | 84 | 224 |
| 16 | 17 | 135 | 359 |
| 17 | 18 | 220 | 579 |
| 18 | 19 | 355 | 934 |
| 19 | 20 | 574 | 1508 |

> **Note:** "Stored Level" is the 0-indexed value in the database and API. "Display Level" is stored_level + 1, used for user-facing display. See [Level Display Convention](concept.md#level-display-convention) for details.

The first 7 levels have 1-day intervals (warm-up), then intervals grow following the Fibonacci curve — reaching ~1.5 years of cumulative review time for a fully graduated song.

---

## Operations Coverage

This table shows how every required operation maps to the minimized endpoints.

### Import Workflow
| Operation | Endpoint |
|-----------|----------|
| Find artist by name | `GET /artist/search?name=X` |
| Find show by name + vintage | `GET /show/search?name=X&vintage=Y` |
| Find song by name + artist | `GET /song/search?name=X&artist_id=Y` |
| List songs by artist (disambiguation) | `GET /song/search?artist_id=X` |
| Create artist | `POST /artist` |
| Create show | `POST /show` |
| Create song | `POST /song` |
| Create play history | `POST /play_history` |
| Link song to show | `POST /rel_show_song` |

### Learning / Spaced Repetition
| Operation | Endpoint |
|-----------|----------|
| Get songs due for review | `GET /learning/due` |
| Create learning record(s) | `POST /learning/batch` |
| Level up | `PATCH /learning/:id` with `{"level": N}` |
| Level down | `PATCH /learning/:id` with `{"level": N}` |
| Graduate | `PATCH /learning/:id` with `{"graduated": 1}` |

### Data Quality
| Operation | Endpoint |
|-----------|----------|
| Find duplicate artists/shows/songs | `GET /:table/duplicates` |
| Reassign single song | `PATCH /song/:id` with `{"artist_id": "..."}` |
| Bulk reassign songs | `POST /song/bulk-reassign` |
| Soft-delete artist | `PATCH /artist/:id` with `{"status": 1}` |
| Hard-delete artist or song | `DELETE /:table/:id` |

### General
| Operation | Endpoint |
|-----------|----------|
| Read any record by ID | `GET /:table/:id?fields=...` |
| Update any record | `PATCH /:table/:id` |

---

## Response Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 400 | Bad request (invalid parameters, JankenSQLHub validation failure) |
| 404 | Resource not found |
| 500 | Internal server error |

## Error Response Format

```json
{
  "error": "Description of the error"
}