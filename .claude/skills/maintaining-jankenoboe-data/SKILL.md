---
name: maintaining-jankenoboe-data
description: Create, update, and delete anime song data in Jankenoboe. Manages artists, shows, songs, play history, and show-song relationships. Handles bulk song reassignment, duplicate detection, and soft deletes. Use when the user wants to add, edit, delete, import, fix, merge, or reassign anime song records.
---

## Setup

The `jankenoboe` CLI must be installed.

### Database Path

Before running commands, check if `JANKENOBOE_DB` is already set:
```bash
echo $JANKENOBOE_DB
```

- **If it prints a path** (e.g., `/Users/you/db/datasource.db`): proceed directly with `jankenoboe` commands.
- **If it prints nothing (empty)**: ask the user for the database path, then either:
  - Export it for the session: `export JANKENOBOE_DB=/path/to/datasource.db`
  - Or prefix each command: `JANKENOBOE_DB=/path/to/datasource.db jankenoboe ...`

---

## Create Records

The CLI generates `id` (UUID) and timestamps (`created_at`, `updated_at`) automatically.

**URL Percent-Encoding:** All string values in `--data` JSON are automatically URL percent-decoded. Use `python3 tools/url_encode.py "<text>"` to encode values containing quotes, spaces, or shell-problematic characters (e.g., `Ado%27s` → `Ado's`, `%20` → space). Plain text without `%` works unchanged. Keys and non-string values (numbers, booleans) are not decoded.

### Artist
```bash
jankenoboe create artist --data '{"name": "ChoQMay"}'
```

### Show
```bash
jankenoboe create show --data '{"name": "A Sign of Affection", "name_romaji": "Yubisaki to Renren", "vintage": "Winter 2024", "s_type": "TV"}'
```

### Song
```bash
jankenoboe create song --data '{"name": "snowspring", "artist_id": "artist-uuid"}'
```

### Play History
```bash
jankenoboe create play_history --data '{"show_id": "show-uuid", "song_id": "song-uuid", "media_url": "https://..."}'
```

### Learning
```bash
jankenoboe create learning --data '{"song_id": "song-uuid", "level_up_path": "[1,1,1,1,1,1,1,2,3,5,7,13,19,32,52,84,135,220,355,574]"}'
```

### Link Show to Song
```bash
jankenoboe create rel_show_song --data '{"show_id": "show-uuid", "song_id": "song-uuid", "media_url": "https://..."}'
```

**Output:**
```json
{
  "id": "generated-uuid"
}
```

---

## Update Records

Only the provided fields are updated; `updated_at` is set automatically.

```bash
jankenoboe update <table> <id> --data '{"field": "value"}'
```

### Common update operations

**Reassign song to different artist:**
```bash
jankenoboe update song <song_id> --data '{"artist_id": "new-artist-id"}'
```

**Soft-delete artist (set status=1):**
```bash
jankenoboe update artist <id> --data '{"status": 1}'
```

**Level up a learning record:**
```bash
jankenoboe update learning <id> --data '{"level": 8}'
```
When `level` is changed, `last_level_up_at` is automatically updated to the current timestamp.

**Graduate a learning record:**
```bash
jankenoboe update learning <id> --data '{"graduated": 1}'
```

**Output:**
```json
{
  "updated": true
}
```

---

## Delete Records

Hard delete a record from the database.

```bash
jankenoboe delete <table> <id>
```

Allowed tables: `artist`, `song`

**Output:**
```json
{
  "deleted": true
}
```

---

## Bulk Reassign Songs

### By song IDs

```bash
jankenoboe bulk-reassign --song-ids song1,song2,song3 --new-artist-id correct-artist-id
```

### By source artist (move all songs from one artist to another)

```bash
jankenoboe bulk-reassign --from-artist-id artist-to-remove --to-artist-id artist-to-keep
```

**Output:**
```json
{
  "reassigned_count": 3
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

---

## Typical Merge Workflow (Duplicate Artists)

1. Find duplicates: `jankenoboe duplicates artist`
2. Review records: `jankenoboe search artist --fields id,name,name_context --term '{"name": {"value": "Minami", "match": "exact-i"}}'`
3. List songs for each: `jankenoboe search song --fields id,name --term '{"artist_id": {"value": "<artist_id>"}}'`
4. Bulk reassign songs from duplicate → keeper: `jankenoboe bulk-reassign --from-artist-id <dup_id> --to-artist-id <keeper_id>`
5. Delete or soft-delete the duplicate:
   - Soft-delete: `jankenoboe update artist <dup_id> --data '{"status": 1}'`
   - Hard-delete: `jankenoboe delete artist <dup_id>`