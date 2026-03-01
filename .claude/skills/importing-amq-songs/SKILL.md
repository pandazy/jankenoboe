---
name: importing-amq-songs
description: Import anime song data from animemusicquiz.com JSON exports into Jankenoboe. Resolves artists (with namesake disambiguation), shows (by name + vintage), songs (by name + artist), and creates play history records. Use when the user wants to import, load, or process an AMQ song export file.
---

## Setup

### Prerequisites

- **`jankenoboe` CLI** must be installed (see [README.md](../../../README.md))
- **Python 3** is required for the import scripts. macOS includes Python 3 by default; if not available, install via `brew install python3`

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
python3 .claude/skills/importing-amq-songs/scripts/parse_amq_import.py <amq_export.json>
```

Check which artists and shows already exist in the database:

```bash
# Check artists
bash .claude/skills/importing-amq-songs/scripts/check_artists.sh <amq_export.json>

# Check shows
bash .claude/skills/importing-amq-songs/scripts/check_shows.sh <amq_export.json>
```

Both scripts dynamically extract unique artists/shows from the AMQ export JSON, check each against the database, and print a summary of found/not found counts.

### Step 2: Import Songs

Choose **one** of the following approaches:

#### Option A: Automated Import

For batch imports, use the automated import script:

```bash
python3 .claude/skills/importing-amq-songs/scripts/import_amq.py <amq_export.json>
```

This script uses a two-phase approach:

1. **Phase 1 (Resolution):** Resolves all entities (artist, show, song) from the database. Entries are separated into "complete" (all three found) and "missing" (any entity not in DB) groups.
2. **Phase 2 (Processing):** For complete entries, automatically creates show-song links and play history records. Missing entries are skipped and reported.

After the run, a **Missing Entities Report** is printed listing all unresolved artists, shows, and songs (deduplicated and grouped). Create the missing entities manually (or use Option B below), then re-run with `--missing-only` to process only the newly-resolved entries:

```bash
python3 .claude/skills/importing-amq-songs/scripts/import_amq.py --missing-only <amq_export.json>
```

The `--missing-only` flag skips entries where the show-song link already exists (i.e., they were successfully processed in a previous run), avoiding duplicate play_history records.

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

- **Not found** → CONFIRM with user, then create. **Always include `name_romaji`** if the AMQ export provides `animeNames.romaji`:
  ```bash
  jankenoboe create show --data '{"name": "<english-name>", "name_romaji": "<romaji>", "vintage": "<vintage>", "s_type": "<animeType>"}'
  ```
  > ⚠️ Do NOT omit `name_romaji` — it is important for search and display. If the AMQ export has a romaji name, always include it when creating a new show.
- **Found** → Use the matched show's `id`. If stored `name` differs in casing from AMQ's English name, update:
  ```bash
  jankenoboe update show <show-id> --data '{"name": "<english-name>"}'
  ```
  Also, if the existing show has an empty `name_romaji` and the AMQ export provides one, backfill it:
  ```bash
  jankenoboe update show <show-id> --data '{"name_romaji": "<romaji>"}'
  ```
  > The automated import script (`import_amq.py`) does this romaji backfill automatically. When importing manually, remember to check and fill it yourself.

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

> ⚠️ Do NOT check for duplicates — always create a new play_history record. Duplicate play history is acceptable (same song played multiple times).

After artist, show, song exist AND show–song are linked:

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