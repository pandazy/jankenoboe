# v2.4.0 — New querying commands and review UX improvement

## New Commands

### `learning-by-song-ids` — Look up learning records by song IDs
- Returns all learning records (active and graduated) for given song IDs
- Usage: `jankenoboe learning-by-song-ids --song-ids <comma-separated-uuids>`
- Output includes `song_name`, `level`, `graduated`, `last_level_up_at`, `wait_days`
- Results ordered by level descending; songs with no learning records are silently omitted

### `shows-by-artist-ids` — Get all shows where given artists have songs
- Traverses the `artist → song → rel_show_song → show` relationship chain
- Usage: `jankenoboe shows-by-artist-ids --artist-ids <comma-separated-uuids>`
- Output includes `show_id`, `show_name`, `vintage`, `song_id`, `song_name`, `artist_id`, `artist_name`
- Results ordered by artist name, show name, song name

### `songs-by-artist-ids` — Get all songs by given artists
- Usage: `jankenoboe songs-by-artist-ids --artist-ids <comma-separated-uuids>`
- Output includes `song_id`, `song_name`, `artist_id`, `artist_name`
- Results ordered by artist name, then song name

## Improvements

### Review report: click-to-toggle reviewed songs
- Clicking a song card in the `learning-song-review` HTML report toggles it to a green "reviewed" color scheme
- Clicking again toggles it back to the default dark blue/red scheme
- Reviewed state persists across page navigation within the same session

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/install.sh | sh
```

## Platform Support

Pre-built binaries for:
- Linux x86_64
- Linux ARM64 (aarch64)
- macOS Apple Silicon (aarch64)