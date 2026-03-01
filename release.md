## v2.5.0

### Import: Romaji Name Backfill for Shows

The AMQ import script now automatically fills in missing `name_romaji` for existing shows when the import JSON provides a romaji name. The skill documentation also emphasizes always including `name_romaji` when manually creating new shows.

### Learning Song Stats: Play History Count

`learning-song-stats` now includes a `play_count` field showing the total number of play_history records for each song, giving a quick view of how often a song has been encountered in quizzes.

### Due Review HTML: Learning ID, Song ID, and Show ID with Copy Buttons

Each song card in the due review HTML report now displays the learning record ID, song ID, and show ID(s) with one-click copy buttons, making it easy to reference these IDs for quick data lookups or CLI commands.

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/install.sh | sh
```

## Platform Support

Pre-built binaries for:
- Linux x86_64
- Linux ARM64 (aarch64)
- macOS Apple Silicon (aarch64)
