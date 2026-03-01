# CLI Learning Commands

Commands for spaced repetition and song memorization. See [CLI Reference](cli.md) for an overview of all commands.

> **Usage examples and workflows:** See [learning-with-jankenoboe skill](../.claude/skills/learning-with-jankenoboe/SKILL.md) and [reviewing-due-songs skill](../.claude/skills/reviewing-due-songs/SKILL.md) for comprehensive examples, workflows, and output formats.

---

## jankenoboe learning-due

Get all songs due for review based on spaced repetition rules.

**Options:**
| Option | Required | Description |
|--------|----------|-------------|
| `--limit` | No | Maximum number of results (default: 100) |
| `--offset` | No | Look-ahead offset in seconds (default: 0). Shifts the reference time forward. |

**Due Filter Logic:**

When `--offset` is provided, `now` in the SQL becomes `now + offset_seconds`.

```sql
graduated = 0 AND (
    -- Level 0 with last_level_up_at set: wait 300 seconds (5 minutes)
    (last_level_up_at > 0 AND level = 0 AND (now + offset) >= last_level_up_at + 300)
    OR
    -- Level 0 newly created (last_level_up_at not yet set): use updated_at + 300 seconds
    (last_level_up_at = 0 AND level = 0 AND (now + offset) >= updated_at + 300)
    OR
    -- Level > 0: use level_up_path[level] days
    (level > 0 AND (level_up_path[level] * 86400 + last_level_up_at) <= (now + offset))
)
```

**Full SQL (with offset):**
```sql
SELECT l.*, s.name as song_name
FROM learning l
JOIN song s ON l.song_id = s.id
WHERE l.graduated = 0
  AND (
    (l.last_level_up_at > 0 AND l.level = 0
     AND (CAST(strftime('%s', 'now') AS INTEGER) + <offset>) >= (l.last_level_up_at + 300))
    OR
    (l.last_level_up_at = 0 AND l.level = 0
     AND (CAST(strftime('%s', 'now') AS INTEGER) + <offset>) >= (l.updated_at + 300))
    OR
    (l.level > 0
     AND (json_extract(l.level_up_path, '$[' || l.level || ']') * 86400 + l.last_level_up_at)
         <= (CAST(strftime('%s', 'now') AS INTEGER) + <offset>))
  )
ORDER BY l.level DESC;
```

---

## jankenoboe learning-batch

Add one or many songs to the learning system. Each song gets a new learning record with `level = 0`, `last_level_up_at = 0`, `graduated = 0`, and a generated `level_up_path`.

**Options:**
| Option | Required | Description |
|--------|----------|-------------|
| `--song-ids` | Yes | Comma-separated song UUIDs to add to learning |
| `--relearn-song-ids` | No | Comma-separated song UUIDs of graduated songs to re-learn |
| `--relearn-start-level` | No | Starting level for re-learned songs (default: `7`, stored as 0-indexed) |

**Behavior:**
- All inserts are performed in a single transaction
- Each record is created with: generated UUID, `level = 0`, current timestamps, `last_level_up_at = 0`, generated `level_up_path`, `graduated = 0`

**Skip and Re-learn Rules:**
1. **Active record exists → skip**: appears in `skipped_song_ids`
2. **Graduated record exists → requires confirmation**: appears in `already_graduated_song_ids`; include in `--relearn-song-ids` to confirm
3. **Graduated song confirmed**: new record created at `--relearn-start-level` (default 7); old record preserved

**Error Cases:**
| Condition | Exit Code | Output |
|-----------|-----------|--------|
| `--song-ids` is empty | 1 | `{"error": "song_ids cannot be empty"}` |
| `song_id` not found in song table | 1 | `{"error": "song not found: <id>"}` |

### Level-Up Path Generation

The `level_up_path` is a JSON array of wait-days generated using a **Fibonacci-based easing curve**:

**Algorithm:** `fibo(n)` → `shrink(x) = x * 2 / 9` → difference between consecutive values → floor at 1 day

**Default path (20 levels):**
```json
[1, 1, 1, 1, 1, 1, 1, 2, 3, 5, 7, 13, 19, 32, 52, 84, 135, 220, 355, 574]
```

