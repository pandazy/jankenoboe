# 2026-02-18: Add Code Coverage

## Task Summary
Added test coverage tooling and tests to bring line coverage from **83.75% → 96.74%**.

## Changes Made

### Coverage Tooling
- Installed `cargo-llvm-cov` for LLVM-based source code coverage
- Added `tempfile = "3"` as a dev-dependency for CLI integration tests with temp databases

### New Tests Added (130 total, up from ~70)

#### Unit Tests in `src/`
- **`src/error.rs`** (7 tests): `Display` for all 4 `AppError` variants, `From` impls for `rusqlite::Error`, `serde_json::Error`, `anyhow::Error`
- **`src/db.rs`** (2 tests): missing `JANKENOBOE_DB` env var error, successful `:memory:` connection
- **`src/models.rs`** (10 tests): error branches for all 5 per-table field functions (`get_fields`, `search_columns`, `search_fields`, `create_data_fields`, `update_data_fields`) with invalid tables, plus positive coverage for all valid tables

#### Integration Tests in `tests/`
- **`tests/test_querying.rs`** (+8 tests): search empty fields, non-object term condition, missing value, non-string value, invalid JSON, song duplicates, multiple duplicate groups, search empty fields validation
- **`tests/test_data_management.rs`** (+15 tests): update show/song/play_history tables, boolean JSON value (`json_value_to_sql` Bool branch), null JSON value (Null branch), float number (f64 branch), JSON array (catch-all branch), float column in DB (`row_value_at` float path), null column (`row_value_at` null path), `rel_show_song` without `media_url`, empty `song_ids` for bulk-reassign, invalid JSON for create/update, invalid table names
- **`tests/test_cli.rs`** (18 tests total — 12 error + 6 success): CLI integration tests using `Command` and temp SQLite databases for all subcommands: `get`, `search`, `duplicates`, `create`, `update`, `delete`, `bulk-reassign`, `learning-due`, `learning-batch`, plus `--version`, no-args help, and missing DB env

### Coverage Results

| File | Before | After |
|------|--------|-------|
| commands.rs | 92.13% | **96.33%** |
| db.rs | 0.00% | **75.00%** |
| easing.rs | 100.00% | **100.00%** |
| error.rs | 25.00% | **100.00%** |
| main.rs | 0.00% | **100.00%** |
| models.rs | 90.07% | **100.00%** |
| **TOTAL** | **83.75%** | **96.74%** |

### Remaining Uncovered (~3.3%)
- `unreachable!()` branches in match arms (impossible to hit by design)
- `?` error-propagation branches on successful DB writes (would require simulated DB failures)
- `db::open_test_connection()` — test infrastructure function, excluded from coverage instrumentation

## How to Run Coverage

```bash
# Summary
JANKENOBOE_DB=":memory:" cargo llvm-cov --no-cfg-coverage --summary-only

# HTML report (opens in browser)
JANKENOBOE_DB=":memory:" cargo llvm-cov --no-cfg-coverage --html
open target/llvm-cov/html/index.html

# Text report with line-level detail
JANKENOBOE_DB=":memory:" cargo llvm-cov --no-cfg-coverage --text
```

## Files Modified
- `Cargo.toml` — added `[dev-dependencies] tempfile = "3"`
- `src/error.rs` — added `#[cfg(test)] mod tests`
- `src/db.rs` — added `#[cfg(test)] mod tests`
- `src/models.rs` — added `#[cfg(test)] mod tests`
- `tests/test_querying.rs` — added 8 new tests
- `tests/test_data_management.rs` — added 15 new tests
- `tests/test_cli.rs` — new file with 18 CLI integration tests