# CLI Learning Commands

Commands for spaced repetition and song memorization. See [CLI Reference](cli.md) for an overview of all commands.

---

## jankenoboe learning-due

Get all songs due for review based on spaced repetition rules. This is a special command because the due-for-review filter involves computed conditions that can't be expressed as a simple generic query.

**Options:**
| Option | Required | Description |
|--------|----------|-------------|
| `--limit` | No | Maximum number of results (default: 100) |
| `--offset` | No | Look-ahead offset in seconds (default: 0). Shifts the reference time forward, e.g., `--offset 7200` finds songs due in the next 2 hours. |

**Example:**
```bash
jankenoboe learning-due
jankenoboe learning-due --limit 20
jankenoboe learning-due --offset 7200          # due within the next 2 hours
jankenoboe learning-due --offset 120 --limit 50 # due within the next 2 minutes
```

**Due Filter Logic:**

When `--offset` is provided, `now` in the SQL becomes `now + offset_seconds`. With `--offset 0` (default), the behavior is identical to the original.

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

**Output:**
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

> When `--offset 0` (default), the `+ <offset>` term is omitted and the query is identical to comparing against `now`.

---

## jankenoboe learning-batch

Add one or many songs to the learning system. Each song gets a new learning record with `level = 0`, `last_level_up_at = 0`, `graduated = 0`, and a generated `level_up_path` based on a Fibonacci easing curve.

**Options:**
| Option | Required | Description |
|--------|----------|-------------|
| `--song-ids` | Yes | Comma-separated song UUIDs to add to learning |
| `--relearn-song-ids` | No | Comma-separated song UUIDs of graduated songs to re-learn |
| `--relearn-start-level` | No | Starting level for re-learned songs (default: `7`, stored as 0-indexed) |

**Example (single song):**
```bash
jankenoboe learning-batch --song-ids 3b105bd4-c437-4720-a373-660bd5d68532
```

**Example (multiple songs):**
```bash
jankenoboe learning-batch --song-ids song-uuid-1,song-uuid-2,song-uuid-3
```

**Output:**
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
- `skipped_song_ids`: song UUIDs skipped because they already have an active (non-graduated) learning record
- `already_graduated_song_ids`: song UUIDs with graduated records not included in `--relearn-song-ids`

