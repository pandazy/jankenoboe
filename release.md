# v2.4.4 — Learning Song Stats

## New Command: `learning-song-stats`

Get learning statistics per song — how many days have been spent learning each song.

```bash
jankenoboe learning-song-stats --song-ids song-uuid-1,song-uuid-2
```

**Output:**
```json
{
  "count": 1,
  "results": [
    {
      "song_id": "song-uuid-1",
      "song_name": "Crossing Field",
      "earliest_created_at": 1700000000,
      "latest_last_level_up_at": 1700864000,
      "days_spent": 10
    }
  ]
}
```

- `days_spent` is the absolute gap in days between the earliest learning record creation and the most recent level-up
- Groups all learning records per song (including graduated and re-learn records)
- Ordered by `days_spent` descending

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/install.sh | sh
```

## Platform Support

Pre-built binaries for:
- Linux x86_64
- Linux ARM64 (aarch64)
- macOS Apple Silicon (aarch64)