---
name: importing-amq-songs
description: Import anime song data from animemusicquiz.com JSON exports into Jankenoboe. Resolves artists (with namesake disambiguation), shows (by name + vintage), songs (by name + artist), and creates play history records. Use when the user wants to import, load, or process an AMQ song export file.
---

## Setup

The `jankenoboe` CLI must be installed. Set the `JANKENOBOE_DB` environment variable to the SQLite database path:
```bash
export JANKENOBOE_DB=~/db/datasource.db
```

**Always ask the user for the database path first** if `JANKENOBOE_DB` is not already set.

---

## Export Format

AMQ exports are JSON files. Each song entry lives under `songs[].songInfo` with these key fields:

| AMQ Field | Maps To |
|-----------|---------|
| `artist` | artist `name` |
| `songName` | song `name` |
| `animeNames.english` | show `name` |
| `animeNames.romaji` | show `name_romaji` |
| `vintage` | show `vintage` |
| `animeType` | show `s_type` |
| `videoUrl` | play_history `media_url` |

---

## Import Workflow

### Step 1: Analyze and Verify

Parse the AMQ export to see a summary of all songs, artists, and shows:

```bash
python3 skills/importing-amq-songs/parse_amq_import.py <amq_export.json>
```

Check which artists and shows already exist in the database:

```bash
# Check artists
bash skills/importing-amq-songs/check_artists.sh <amq_export.json>

# Check shows
bash skills/importing-amq-songs/check_shows.sh <amq_export.json>
```

Both scripts dynamically extract unique artists/shows from the AMQ export JSON, check each against the database, and print a summary of found/not found counts.

### Step 2: Import Songs

Choose **one** of the following approaches:

#### Option A: Automated Import

For batch imports where all artists and shows already exist, use the automated import script:

```bash
python3 skills/importing-amq-songs/import_amq.py <amq_export.json>
```

This script processes all songs sequentially — resolving artists, shows, songs, creating links and play history. It requires all artists and shows to already exist in the database (use Step 1 to verify first). Songs and show-song links are created automatically if missing.

#### Option B: Manual Import

Process each song in the export manually. For each song:

##### Resolve Artist

```bash
jankenoboe search artist --fields id,name --term '{"name": {"value": "<artist>", "match": "exact"}}'
```

- **Not found** → CONFIRM with user, then: `jankenoboe create artist --data '{"name": "<artist>"}'`
- **Single match** → Use the matched artist's `id`
- **Multiple matches** (namesakes) → List songs for each to help user choose:
  ```bash
  jankenoboe search song --fields id,name --term '{"artist_id": {"value": "<artist-id>"}}'
  ```
  Wait for user to select the correct artist, or confirm creating a new one.

##### Resolve Show

Use `animeNames.english` as the `name` value (case-insensitive match):

```bash
jankenoboe search show --fields id,name,vintage --term '{"name": {"value": "<english-name>", "match": "exact-i"}, "vintage": {"value": "<vintage>"}}'
```

- **Not found** → CONFIRM with user, then create (store english as `name`, romaji as `name_romaji`):
  ```bash
  jankenoboe create show --data '{"name": "<english-name>", "name_romaji": "<romaji>", "vintage": "<vintage>", "s_type": "<animeType>"}'
  ```
- **Found** → Use the matched show's `id`. If stored `name` differs in casing from AMQ's English name, update:
  ```bash
  jankenoboe update show <show-id> --data '{"name": "<english-name>"}'
  ```

##### Resolve Song

```bash
jankenoboe search song --fields id,name,artist_id --term '{"name": {"value": "<songName>", "match": "exact"}, "artist_id": {"value": "<resolved-artist-id>"}}'
```

- **Not found** → CONFIRM with user, then: `jankenoboe create song --data '{"name": "<songName>", "artist_id": "<resolved-artist-id>"}'`
- **Found** → Use the matched song's `id`

##### Link Show to Song

> ⚠️ The `rel_show_song` table has **no `id` column**. Do NOT include `id` in `--fields`. Use `show_id`, `song_id`, `media_url`, or `created_at`.

```bash
jankenoboe search rel_show_song --fields show_id,song_id,media_url --term '{"show_id": {"value": "<show-id>"}, "song_id": {"value": "<song-id>"}}'
```

- **Not linked** → CONFIRM with user, then: `jankenoboe create rel_show_song --data '{"show_id": "<show-id>", "song_id": "<song-id>"}'`
- **Already linked** → No action needed

##### Create Play History

Only after artist, show, song all exist AND show–song are linked:

```bash
jankenoboe create play_history --data '{"show_id": "<show-id>", "song_id": "<song-id>", "media_url": "<videoUrl>"}'
```

---

## Namesake Conflict Resolution

When multiple artists share the same name:

1. List songs for each artist:
   ```bash
   jankenoboe search song --fields id,name --term '{"artist_id": {"value": "<artist-id>"}}'
   ```
2. Present both lists to user with artist IDs
3. User selects the correct artist based on which anime the song is from
4. If neither is correct, create a new artist

---

## Post-Import Data Quality

### Find duplicates
```bash
jankenoboe duplicates artist
jankenoboe duplicates show
jankenoboe duplicates song
```

### Fix wrong artist assignment
```bash
jankenoboe update song <song-id> --data '{"artist_id": "<correct-artist-id>"}'
```

### Bulk reassign songs
```bash
jankenoboe bulk-reassign --song-ids song1,song2 --new-artist-id <correct-artist-id>
jankenoboe bulk-reassign --from-artist-id <wrong-id> --to-artist-id <correct-id>
```

---

## URL Percent-Encoding

All string values in `--data` and `--term` JSON are automatically URL percent-decoded. Use `python3 tools/url_encode.py "<text>"` to encode values containing quotes, spaces, or shell-problematic characters (e.g., `Ado%27s` → `Ado's`, `%20` → space). Plain text without `%` works unchanged.

**This is especially important for song/artist/show names** that often contain special characters like `'`, `"`, `(`, `)`, `!`, `&`.

## Important Rules

- Always **CONFIRM with the user** before creating new artists, shows, songs, or links
- Artist matching is **case-sensitive exact** match
- Show matching uses **case-insensitive exact** match (`exact-i`) on `animeNames.english` + `vintage`
- Song matching uses **case-sensitive exact** match on `name` + `artist_id`
- Process songs sequentially — earlier songs may create entities reused by later songs
- Batch confirmations are acceptable (e.g., "I'll create these 5 new artists: ...")