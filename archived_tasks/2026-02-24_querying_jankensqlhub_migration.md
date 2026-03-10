# Querying JankenSQLHub Migration

**Date:** 2026-02-24
**Status:** Complete

## Task

Migrate the last remaining manual rusqlite SQL operations in `src/commands/querying.rs` to use JankenSQLHub.

## Changes

### `src/commands/querying.rs` — `cmd_duplicates`

The only function still using raw rusqlite (`conn.prepare()`, `stmt.query_map()`, manual `row.get()`) was `cmd_duplicates`. It was migrated to use JankenSQLHub's `QueryDefinitions` and `query_run_sqlite`.

**Before:** Manual SQL string construction with `format!()`, `conn.prepare()`, `stmt.query_map()`, and `row.get()` for each column.

**After:** JankenSQLHub query definition with `#[table]` identifier parameter (with `enum` constraint for safe table names), `returns` for automatic JSON response mapping, and `query_run_sqlite` for execution.

Two query variants are used:
- `duplicates_with_song_count` — for `artist`/`song` tables (includes song_count subquery)
- `duplicates_no_song_count` — for `show` table (uses `0 as song_count`)

The post-query grouping logic (grouping rows by lowercase name) remains the same, now iterating over `result.data` (Vec<Value>) instead of raw rusqlite rows.

## Result

All manual rusqlite access has been eliminated from `src/commands/*.rs`. Every command module now exclusively uses JankenSQLHub for database operations:
- `commands/querying.rs` — fully migrated
- `commands/learning.rs` — previously migrated
- `commands/data_management.rs` — previously migrated

## Tests

All 202 tests pass (38 unit + 18 CLI + 48 data management + 44 learning + 54 querying).