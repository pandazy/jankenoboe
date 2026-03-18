# Review Report: Play History Shows with Grouped Media URLs and Vintage

**Date:** 2026-03-17
**Status:** Completed

## Summary

Changed the `learning-song-review` HTML report to source show data and media URLs directly from `play_history` instead of `rel_show_song`. Shows are now grouped per song with their media URLs deduplicated within each show. Added vintage display next to each show name.

## Changes

### SQL Query (`get_show_media`)
- **Before:** Two separate queries — `get_show_info` (from `rel_show_song` → `show`) and `get_play_history_urls` (from `play_history`)
- **After:** Single `get_show_media` query joining `play_history` → `show`, returning `show_id`, `show_name`, `vintage`, `media_url`

### Rust (`src/commands/learning.rs`)
- Replaced `get_show_info` + `get_play_history_urls` queries with single `get_show_media` query
- Added `ShowMedia` struct with fields: `show_id`, `show_name`, `vintage`, `media_urls`
- Enrichment loop groups play_history rows by `show_id`, deduplicating media URLs within each show
- `EnrichedSong` now holds `Vec<ShowMedia>` instead of flat arrays
- `SongReviewData` trait updated with `shows()` method
- `build_review_html` outputs `shows` array with `showId`, `showName`, `vintage`, `mediaUrls` per show
- Added `test_build_review_html_multiple_shows_with_grouped_urls` unit test
- Updated all existing unit tests with `vintage` field

### HTML Template (`templates/learning-song-review.html`)
- Replaced `renderMediaHtml()` with `renderShowsHtml()` that iterates the `shows` array
- Each show renders as a bordered `.show-group` with show name, vintage in parentheses, copy-show-ID button, and its media links
- Added `.show-group` CSS class

### Integration Tests (`tests/test_learning.rs`)
- Updated `test_learning_song_review_generates_html` and `test_learning_song_review_deduplicates_media_urls` to use `play_history` inserts instead of `rel_show_song`

### E2E Tests (`e2e/run_tests.sh`)
- Section 18 now creates `play_history` records instead of `rel_show_song` for the review test

### Documentation
- `docs/cli-learning.md` — Updated HTML report feature description
- `.claude/skills/reviewing-due-songs/SKILL.md` — Updated to document play_history source and vintage display

## Files Modified
- `src/commands/learning.rs`
- `templates/learning-song-review.html`
- `tests/test_learning.rs`
- `e2e/run_tests.sh`
- `docs/cli-learning.md`
- `.claude/skills/reviewing-due-songs/SKILL.md`