| Stored Level | Display Level | Wait (days) | Cumulative (days) |
|--------------|---------------|-------------|-------------------|
| 0 | 1 | 1 | 1 |
| 1–6 | 2–7 | 1 | 2–7 |
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

> **Note:** "Stored Level" is 0-indexed (database/CLI). "Display Level" is stored_level + 1 (user-facing). See [Level Display Convention](concept.md#level-display-convention).

---

## jankenoboe learning-by-song-ids

Get learning records for specific songs. Returns all records (active and graduated) with song names and computed wait days. Uses JankenSQLHub's `:[song_ids]` list parameter.

**Options:**
| Option | Required | Description |
|--------|----------|-------------|
| `--song-ids` | Yes | Comma-separated song UUIDs |

**Behavior:**
- Includes both active and graduated records
- A single song may have multiple records (graduated + active re-learn)
- Ordered by level descending
- Songs with no learning records are absent (no error)

**Error Cases:**
| Condition | Exit Code | Output |
|-----------|-----------|--------|
| `--song-ids` is empty | 1 | `{"error": "song_ids cannot be empty"}` |

---

## jankenoboe learning-song-stats

Get learning statistics per song. Groups all learning records by song and calculates the time span from earliest creation to most recent level-up.

**Options:**
| Option | Required | Description |
|--------|----------|-------------|
| `--song-ids` | Yes | Comma-separated song UUIDs |

**Fields:**
| Field | Description |
|-------|-------------|
| `song_id` | Song UUID |
| `song_name` | Song name |
| `earliest_created_at` | Unix timestamp of the earliest learning record creation |
| `latest_last_level_up_at` | Unix timestamp of the most recent level-up |
| `days_spent` | `ROUND(ABS(MAX(last_level_up_at) - MIN(created_at)) / 86400)` |
| `play_count` | Total number of play_history records for the song |

**Behavior:**
- Groups all learning records by `song_id` (including graduated and re-learn)
- Ordered by `days_spent` descending
- Songs with no learning records are absent (no error)

---

## jankenoboe learning-song-review

Generate a self-contained HTML report of all songs currently due for review.

**Options:**
| Option | Required | Description |
|--------|----------|-------------|
| `--output` | No | Output file path (default: `learning-song-review.html` in current directory) |
| `--limit` | No | Maximum number of due songs (default: 500) |
| `--offset` | No | Look-ahead offset in seconds (default: 0) |

**Output includes `learning_ids`:** Array of learning record UUIDs in the report. Use with `learning-song-levelup-ids` to level up exactly the reviewed songs, avoiding race conditions.

**HTML Report Features:**
- Summary statistics: total due songs, level distribution
- Each song: name, artist, level (display = stored + 1), wait days, show names, clickable media URLs
- Copyable IDs per song: learning ID, song ID, show ID(s) with one-click copy
- Client-side pagination (20 per page), sorted by level descending
- Self-contained, works offline

---

## jankenoboe learning-song-levelup-ids

Level up specific learning records by their IDs. Does **not** check due status — levels up exactly the specified records. Race-condition-safe.

**Options:**
| Option | Required | Description |
|--------|----------|-------------|
| `--ids` | Yes | Comma-separated learning record UUIDs |

**Behavior:**
- Level < 19: increments level, updates `last_level_up_at` and `updated_at`
- Level = 19: sets `graduated = 1`, updates timestamps
- All updates in a single transaction

**Error Cases:**
| Condition | Exit Code | Output |
|-----------|-----------|--------|
| `--ids` is empty | 1 | `{"error": "ids cannot be empty"}` |
| Any ID not found | 1 | `{"error": "learning record(s) not found: <ids>"}` |
| Any ID already graduated | 1 | `{"error": "learning record already graduated: <id>"}` |

---

### Related: Level Up/Down/Graduate via Update

Level changes can also be performed using `update` from [Data Management](cli-data-management.md):

```bash
jankenoboe update learning <id> --data '{"level": 8}'      # level up
jankenoboe update learning <id> --data '{"level": 3}'      # level down
jankenoboe update learning <id> --data '{"graduated": 1}'  # graduate
```

When `level` is changed, `last_level_up_at` is automatically updated to the current timestamp.