# Core Concepts

## Overview

Janken helps you track and learn anime songs through a spaced repetition system. After playing anime guessing games on [animemusicquiz.com](https://animemusicquiz.com), you can save songs to your play history and use the learning system to systematically memorize them.

## Data Model

| Entity | Description |
|--------|-------------|
| **Shows** | Anime series or movies (e.g., "K-On! Season 2", "So I'm a Spider, So What?") |
| **Songs** | Theme songs from anime shows (openings, endings, inserts) |
| **Artists** | Singers/bands who perform the songs |
| **Play History** | Records of songs encountered during quiz games |
| **Learning** | Spaced repetition tracking to help memorize songs |

## Relationships

```
artist ──< song >── rel_show_song ──< show
                         │
                    play_history
                         │
                     learning
```

- Each **song** belongs to one **artist**
- **Shows** and **songs** have a many-to-many relationship via `rel_show_song`
- **Play history** records each song encounter with show context
- **Learning** tracks memorization progress for individual songs

## Spaced Repetition System

The learning system uses customizable review intervals stored per song:

```
level_up_path: [1,1,1,1,1,1,1,2,3,5,7,13,19,32,52,84,135,220,355,574]
```

**How it works:**
- Each index represents a **level** (starting from 0)
- The value is the **number of days to wait** before the next review
- When you level up, `last_level_up_at` is updated to the current timestamp
- A song appears in "time to learn" list when: `now >= last_level_up_at + path[level] days`
- When `graduated = 1`, the song has been fully memorized

**Example:** At level 7 with path value `2`, you should review the song 2 days after your last level-up.

### Level Display Convention

> **Important:** The level is stored as a **0-indexed** value in the database (for natural array indexing in SQL and code), but should always be displayed to the user as **stored_level + 1** (1-indexed).

| Stored (DB) | Displayed (UI) | Meaning |
|-------------|----------------|---------|
| 0 | 1 | First level (just added) |
| 7 | 8 | Default re-learn start |
| 19 | 20 | Final level before graduation |

This convention keeps the database representation aligned with array indexing (`level_up_path[level]` works directly), while the user-facing display is more intuitive ("Level 1" instead of "Level 0"). All API inputs and outputs use the **stored (0-indexed)** value — the +1 transformation is a **display-only** concern handled by the client/UI.

**Level changes:**
- **Level up**: After correctly reviewing a song, increment its level. The `last_level_up_at` is updated to the current timestamp.
- **Level down**: If the song is forgotten or needs more practice, set the level to a lower value. The `last_level_up_at` is also updated to the current timestamp, resetting the review timer for the new level's wait period.
- **Graduate**: When a song reaches the end of its `level_up_path` and is fully memorized, set `graduated = 1`.

## Due for Review Filter

A song is considered "due for review" when it meets the following conditions:

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

**Explanation:**
- `graduated = 0`: Only non-graduated songs need review
- **Level 0 (newly added)**: Songs at level 0 were just added and have a short 5-minute (300 seconds) warm-up period before their first review
  - If `last_level_up_at` is set, use it
  - For newly created records where `last_level_up_at` hasn't been set yet, fall back to `updated_at`
- **Level > 0**: Uses the `level_up_path` array to determine wait time in days (converted to seconds by multiplying by 86400)

**Full SQL Query Example:**
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

## Flexible Memory Curves

The `level_up_path` is stored per song, allowing for different memory curves. While most songs currently share the same Fibonacci-like progression, the system is designed to support:

- **Personalized curves** based on individual learning patterns
- **Song-specific curves** (e.g., harder songs might need more frequent reviews)
- **Experimental curves** for optimizing retention

### Default Path Generation (Fibonacci Easing)

When songs are added to learning via `POST /learning/batch`, the server generates the `level_up_path` automatically using a Fibonacci-based easing algorithm:

1. **Fibonacci sequence**: `fibo(0)=0, fibo(1)=1, fibo(n)=fibo(n-1)+fibo(n-2)`
2. **Shrink**: Scale down each value with `shrink(x) = x * 2 / 9` (integer division)
3. **Difference**: For each level `n`, wait-days = `shrink(fibo(n+1)) - shrink(fibo(n))`
4. **Floor**: If the difference is 0, use 1 (minimum 1-day wait)

This produces a smooth ramp from short intervals (1 day) to long intervals (574 days), following the natural growth rate of the Fibonacci sequence. The early levels act as a warm-up with 1-day waits, while later levels space out reviews as the song becomes more familiar.

**Default 20-level path:**
```
[1, 1, 1, 1, 1, 1, 1, 2, 3, 5, 7, 13, 19, 32, 52, 84, 135, 220, 355, 574]
```

See the [API Reference — Level-Up Path Generation](api.md#level-up-path-generation) for the full breakdown table.

## Adding Songs to Learning

Songs are added to the learning system via `POST /learning/batch` with one or more `song_ids`. Each new learning record is initialized with:

| Field | Value |
|-------|-------|
| `level` | `0` |
| `last_level_up_at` | `0` (never leveled up) |
| `graduated` | `0` |
| `level_up_path` | Server-generated (Fibonacci easing, 20 levels) |

Setting `last_level_up_at = 0` signals that the song is brand new. The [due-for-review filter](#due-for-review-filter) handles this case by falling back to `updated_at + 300 seconds` for the initial review timing.

### Skip and Re-learn Rules

When adding songs to learning, the system checks each song's existing learning status:

1. **No existing record** → create a new learning record
2. **Active (non-graduated) record exists** (`graduated = 0`) → **skip** the song. This prevents duplicate active learning records for the same song.
3. **Only graduated records exist** (`graduated = 1`) → **do not auto-add**. Instead, return the song in `already_graduated_song_ids` so the caller can decide whether to re-learn it. If the caller explicitly confirms (via `relearn_song_ids`), a new learning record is created starting from stored level `7` (display level 8) by default, since the song was previously memorized and doesn't need the early warm-up levels. The start level is customizable via `relearn_start_level`. The old graduated record is preserved as history.

This two-step confirmation for graduated songs prevents accidental re-learning of songs that were intentionally completed. See the [API Reference — Skip and Re-learn Rules](api.md#skip-and-re-learn-rules) for the full request/response flow.
