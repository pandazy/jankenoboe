---
name: importing-amq-songs
description: Import anime song data from animemusicquiz.com JSON exports into Jankenoboe. Resolves artists (with namesake disambiguation), shows (by name + vintage), songs (by name + artist), and creates play history records. Use when the user wants to import, load, or process an AMQ song export file.
---

## Setup

Follow the [initialize skill](../initialize/SKILL.md) to ensure the CLI is installed and `JANKENOBOE_DB` is set.

**Additional prerequisite:** Python 3 is required for the import script. macOS includes Python 3 by default; if not available, install via `brew install python3`.

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

Run the import script:

```bash
python3 .claude/skills/importing-amq-songs/scripts/import_amq.py <amq_export.json>
```

This script uses a two-phase approach:

1. **Phase 1 (Resolution):** Resolves all entities (artist, show, song) from the database. Entries are separated into "complete" (all three found) and "missing" (any entity not in DB) groups.
2. **Phase 2 (Processing):** For complete entries, automatically creates show-song links and play history records. Missing entries are skipped and reported.

After the run, a **Missing Entities Report** is printed with:
- **Deduplicated missing entities** — each missing artist, show, or song listed once with a ready-to-run `jankenoboe create` CLI command (shows always include `name_romaji` from the AMQ export)
- **Grouped by missing pattern** — entries grouped by what's missing, with resolved IDs so follow-up procedures can reuse them without re-fetching:
  - **missing artist, show, song** — nothing resolved, create all three
  - **missing show, song (artist resolved)** — `artist_id` provided; create show and song, then link
  - **missing artist, song (show resolved)** — `show_id` provided; create artist and song, then link
  - **missing song (artist and show resolved)** — `artist_id` and `show_id` provided; just create the song and link

Create the missing entities using the provided CLI commands (or manually), then re-run with `--missing-only` to process only the newly-resolved entries:

```bash
python3 .claude/skills/importing-amq-songs/scripts/import_amq.py --missing-only <amq_export.json>
```

The `--missing-only` flag skips entries where the show-song link already exists (i.e., they were successfully processed in a previous run), avoiding duplicate play_history records.

### Romaji Backfill

The import script automatically backfills `name_romaji` for existing shows that have an empty value, using the romaji name from the AMQ export.

---

## Namesake Conflict Resolution

When multiple artists share the same name, the script **pauses and prompts** the user to choose:

1. Each matching artist is listed with their ID and existing songs
2. The user sees context about which song/show is being resolved
3. Options:
   - **Select an existing artist** — pick by number based on song list context
   - **Create a NEW artist** — adds a new artist with the same name (legitimate namesake)
   - **Skip** — leaves the entry unresolved (reported in the missing entities report)

**Example prompt:**
```
  ⚠ Multiple artists named "Minami" found!
    (resolving: "Kawaki wo Ameku" from "Domestic Girlfriend")

    [1] Artist ID: 14b7393a
        Songs: Beautiful Soldier, One Unit, Patria, SWITCH

    [2] Artist ID: 6136d7b3
        Songs: Kawaki o Ameku, Rude Lose Dance

    [3] Create a NEW artist named "Minami"
    [4] Skip this entry

    Select [1-4]:
```

The user selects based on which anime the song is from. If neither artist is correct, option [3] creates a new one.

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

All string values in `--data` and `--term` JSON are automatically URL percent-decoded. To encode values containing quotes, spaces, or shell-problematic characters, use inline Python:

```bash
python3 -c "from urllib.parse import quote; print(quote('<text>', safe=''))"
```

For example, `Ado's` → `Ado%27s`, spaces → `%20`. Plain text without `%` works unchanged.

**This is especially important for song/artist/show names** that often contain special characters like `'`, `"`, `(`, `)`, `!`, `&`.

## Important Rules

- Artist matching is **case-sensitive exact** match
- Show matching uses **case-insensitive exact** match (`exact-i`) on `animeNames.english` + `vintage`
- Song matching uses **case-sensitive exact** match on `name` + `artist_id`
- When creating shows manually, **always include `name_romaji`** if available — the missing report provides it
- Process songs sequentially — earlier songs may create entities reused by later songs
