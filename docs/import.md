# AMQ Song Import Workflow

## Overview

This document describes the concepts and rules for importing song data from [animemusicquiz.com](https://animemusicquiz.com) exports into the Jankenoboe database.

For step-by-step CLI commands, see the [importing-amq-songs skill](../skills/importing-amq-songs/SKILL.md).

## Export Format

AMQ exports JSON files with guessed song records. See `docs/amq_song_export-sample.json` for an example.

**Key fields per song:**
```json
{
  "songInfo": {
    "animeNames": {
      "english": "A Sign of Affection",
      "romaji": "Yubisaki to Renren"
    },
    "artist": "ChoQMay",
    "songName": "snowspring",
    "vintage": "Winter 2024",
    "animeType": "TV",
    "videoUrl": "https://nawdist.animemusicquiz.com/dygyly.webm"
  }
}
```

**Field mapping:**

| AMQ Field | Maps To |
|-----------|---------|
| `artist` | artist `name` |
| `songName` | song `name` |
| `animeNames.english` | show `name` |
| `animeNames.romaji` | show `name_romaji` |
| `vintage` | show `vintage` |
| `animeType` | show `s_type` |
| `videoUrl` | play_history `media_url` |

## Entity Matching Rules

### Show (Anime)

**Matching criteria:** `name` + `vintage` (season)

- AMQ exports provide both `animeNames.english` and `animeNames.romaji` for each show
- Always compare AMQ's `animeNames.english` with our `name` field for primary matching
- The `romaji` name is optional supplementary info — stored in `name_romaji` when creating a show, but not used as the primary search criterion
- A show is uniquely identified by its English name combined with its vintage (season)
- Example: "K-On!" (Spring 2009) and "K-On!" (Spring 2010) are different shows (Season 1 vs Season 2)

### Artist

**Matching criteria:** `name` (case-sensitive exact)

⚠️ **Namesake Conflict:** Artists with the same name may be different people. When multiple artists share the same name, the user must confirm which artist to use by reviewing their existing song lists.

### Song

**Matching criteria:** `name` + `artist_id` (case-sensitive exact)

- Two songs with the same name and the same artist are considered the same song
- The artist must be resolved first before song matching can occur

### Show–Song Link (`rel_show_song`)

**Matching criteria:** `show_id` + `song_id`

- The `rel_show_song` table has **no `id` column** — it uses a composite unique constraint on `(show_id, song_id)`
- Available fields: `show_id`, `song_id`, `media_url`, `created_at`

## Import Processing Steps

For each song in the export, process sequentially through these steps. Earlier songs may create entities reused by later songs.

1. **Resolve Artist** — Search by name. Handle not-found, single match, or namesake conflicts.
2. **Resolve Show** — Search by English name (case-insensitive) + vintage. Create if missing; update casing if it differs.
3. **Resolve Song** — Search by name + resolved artist ID. Create if missing.
4. **Link Show to Song** — Check if the show–song relationship exists. Create if missing.
5. **Create Play History** — Only after all entities exist and are linked.

All create operations require user confirmation before executing.

## Import Flow Diagram

```
┌─────────────────────────────────────────────────────────┐
│                   For each song in export               │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
                ┌───────────────────────┐
                │   Lookup artist by    │
                │        name           │
                └───────────────────────┘
                            │
           ┌────────────────┼────────────────┐
           ▼                ▼                ▼
      Not found      Single match      Multiple matches
           │                │                │
           ▼                │                ▼
    ┌────────────┐          │         ┌────────────┐
    │  CONFIRM:  │          │         │  CONFIRM:  │
    │ Add artist │          │         │   Select   │
    └────────────┘          │         │   artist   │
           │                │         └────────────┘
           └────────────────┼────────────────┘
                            │
                            ▼
                ┌───────────────────────┐
                │  Lookup show by       │
                │  name + vintage       │
                └───────────────────────┘
                            │
              ┌─────────────┴─────────────┐
              ▼                           ▼
         Not found                   Found
              │                           │
              ▼                           │
       ┌────────────┐                     │
       │  CONFIRM:  │                     │
       │  Add show  │                     │
       └────────────┘                     │
              │                           │
              └─────────────┬─────────────┘
                            │
                            ▼
                ┌───────────────────────┐
                │  Lookup song by       │
                │  name + artist_id     │
                └───────────────────────┘
                            │
              ┌─────────────┴─────────────┐
              ▼                           ▼
         Not found                   Found
              │                           │
              ▼                           │
       ┌────────────┐                     │
       │  CONFIRM:  │                     │
       │  Add song  │                     │
       └────────────┘                     │
              │                           │
              └─────────────┬─────────────┘
                            │
                            ▼
                ┌───────────────────────┐
                │  Check rel_show_song  │
                │  (show_id + song_id)  │
                └───────────────────────┘
                            │
              ┌─────────────┴─────────────┐
              ▼                           ▼
        Not linked                   Linked
              │                           │
              ▼                           │
       ┌────────────┐                     │
       │  CONFIRM:  │                     │
       │  Link song │                     │
       │  to show   │                     │
       └────────────┘                     │
              │                           │
              └─────────────┬─────────────┘
                            │
                            ▼
                ┌───────────────────────┐
                │  Add play_history     │
                │  record               │
                └───────────────────────┘
```

## Conflict Resolution

### Namesake Artists

When two artists share the same name but are different people:

1. Display both artists with their existing song lists
2. User selects the correct artist based on context (which anime the song is from)
3. If neither is correct, create a new artist

**Example scenario:**
- Artist "Minami" appears in export
- Database has two "Minami" entries (both are real singers with the same name):
  - Minami (ID: 14b7393a) — songs: "Beautiful Soldier", "One Unit", "Patria", "SWITCH", "illuminate"
  - Minami (ID: 6136d7b3) — songs: "Kawaki o Ameku", "Rude Lose Dance", "illuminate"
- User reviews and selects the appropriate one based on which anime the song is from

> Note: Both entries above are legitimate different artists who happen to share the same name. This is not a data quality issue — they should remain separate.

### Song Reassignment

To fix import mistakes where songs were assigned to the wrong namesake artist:

1. Identify the incorrectly assigned songs
2. Create or select the correct artist
3. Update the song's `artist_id` to point to the correct artist (single or bulk reassignment)

### Artist Merging

When two artists are confirmed to be the same person:

1. Identify duplicates via duplicate detection
2. Review song lists for each duplicate
3. Bulk reassign all songs from the secondary artist to the primary
4. Optionally delete or soft-delete the secondary artist

## Data Quality

### Duplicate Detection

Use the `duplicates` command to find potential duplicates across artists, shows, and songs using case-insensitive name matching. See [CLI Reference — duplicates](cli-querying.md#jankenoboe-duplicates-table) for details.

### Duplicate Prevention

When adding new records, the importing agent should:

1. Perform case-insensitive search before creating
2. Return existing matches for review
3. Allow user to select an existing record or confirm new creation

Batch confirmations are acceptable (e.g., "I'll create these 5 new artists: ...").