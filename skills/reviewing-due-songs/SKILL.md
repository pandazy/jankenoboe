---
name: reviewing-due-songs
description: Display anime songs that are due for spaced repetition review as a formatted list. Each entry shows the show name, song name, and media URLs from play history. Use when the user wants to see their due review list, practice queue, or review songs with playback links.
---

## Setup

**Always ask the user for the SQLite database file path first** (e.g., `datasource.db`). Store it for the session.

**⚠️ Shell safety for SQL mode:** Many SQL queries contain `%s` (in `strftime`) and `$[` (in `json_extract`) which zsh interprets as format specifiers and arithmetic expansion. **Always write SQL to a temp file and pipe it to sqlite3** instead of passing inline:
```bash
cat > /tmp/query.sql << 'EOSQL'
SELECT CAST(strftime('%s','now') AS INTEGER);
EOSQL
sqlite3 <db_path> < /tmp/query.sql
```
The `<< 'EOSQL'` (quoted heredoc) prevents all shell interpolation inside the SQL.

## Workflow

### Step 1: Query due songs with show names

```sql
SELECT
  l.id as learning_id,
  s.id as song_id,
  s.name as song_name,
  l.level + 1 as display_level,
  GROUP_CONCAT(DISTINCT sh.name) as show_names
FROM learning l
JOIN song s ON l.song_id = s.id
LEFT JOIN rel_show_song rss ON rss.song_id = s.id
LEFT JOIN show sh ON sh.id = rss.show_id
WHERE l.graduated = 0
  AND (
    (l.last_level_up_at > 0 AND l.level = 0
     AND CAST(strftime('%s','now') AS INTEGER) >= l.last_level_up_at + 300)
    OR (l.last_level_up_at = 0 AND l.level = 0
     AND CAST(strftime('%s','now') AS INTEGER) >= l.updated_at + 300)
    OR (l.level > 0
     AND json_extract(l.level_up_path,'$['||l.level||']') * 86400 + l.last_level_up_at
         <= CAST(strftime('%s','now') AS INTEGER))
  )
GROUP BY l.id
ORDER BY l.level DESC;
```

### Step 2: Query media URLs for each due song

For each `song_id` from Step 1, query distinct media URLs from play_history:

```sql
SELECT DISTINCT ph.media_url, sh.name as show_name
FROM play_history ph
JOIN show sh ON ph.show_id = sh.id
WHERE ph.song_id = '<song_id>'
  AND ph.media_url != ''
ORDER BY sh.name;
```

**Alternative — batch query all media URLs for all due songs at once:**

```sql
SELECT DISTINCT ph.song_id, ph.media_url, sh.name as show_name
FROM play_history ph
JOIN show sh ON ph.show_id = sh.id
WHERE ph.song_id IN (
  SELECT s.id
  FROM learning l
  JOIN song s ON l.song_id = s.id
  WHERE l.graduated = 0
    AND (
      (l.last_level_up_at > 0 AND l.level = 0
       AND CAST(strftime('%s','now') AS INTEGER) >= l.last_level_up_at + 300)
      OR (l.last_level_up_at = 0 AND l.level = 0
       AND CAST(strftime('%s','now') AS INTEGER) >= l.updated_at + 300)
      OR (l.level > 0
       AND json_extract(l.level_up_path,'$['||l.level||']') * 86400 + l.last_level_up_at
           <= CAST(strftime('%s','now') AS INTEGER))
    )
)
AND ph.media_url != ''
ORDER BY ph.song_id, sh.name;
```

## Output Format

Present the list as:

```
1. show: <show_name>, song: <song_name> (Lv.<display_level>)
   - <media_url_1> (<show_name_from_play_history>)
   - <media_url_2> (<show_name_from_play_history>)

2. show: <show_name>, song: <song_name> (Lv.<display_level>)
   - <media_url_1> (<show_name_from_play_history>)
```

- If a song appears in multiple shows (via `rel_show_song`), join show names with ` | ` in the header line.
- If no media URLs exist for a song, show `(no media URLs)` instead of the sublist.
- Media URLs from play_history may reference different shows than the song's `rel_show_song` entries (e.g., the song was encountered in a different show context during a quiz game). Always include the show name from play_history next to each URL for clarity.

## Notes

- The due condition uses a 5-minute (300 second) warm-up for level 0 songs and day-based intervals for higher levels.
- Levels are 0-indexed in the database but displayed as 1-indexed (level + 1).
- Media URLs come from `play_history`, which records actual song encounters during AMQ quiz games. Each play may have a different URL clip.
- A song may have zero, one, or many play_history entries with media URLs.