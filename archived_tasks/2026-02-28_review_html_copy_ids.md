# Due Report HTML: Learning ID, Song ID, and Show ID with Copy Buttons

**Date:** 2026-02-28

## Summary

Added learning record ID, song ID, and show IDs to each song card in the due review HTML report, with copy-to-clipboard buttons for quick data reference.

## Changes

### Rust Backend (`src/commands/learning.rs`)
- Added `learning_id`, `song_id`, and `show_ids` fields to `EnrichedSong` struct
- Added `learning_id()`, `song_id()`, and `show_ids()` methods to `SongReviewData` trait
- `cmd_learning_song_review`: Now collects `show_id` values from `get_show_info` query results and passes `learning_id` (from `l.id`) and `song_id` into each enriched song
- `build_review_html`: Now includes `learningId` (string), `songId` (string), and `showIds` (array of strings) in the songs JSON data

### HTML Template (`templates/learning-song-review.html`)
- Added CSS styles for `.ids-row`, `.id-label`, `.copy-btn`, and `.copy-btn.copied`
- Added `copyId(btn, text)` JS function using `navigator.clipboard.writeText()` with visual ✓ feedback
- Added `renderIdsRow(s)` JS function that renders learning ID, song ID, and show ID(s) with ⎘ copy buttons
- Copy buttons use `event.stopPropagation()` to avoid triggering the card's click-to-toggle-reviewed behavior
- Each song card now includes an IDs row at the bottom showing learning ID, song ID, and show ID(s)

### Unit Tests (`src/commands/learning.rs` tests)
- Updated `TestSong` struct with `learning_id`, `song_id`, and `show_ids` fields
- Updated `SongReviewData` impl for `TestSong` with new trait methods
- Updated all 4 `build_review_html` test constructors with new fields

## Files Modified
- `src/commands/learning.rs`
- `templates/learning-song-review.html`