**Behavior:**
- Each learning record is created with:
  - `id`: generated UUID
  - `song_id`: from the input
  - `level`: `0`
  - `created_at`: current Unix timestamp
  - `updated_at`: current Unix timestamp
  - `last_level_up_at`: `0` (never leveled up)
  - `level_up_path`: generated JSON array (see [Level-Up Path Generation](#level-up-path-generation))
  - `graduated`: `0`
- All inserts are performed in a single transaction (all succeed or all fail)

### Skip and Re-learn Rules

1. **Active (non-graduated) record exists → skip**: Song is skipped, appears in `skipped_song_ids`.
2. **Graduated record exists → requires confirmation**: Song appears in `already_graduated_song_ids`. Include it in `--relearn-song-ids` to confirm.
3. **Graduated song confirmed for re-learning**: New record created starting from `--relearn-start-level` (default: `7`). Old graduated record preserved.

**Typical two-step flow for graduated songs:**

```bash
# Step 1: Attempt to add songs (some may be graduated)
jankenoboe learning-batch --song-ids song-new,song-active,song-graduated
# Output: created_ids: ["..."], skipped_song_ids: ["song-active"], already_graduated_song_ids: ["song-graduated"]

# Step 2: After user confirms, re-learn the graduated songs
jankenoboe learning-batch --song-ids song-graduated --relearn-song-ids song-graduated

# Step 2 (alternative): Re-learn with custom start level
jankenoboe learning-batch --song-ids song-graduated --relearn-song-ids song-graduated --relearn-start-level 5
```

**Error Cases:**
| Condition | Exit Code | Output |
|-----------|-----------|--------|
| `--song-ids` is empty | 1 | `{"error": "song_ids cannot be empty"}` |
| `song_id` not found in song table | 1 | `{"error": "song not found: <id>"}` |

### Level-Up Path Generation

The `level_up_path` is a JSON array of wait-days generated using a **Fibonacci-based easing curve**:

**Algorithm:**
1. Compute Fibonacci: `fibo(0)=0, fibo(1)=1, fibo(n)=fibo(n-1)+fibo(n-2)`
2. Shrink: `shrink(x) = x * 2 / 9` (integer division)
3. Difference: wait-days = `shrink(fibo(n+1)) - shrink(fibo(n))`
4. Floor: minimum 1-day wait

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

> **Note:** "Stored Level" is 0-indexed (database/CLI). "Display Level" is stored_level + 1 (user-facing). See [Level Display Convention](concept.md#level-display-convention).

The first 7 levels have 1-day intervals (warm-up), then intervals grow following the Fibonacci curve — reaching ~1.5 years of cumulative review time for a fully graduated song.

### Related: Level Up/Down/Graduate via Update

Level changes are performed using the generic `update` command from [Data Management](cli-data-management.md):

```bash
# Level up
jankenoboe update learning <id> --data '{"level": 8}'

# Level down
jankenoboe update learning <id> --data '{"level": 3}'

# Graduate
jankenoboe update learning <id> --data '{"graduated": 1}'
```

When `level` is changed, `last_level_up_at` is automatically updated to the current timestamp.

---

## jankenoboe learning-song-review

Generate a self-contained HTML report of all songs currently due for review. The report is an offline file that can be opened in any browser — no server required.

For each due song, the report enriches the data with:
- **Artist name** (from the song's artist)
- **Show names** (from `rel_show_song` → `show`)
- **All media URLs** for the song (from both `rel_show_song` and `play_history`, deduplicated)

The report includes client-side pagination and statistics (total count, level distribution).

**Options:**
| Option | Required | Description |
|--------|----------|-------------|
| `--output` | No | Output file path (default: `learning-song-review.html` in current directory) |
| `--limit` | No | Maximum number of due songs to include (default: 500) |
| `--offset` | No | Look-ahead offset in seconds (default: 0). Same as `learning-due --offset`. |

**Example:**
```bash
jankenoboe learning-song-review
jankenoboe learning-song-review --output ~/reports/review.html
jankenoboe learning-song-review --limit 50
jankenoboe learning-song-review --offset 7200   # include songs due within 2 hours
```

**Output (stdout):**
```json
{
  "file": "/path/to/learning-song-review.html",
  "count": 13,
  "learning_ids": [
    "5a1af77e-5d26-4b21-92f5-79f4d1332fef",
    "..."
  ]
}
```

- `learning_ids`: Array of learning record UUIDs included in the report. Use these with `learning-song-levelup-ids` to level up exactly the songs shown in the report, avoiding race conditions from new due songs appearing between report generation and level-up.

**HTML Report Features:**
- Summary statistics: total due songs, level distribution breakdown
- Each song card shows: song name, artist, level (display = stored + 1), wait days, show names, and clickable media URLs
- Client-side pagination (20 songs per page) for large lists
- Songs sorted by level descending (highest level first)
- Self-contained: no external dependencies, works offline

---

## jankenoboe learning-song-levelup-ids

Level up specific learning records by their IDs. This command does **not** check whether songs are due — it levels up exactly the specified records. This is the race-condition-safe way to level up songs after reviewing a report.

**Options:**
| Option | Required | Description |
|--------|----------|-------------|
| `--ids` | Yes | Comma-separated learning record UUIDs |

**Example:**
```bash
jankenoboe learning-song-levelup-ids --ids learning-uuid-1,learning-uuid-2
```

**Output:**
```json
{
  "leveled_up_count": 1,
  "graduated_count": 1,
  "total_processed": 2
}
```

**Behavior:**
- Validates that all specified IDs exist and are not already graduated
- For each record with level < 19: increments level by 1, updates `last_level_up_at` and `updated_at`
- For each record with level = 19: sets `graduated = 1`, updates `last_level_up_at` and `updated_at`
- All updates are performed in a single transaction
- Does **not** require songs to be due — works regardless of due status

**Error Cases:**
| Condition | Exit Code | Output |
|-----------|-----------|--------|
| `--ids` is empty | 1 | `{"error": "ids cannot be empty"}` |
| Any ID not found | 1 | `{"error": "learning record(s) not found: <ids>"}` |
| Any ID already graduated | 1 | `{"error": "learning record already graduated: <id>"}` |

**Typical workflow with `learning-song-review`:**

```bash
# Step 1: Generate review report (captures learning_ids at this moment)
out=$(jankenoboe learning-song-review --output ~/review.html)
# out: {"file":"...","count":13,"learning_ids":["id1","id2",...]}

# Step 2: User reviews the HTML report in browser

# Step 3: Level up exactly those songs (no race condition)
ids=$(echo "$out" | jq -r '.learning_ids | join(",")')
jankenoboe learning-song-levelup-ids --ids "$ids"
```
