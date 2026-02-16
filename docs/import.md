# AMQ Song Import Workflow

## Overview

This document describes the workflow for importing song data from [animemusicquiz.com](https://animemusicquiz.com) exports into the Jankenoboe database.

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

## Entity Matching Rules

### Show (Anime)

**Matching criteria:** `name` + `vintage` (season)

- A show is uniquely identified by its English name (or romaji) combined with its season
- Example: "K-On!" (Spring 2009) and "K-On!" (Spring 2010) are different shows (Season 1 vs Season 2)

### Artist

**Matching criteria:** `name`

⚠️ **Namesake Conflict:** Artists with the same name may be different people. When multiple artists share the same name, the user must confirm which artist to use by reviewing their existing song lists.

### Song

**Matching criteria:** `name` + `artist_id`

- Two songs with the same name and the same artist are considered the same song
- The artist must be resolved first before song matching can occur

## Import Processing Rules

For each song in the export:

### Step 1: Check Artist
```
IF artist (by name) does not exist:
    → CONFIRM: Add new artist to database
    → Wait for user confirmation before proceeding
ELSE IF multiple artists with same name exist:
    → CONFIRM: Select correct artist by reviewing song lists
    → Wait for user selection before proceeding
```

### Step 2: Check Show (Anime)
```
IF show (by name + vintage) does not exist:
    → CONFIRM: Add new show to database
    → Wait for user confirmation before proceeding
```

### Step 3: Check Song
```
IF song (by name + resolved artist_id) does not exist:
    → CONFIRM: Add new song to database
    → Wait for user confirmation before proceeding
```

### Step 4: Create Play History
```
IF artist, show, and song all exist:
    → Add record to play_history with:
        - show_id
        - song_id
        - created_at (current timestamp)
        - media_url (from videoUrl)
```

## Conflict Resolution

### Namesake Artists

When two artists share the same name but are different people:

1. Display both artists with their existing song lists
2. User selects the correct artist based on context
3. If neither is correct, create a new artist

**Example scenario:**
- Artist "Minami" appears in export
- Database has two "Minami" entries (both are real singers with the same name):
  - Minami (ID: 14b7393a) - songs: "Beautiful Soldier", "One Unit", "Patria", "SWITCH", "illuminate"
  - Minami (ID: 6136d7b3) - songs: "Kawaki o Ameku", "Rude Lose Dance", "illuminate"
- User reviews and selects the appropriate one based on which anime the song is from

### Song Reassignment

To fix import mistakes where songs were assigned to the wrong namesake artist:

1. Identify the incorrectly assigned songs
2. Create or select the correct artist
3. Update the song's `artist_id` to point to the correct artist

**API Support Needed:**
- List songs by artist
- Update song's artist_id
- Merge duplicate artists (optional, advanced)

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
                │  Add play_history     │
                │  record               │
                └───────────────────────┘
```

## Database Operations Required

### For Import
- `GET /artist/search?name=X` - Find artist by name (case-insensitive)
- `GET /show/search?name=X&vintage=Y` - Find show by name and vintage
- `GET /song/search?name=X&artist_id=Y` - Find song by name and artist
- `POST /artist` - Create new artist
- `POST /show` - Create new show
- `POST /song` - Create new song
- `POST /play_history` - Create play history record
- `POST /rel_show_song` - Link song to show

### For Conflict Resolution
- `GET /song/search?artist_id=X` - List songs by artist (for namesake disambiguation)
- `PATCH /song/:id` - Update song's artist_id (for fixing mistakes)

## Data Quality Management

### Duplicate Detection

Find potential duplicates using case-insensitive name matching:

**API Operations:**
- `GET /artist/duplicates` - Find artists with case-insensitive matching names
- `GET /show/duplicates` - Find shows with case-insensitive matching names
- `GET /song/duplicates` - Find songs with case-insensitive matching names

All duplicate endpoints use the generic `GET /:table/duplicates` pattern (see [API Reference](api.md#7-get-tableduplicates)).

**Example Response:**
```json
{
  "duplicates": [
    {
      "name": "minami",
      "records": [
        {"id": "14b7393a-0c01-4c7c-a694-84a83782908f", "name": "Minami", "song_count": 5},
        {"id": "6136d7b3-49d2-4747-b970-96f322286c47", "name": "Minami", "song_count": 3}
      ]
    }
  ]
}
```

Note: Both "Minami" entries above are legitimate different artists who happen to share the same name. This is not a data quality issue—they should remain separate. The duplicate detection helps identify such cases for review.

### Song Ownership Transfer

When duplicate artists are found or import mistakes occur, songs can be reassigned:

**API Operations:**
- `PATCH /song/:id` - Update a single song's `artist_id`
- `POST /song/bulk-reassign` - Reassign multiple songs to a new artist

**Single Song Reassignment:**
```bash
curl -X PATCH "http://localhost:3000/song/abc123" \
  -H "Content-Type: application/json" \
  -d '{"artist_id": "new-artist-id"}'
```

**Bulk Reassignment:**
```bash
curl -X POST "http://localhost:3000/song/bulk-reassign" \
  -H "Content-Type: application/json" \
  -d '{
    "song_ids": ["song1", "song2", "song3"],
    "new_artist_id": "correct-artist-id"
  }'
```

### Artist Merging Workflow

When two artists are confirmed to be the same person:

1. **Identify duplicates:**
   ```
   GET /artist/duplicates
   ```

2. **Review song lists for each duplicate:**
   ```
   GET /song/search?fields=id,name&artist_id=artist1
   GET /song/search?fields=id,name&artist_id=artist2
   ```

3. **Reassign all songs from secondary artist to primary:**
   ```
   POST /song/bulk-reassign
   {
     "from_artist_id": "artist-to-remove",
     "to_artist_id": "artist-to-keep"
   }
   ```

4. **Optionally delete or mark the secondary artist as deleted:**
   ```
   DELETE /artist/artist-to-remove
   -- or --
   PATCH /artist/artist-to-remove {"status": 1}
   ```

### Duplicate Prevention

When adding new records, the system should:

1. Perform case-insensitive search before creating
2. Return existing matches with similarity scores
3. Allow user to select existing record or confirm new creation

**Example warning response on artist creation (creating "Minami"):**
```json
{
  "warning": "Similar artists found",
  "matches": [
    {"id": "14b7393a-0c01-4c7c-a694-84a83782908f", "name": "Minami", "song_count": 5},
    {"id": "6136d7b3-49d2-4747-b970-96f322286c47", "name": "Minami", "song_count": 3}
  ],
  "action_required": "Select existing artist or confirm creation of new artist",
  "confirm_url": "/artist?confirm=true"
}
```
