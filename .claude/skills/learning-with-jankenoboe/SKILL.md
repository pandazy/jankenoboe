---
name: learning-with-jankenoboe
description: Manage anime song spaced repetition learning via Jankenoboe. Add songs to learning, level up/down, graduate, check due reviews, and re-learn graduated songs. Use when the user wants to practice, review, level up, or manage their anime song learning progress.
---

## Setup

The `jankenoboe` CLI must be installed. Set the `JANKENOBOE_DB` environment variable to the SQLite database path:
```bash
export JANKENOBOE_DB=~/db/datasource.db
```

**Always ask the user for the database path first** if `JANKENOBOE_DB` is not already set.

## Spaced Repetition Overview

Songs progress through 20 levels (stored 0–19, displayed 1–20). Each level has a wait period before the next review. The `level_up_path` JSON array stores wait-days per level.

Default path: `[1,1,1,1,1,1,1,2,3,5,7,13,19,32,52,84,135,220,355,574]`

- **Level 0**: 5-minute warm-up (300 seconds)
- **Levels 1–6**: 1-day intervals
- **Levels 7+**: Fibonacci-based growth up to 574 days
- **Graduation**: `graduated = 1` when fully memorized (after level 19)
- **Re-learn**: Graduated songs restart at level 7 by default

---

## Check Due Reviews

```bash
jankenoboe learning-due
jankenoboe learning-due --limit 20
jankenoboe learning-due --offset 7200          # due within the next 2 hours
jankenoboe learning-due --offset 120 --limit 50 # due within the next 2 minutes
```

- `--offset`: Look-ahead in seconds. Shifts the reference time forward so you can see songs that will be due soon. Default `0` = now only.

**Output:**
```json
{
  "count": 13,
  "results": [
    {
      "id": "...",
      "song_id": "...",
      "song_name": "1-nichi wa 25-jikan.",
      "level": 16,
      "wait_days": 135
    }
  ]
}
```

---

## Add Songs to Learning (Batch)

```bash
jankenoboe learning-batch --song-ids song-uuid-1,song-uuid-2,song-uuid-3
```

**Output:**
```json
{
  "created_ids": ["generated-learning-uuid-1", "generated-learning-uuid-2"],
  "skipped_song_ids": ["song-uuid-3"],
  "already_graduated_song_ids": ["song-uuid-4"]
}
```

- `created_ids`: UUIDs of newly created learning records
- `skipped_song_ids`: songs skipped because they already have an active (non-graduated) learning record
- `already_graduated_song_ids`: songs with graduated records not included in `--relearn-song-ids`

---

## Re-learn Graduated Songs

Graduated songs require explicit confirmation via `--relearn-song-ids`:

```bash
# Step 1: Attempt to add (some may be graduated)
jankenoboe learning-batch --song-ids song-new,song-graduated
# Output shows already_graduated_song_ids: ["song-graduated"]

# Step 2: Confirm re-learn
jankenoboe learning-batch --song-ids song-graduated --relearn-song-ids song-graduated

# Step 2 (alternative): Re-learn with custom start level
jankenoboe learning-batch --song-ids song-graduated --relearn-song-ids song-graduated --relearn-start-level 5
```

- Default re-learn start level: `7` (stored 0-indexed)
- Old graduated record is preserved

---

## Level Up

After a successful review, increment the level. The `last_level_up_at` is automatically updated to the current timestamp.

```bash
jankenoboe update learning <learning_id> --data '{"level": 8}'
```

**Output:**
```json
{
  "updated": true
}
```

---

## Level Down

If a song is forgotten, set a lower level. The `last_level_up_at` is also updated.

```bash
jankenoboe update learning <learning_id> --data '{"level": 3}'
```

---

## Graduate

When a song at level 19 is reviewed correctly, mark it as graduated:

```bash
jankenoboe update learning <learning_id> --data '{"graduated": 1}'
```

---

## Batch Level Up by IDs (Race-Condition Safe)

Level up specific learning records by their IDs. Use this after generating a review report to ensure only the reviewed songs are leveled up.

```bash
jankenoboe learning-song-levelup-ids --ids learning-uuid-1,learning-uuid-2
```

**Output:**
```json
{
  "leveled_up_count": 1,
  "graduated_count": 1,
  "total_processed": 2
}
```

- Does not require songs to be due — works regardless of due status
- Songs at max level (19) are automatically graduated
- Rejects already-graduated records with an error

---

## Typical Review Workflow

1. Generate review report: `jankenoboe learning-song-review`
   - Output includes `learning_ids` array
2. Present report to user for review
3. Upon confirmation, level up the reviewed songs:
   ```bash
   ids=$(echo "$review_output" | jq -r '.learning_ids | join(",")')
   jankenoboe learning-song-levelup-ids --ids "$ids"
   ```
4. For individual corrections:
   - **Forgotten** → level down: `jankenoboe update learning <id> --data '{"level": <lower_level>}'`
   - **Level 19 + correct** → graduate: `jankenoboe update learning <id> --data '{"graduated": 1}'`
