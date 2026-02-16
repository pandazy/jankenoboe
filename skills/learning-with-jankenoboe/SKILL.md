---
name: learning-with-jankenoboe
description: Manage anime song spaced repetition learning via Jankenoboe. Add songs to learning, level up/down, graduate, check due reviews, and re-learn graduated songs. Use when the user wants to practice, review, level up, or manage their anime song learning progress. Supports both HTTP API (localhost:3000) and direct SQLite queries.
---

## Setup

**Always ask the user for the SQLite database file path first** (e.g., `datasource.db`). Store it for the session.

**Determine access mode:**
1. **HTTP mode**: Try `curl -s http://localhost:3000/learning/due`. If it responds, use HTTP.
2. **SQL mode**: If the server is unreachable, use `sqlite3 <db_path>` directly.

**⚠️ Shell safety for SQL mode:** Many SQL queries contain `%s` (in `strftime`) and `$[` (in `json_extract`) which zsh interprets as format specifiers and arithmetic expansion. **Always write SQL to a temp file and pipe it to sqlite3** instead of passing inline:
```bash
cat > /tmp/query.sql << 'EOSQL'
SELECT CAST(strftime('%s','now') AS INTEGER);
EOSQL
sqlite3 <db_path> < /tmp/query.sql
```
The `<< 'EOSQL'` (quoted heredoc) prevents all shell interpolation inside the SQL.

## Spaced Repetition Overview

Songs progress through 20 levels (stored 0-19, displayed 1-20). Each level has a wait period in days before the next review. Early levels wait 1 day; later levels wait months. The `level_up_path` JSON array stores wait-days per level.

Default path: `[1,1,1,1,1,1,1,2,3,5,7,13,19,32,52,84,135,220,355,574]`

## Check Due Reviews

**HTTP:** `GET /learning/due?limit=100`

**SQL:**
```sql
SELECT l.id, l.song_id, s.name as song_name, l.level,
  json_extract(l.level_up_path, '$[' || l.level || ']') as wait_days
FROM learning l JOIN song s ON l.song_id = s.id
WHERE l.graduated = 0 AND (
  (l.last_level_up_at > 0 AND l.level = 0
   AND CAST(strftime('%s','now') AS INTEGER) >= l.last_level_up_at + 300)
  OR (l.last_level_up_at = 0 AND l.level = 0
   AND CAST(strftime('%s','now') AS INTEGER) >= l.updated_at + 300)
  OR (l.level > 0
   AND json_extract(l.level_up_path,'$['||l.level||']') * 86400 + l.last_level_up_at
       <= CAST(strftime('%s','now') AS INTEGER))
) ORDER BY l.level DESC;
```

## Add Songs to Learning (Batch)

**HTTP:**
```bash
curl -X POST http://localhost:3000/learning/batch \
  -H "Content-Type: application/json" \
  -d '{"song_ids": ["<song_id_1>", "<song_id_2>"]}'
```

Response includes `created_ids`, `skipped_song_ids` (already active), `already_graduated_song_ids`.

**SQL (per song):**
```sql
-- Check if active learning exists
SELECT id FROM learning WHERE song_id='<song_id>' AND graduated=0;
-- If no active record, insert:
INSERT INTO learning (id, song_id, level, created_at, updated_at, last_level_up_at, level_up_path, graduated)
VALUES ('<uuid>', '<song_id>', 0,
  CAST(strftime('%s','now') AS INTEGER),
  CAST(strftime('%s','now') AS INTEGER),
  0,
  '[1,1,1,1,1,1,1,2,3,5,7,13,19,32,52,84,135,220,355,574]',
  0);
```

Generate UUIDs via `python3 -c "import uuid; print(uuid.uuid4())"`.

## Re-learn Graduated Songs

**HTTP:**
```bash
curl -X POST http://localhost:3000/learning/batch \
  -H "Content-Type: application/json" \
  -d '{"song_ids": ["<id>"], "relearn_song_ids": ["<id>"], "relearn_start_level": 7}'
```

**SQL:** Same as above but set `level=7` (or chosen start level) instead of 0.

## Level Up

**HTTP:**
```bash
curl -X PATCH http://localhost:3000/learning/<learning_id> \
  -H "Content-Type: application/json" \
  -d '{"level": <new_level>}'
```

**SQL:**
```sql
UPDATE learning
SET level=<new_level>,
    last_level_up_at=CAST(strftime('%s','now') AS INTEGER),
    updated_at=CAST(strftime('%s','now') AS INTEGER)
WHERE id='<learning_id>';
```

## Level Down

Same as level up but with a lower level number. Server also updates `last_level_up_at`.

## Graduate

**HTTP:**
```bash
curl -X PATCH http://localhost:3000/learning/<learning_id> \
  -H "Content-Type: application/json" \
  -d '{"graduated": 1}'
```

**SQL:**
```sql
UPDATE learning
SET graduated=1, updated_at=CAST(strftime('%s','now') AS INTEGER)
WHERE id='<learning_id>';
```

## Typical Review Workflow

1. Get due songs → present to user
2. User reviews each song → level up (correct) or level down (forgotten)
3. Song at level 19 and correct → graduate