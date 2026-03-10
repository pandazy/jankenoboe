# Task: Decouple HTML Rendering from Rust Code

**Date:** 2026-02-24
**Status:** ✅ Complete

## Summary

Decoupled HTML rendering logic from Rust code (`learning.rs`) into the HTML template (`templates/learning-song-review.html`). Previously, the `build_review_html` function in Rust was constructing HTML fragments (level-badge `<span>` elements, media URL `<a>` tags, no-media fallback) and embedding them in the template. Now, Rust passes pure JSON data and the template's JavaScript handles all rendering client-side.

## Changes Made

### `src/commands/learning.rs`
- **`build_review_html`** now builds pure JSON data instead of HTML fragments:
  - Level distribution: `{{DIST_JSON}}` — array of `{level, count}` objects (was `{{DIST_HTML}}` with pre-rendered `<span>` badges)
  - Songs: `mediaUrls` field — array of `{url, ext}` objects (was `mediaHtml` with pre-rendered `<a>` tags)
  - Uses `serde_json::to_string` for proper JSON serialization
- **Removed** `escape_json_string` function (no longer needed — replaced by `serde_json::to_string`)

### `templates/learning-song-review.html`
- Added `renderLevelDist()` JS function — renders level-badge HTML from `LEVEL_DIST` JSON data
- Added `renderMediaHtml()` JS function — renders media links from `mediaUrls` data (including file extension labels and "No media URLs" fallback)
- Replaced `{{DIST_HTML}}` placeholder with `{{DIST_JSON}}`
- Song cards now use `s.mediaUrls` instead of pre-rendered `s.mediaHtml`

### `tests/test_learning.rs`
- Updated `test_learning_song_review_deduplicates_media_urls` to check JSON data (URL string count) instead of pre-rendered HTML ("Media 1" count)

## Verification

- All 164 tests pass (44 learning, 54 querying, 48 data management, 18 CLI)
- `cargo clippy --fix --allow-dirty` — clean
- `cargo fmt` — clean
- Real-world test: generated report with 500 songs from production database, verified in browser