---
name: maintaining-jankenoboe-data
description: Create, update, and delete anime song data in Jankenoboe. Manages artists, shows, songs, play history, and show-song relationships. Handles bulk song reassignment, duplicate detection, and soft deletes. Use when the user wants to add, edit, delete, import, fix, merge, or reassign anime song records. Supports both HTTP API (localhost:3000) and direct SQLite queries.
---

## Setup

**Always ask the user for the SQLite database file path first** (e.g., `datasource.db`). Store it for the session.

**Determine access mode:**
1. **HTTP mode**: Try `curl -s http://localhost:3000/artist/search?fields=id,name&name=test`. If it responds, use HTTP.
2. **SQL mode**: If the server is unreachable, use `sqlite3 <db_path>` directly.

**⚠️ Shell safety for SQL mode:** SQL queries containing `%s` (in `strftime`) get interpreted by zsh as format specifiers. **Always write SQL to a temp file and pipe it to sqlite3** instead of passing inline:
```bash
cat > /tmp/query.sql << 'EOSQL'
INSERT INTO artist (id, name, created_at, updated_at, status)
VALUES ('abc', 'Test', CAST(strftime('%s','now') AS INTEGER), CAST(strftime('%s','now') AS INTEGER), 0);
EOSQL
sqlite3 <db_path> < /tmp/query.sql
```
The `<< 'EOSQL'` (quoted heredoc) prevents all shell interpolation inside the SQL.

## Create Records

Server generates `id` (UUID) and timestamps. In SQL mode, generate UUIDs via `python3 -c "import uuid; print(uuid.uuid4())"`.

### Artist
**HTTP:**
```bash
curl -X POST http://localhost:3000/artist \
  -H "Content-Type: application/json" \
  -d '{"name": "<name>"}'
```
**SQL:**
```sql
INSERT INTO artist (id, name, created_at, updated_at, status)
VALUES ('<uuid>', '<name>', CAST(strftime('%s','now') AS INTEGER), CAST(strftime('%s','now') AS INTEGER), 0);
```

### Show
**HTTP:**
```bash
curl -X POST http://localhost:3000/show \
  -H "Content-Type: application/json" \
  -d '{"name": "<name>", "name_romaji": "<romaji>", "vintage": "<vintage>", "s_type": "<type>"}'
```
**SQL:**
```sql
INSERT INTO show (id, name, name_romaji, vintage, s_type, created_at, updated_at, status)
VALUES ('<uuid>', '<name>', '<romaji>', '<vintage>', '<type>',
  CAST(strftime('%s','now') AS INTEGER), CAST(strftime('%s','now') AS INTEGER), 0);
```

### Song
**HTTP:**
```bash
curl -X POST http://localhost:3000/song \
  -H "Content-Type: application/json" \
  -d '{"name": "<name>", "artist_id": "<artist_id>"}'
```
**SQL:**
```sql
INSERT INTO song (id, name, artist_id, created_at, updated_at, status)
VALUES ('<uuid>', '<name>', '<artist_id>',
  CAST(strftime('%s','now') AS INTEGER), CAST(strftime('%s','now') AS INTEGER), 0);
```

### Play History
**HTTP:**
```bash
curl -X POST http://localhost:3000/play_history \
  -H "Content-Type: application/json" \
  -d '{"show_id": "<show_id>", "song_id": "<song_id>", "media_url": "<url>"}'
```
**SQL:**
```sql
INSERT INTO play_history (id, show_id, song_id, media_url, created_at, status)
VALUES ('<uuid>', '<show_id>', '<song_id>', '<url>',
  CAST(strftime('%s','now') AS INTEGER), 0);
```

### Link Show to Song
**HTTP:**
```bash
curl -X POST http://localhost:3000/rel_show_song \
  -H "Content-Type: application/json" \
  -d '{"show_id": "<show_id>", "song_id": "<song_id>", "media_url": "<url>"}'
```
**SQL:**
```sql
INSERT INTO rel_show_song (show_id, song_id, media_url, created_at)
VALUES ('<show_id>', '<song_id>', '<url>', CAST(strftime('%s','now') AS INTEGER));
```

## Update Records

### Reassign song to different artist
**HTTP:**
```bash
curl -X PATCH http://localhost:3000/song/<song_id> \
  -H "Content-Type: application/json" \
  -d '{"artist_id": "<new_artist_id>"}'
```
**SQL:**
```sql
UPDATE song SET artist_id='<new_artist_id>',
  updated_at=CAST(strftime('%s','now') AS INTEGER) WHERE id='<song_id>';
```

### Soft-delete artist (set status=1)
**HTTP:**
```bash
curl -X PATCH http://localhost:3000/artist/<id> \
  -H "Content-Type: application/json" \
  -d '{"status": 1}'
```
**SQL:**
```sql
UPDATE artist SET status=1, updated_at=CAST(strftime('%s','now') AS INTEGER) WHERE id='<id>';
```

## Bulk Reassign Songs

**HTTP (by song IDs):**
```bash
curl -X POST http://localhost:3000/song/bulk-reassign \
  -H "Content-Type: application/json" \
  -d '{"song_ids": ["<id1>","<id2>"], "new_artist_id": "<artist_id>"}'
```

**HTTP (by source artist):**
```bash
curl -X POST http://localhost:3000/song/bulk-reassign \
  -H "Content-Type: application/json" \
  -d '{"from_artist_id": "<old>", "to_artist_id": "<new>"}'
```

**SQL:**
```sql
UPDATE song SET artist_id='<new_artist_id>',
  updated_at=CAST(strftime('%s','now') AS INTEGER)
WHERE artist_id='<old_artist_id>';
```

## Delete Records

**HTTP:** `DELETE /artist/<id>` or `DELETE /song/<id>` (only artist and song allowed)

**SQL:**
```sql
DELETE FROM <table> WHERE id='<id>';
```

## Find Duplicates

**HTTP:** `GET /<table>/duplicates` (artist, show, song)

**SQL:**
```sql
SELECT LOWER(name) as lname, COUNT(*) as cnt
FROM <table> GROUP BY lname HAVING cnt > 1;
```

## Typical Merge Workflow (Duplicate Artists)

1. Find duplicates: `GET /artist/duplicates`
2. Review records for each duplicate name
3. Bulk reassign songs from duplicate → keeper: `POST /song/bulk-reassign`
4. Soft-delete or hard-delete the duplicate artist