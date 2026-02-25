# v2.4.2 — JankenSQLHub migration & HTML template decoupling

## JankenSQLHub Migration

All command modules now use JankenSQLHub for parameterized SQL query management, replacing raw SQL string building with JSON-configured `QueryDef` definitions:

- **`querying.rs`** — `cmd_get`, `cmd_search`, `cmd_duplicates`, `cmd_shows_by_artist_ids`, `cmd_songs_by_artist_ids`
- **`learning.rs`** — `cmd_learning_due`, `cmd_learning_batch`, `cmd_learning_song_review`, `cmd_learning_song_levelup_ids`, `cmd_learning_by_song_ids`
- **`data_management.rs`** — `cmd_create`, `cmd_update`, `cmd_delete`, `cmd_bulk_reassign`

Benefits:
- `#[table]` with `enum` constraints for safe dynamic table names
- `~[fields]` with `enumif` for per-table field validation
- All parameters go through JankenSQLHub's validation layer
- Consistent query pattern across all commands

## HTML Template Decoupling

The `learning-song-review` HTML report now renders entirely client-side from pure JSON data:

- **Before:** Rust built HTML fragments (level badges, media URL links) and embedded them in the template
- **After:** Rust passes pure JSON data (`{{DIST_JSON}}`, `{{SONGS_JSON}}`), and JavaScript in the template handles all rendering

This separation makes the template easier to modify without touching Rust code.

## Test Coverage Improvements

Added 26 unit tests for private helper functions:

- **`learning.rs`** (14 tests): `escape_html`, `extract_url_extension`, `build_review_html` with `SongReviewData` trait
- **`data_management.rs`** (11 tests): `json_value_to_param`, `url_decode_map_values`, `add_integer_column`
- **`querying.rs`** (1 test): Invalid URL encoding error path

Coverage: **96.72% lines**, **85.42% functions** (up from 95.89% / 82.21%)

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/install.sh | sh
```

## Platform Support

Pre-built binaries for:
- Linux x86_64
- Linux ARM64 (aarch64)
- macOS Apple Silicon (aarch64)