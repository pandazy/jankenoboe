---
name: querying-jankenoboe
description: Search and read anime song learning data from a Jankenoboe SQLite database. Finds artists, shows, songs, learning records, and play history. Use when the user asks to look up, search, find, or list anime songs, artists, shows, or learning progress. Supports both HTTP API (localhost:3000) and direct SQLite queries.
---

## Setup

**Always ask the user for the SQLite database file path first** (e.g., `datasource.db`). Store it for the session.

**Determine access mode:**
1. **HTTP mode**: Try `curl -s http://localhost:3000/artist/search?fields=id,name&name=test`. If it responds, use HTTP.
2. **SQL mode**: If the server is unreachable, use `sqlite3 <db_path>` directly.

**⚠️ Shell safety for SQL mode:** Many SQL queries contain `%s` (in `strftime`) and `$[` (in `json_extract`) which zsh interprets as format specifiers and arithmetic expansion. **Always write SQL to a temp file and pipe it to sqlite3** instead of passing inline:
```bash
cat > /tmp/query.sql << 'EOSQL'
SELECT CAST(strftime('%s','now') AS INTEGER);
EOSQL
sqlite3 <db_path> < /tmp/query.sql
```
The `<< 'EOSQL'` (quoted heredoc) prevents all shell interpolation inside the SQL.

## Tables

| Table | Key Fields |
|-------|-----------|
| artist | id, name, name_context, status |
| show | id, name, name_romaji, vintage, s_type, status |
| song | id, name, name_context, artist_id, status |
| learning | id, song_id, level, graduated, level_up_path, last_level_up_at |
| play_history | id, show_id, song_id, media_url |
| rel_show_song | show_id, song_id, media_url |

## Read by ID

**HTTP:**
```bash
curl "http://localhost:3000/<table>/<id>?fields=<comma-separated-fields>"
```

**SQL:**
```bash
sqlite3 <db_path> "SELECT <fields> FROM <table> WHERE id='<id>';"
```

## Search

### Artist by name
**HTTP:** `GET /artist/search?fields=id,name&name=<name>`
**SQL:** `SELECT <fields> FROM artist WHERE LOWER(name) = LOWER('<name>');`

### Show by name + vintage
**HTTP:** `GET /show/search?fields=id,name,vintage&name=<name>&vintage=<vintage>`
**SQL:** `SELECT <fields> FROM show WHERE LOWER(name) = LOWER('<name>') AND vintage='<vintage>';`

### Song by name + artist
**HTTP:** `GET /song/search?fields=id,name,artist_id&name=<name>&artist_id=<artist_id>`
**SQL:** `SELECT <fields> FROM song WHERE LOWER(name) = LOWER('<name>') AND artist_id='<artist_id>';`

### Songs by artist
**HTTP:** `GET /song/search?fields=id,name&artist_id=<artist_id>`
**SQL:** `SELECT <fields> FROM song WHERE artist_id='<artist_id>';`

## Learning Due for Review

**HTTP:** `GET /learning/due?limit=100`

**SQL:**
```sql
SELECT l.id, l.song_id, s.name as song_name, l.level,
  json_extract(l.level_up_path, '$[' || l.level || ']') as wait_days
FROM learning l
JOIN song s ON l.song_id = s.id
WHERE l.graduated = 0
  AND (
    (l.last_level_up_at > 0 AND l.level = 0
     AND CAST(strftime('%s', 'now') AS INTEGER) >= (l.last_level_up_at + 300))
    OR (l.last_level_up_at = 0 AND l.level = 0
     AND CAST(strftime('%s', 'now') AS INTEGER) >= (l.updated_at + 300))
    OR (l.level > 0
     AND (json_extract(l.level_up_path, '$[' || l.level || ']') * 86400 + l.last_level_up_at)
         <= CAST(strftime('%s', 'now') AS INTEGER))
  )
ORDER BY l.level DESC;
```

## Find Duplicates

**HTTP:** `GET /<table>/duplicates` (allowed: artist, show, song)

**SQL (example for artist):**
```sql
SELECT LOWER(name) as lname, COUNT(*) as cnt
FROM artist GROUP BY lname HAVING cnt > 1;
```
Then query each duplicate name for full records.

## Response Format

HTTP responses use: `{"results": [...]}` for reads/searches, `{"count": N, "results": [...]}` for due items, `{"duplicates": [...]}` for duplicates.

SQL mode: format results as readable tables or JSON for the user.