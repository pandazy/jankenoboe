# Progress: Migrate learning.rs manual SQL to JankenSQLHub

## Goal
Replace manual low-level rusqlite operations in `src/commands/learning.rs` with JankenSQLHub query definitions.

## Status: ✅ COMPLETE

## Changes Made

### `src/commands/learning.rs`
- **Removed** `build_due_where(offset_seconds)` function — replaced with `DUE_WHERE` const that uses `@offset` parameter
- **`cmd_learning_due`** — replaced manual `conn.prepare()` + `query_map` with `QueryDefinitions` + `query_run_sqlite`
- **`cmd_learning_batch`** — replaced manual `tx.query_row` and `tx.execute` with `query_run_sqlite_with_transaction` for all check/insert operations within the transaction
- **`cmd_learning_song_review`** — replaced all manual `conn.prepare()` + `query_map` calls (due songs, artist name, show info, play history URLs) with `QueryDefinitions` + `query_run_sqlite`
- **`cmd_learning_song_levelup_ids`** — replaced `conn.query_row` for fetches + `tx.execute` for updates with `query_run_sqlite` (reads) + `query_run_sqlite_with_transaction` (writes)
- **`cmd_learning_by_song_ids`** — already used JankenSQLHub (no changes)

### Key patterns used:
- `query_run_sqlite` for standalone reads
- `query_run_sqlite_with_transaction` for operations within user-managed transactions
- `@param` with `{"type": "integer"}` for numeric parameters
- `COALESCE()` to handle NULL `wait_days` (was previously handled by `get::<_, Option<i64>>`)

## Checklist
- [x] Analyze current code and identify manual SQL
- [x] Migrate `cmd_learning_due`
- [x] Migrate `cmd_learning_batch`
- [x] Migrate `cmd_learning_song_review`
- [x] Migrate `cmd_learning_song_levelup_ids`
- [x] Remove `build_due_where` helper (replaced by `DUE_WHERE` const)
- [x] Run clippy and fmt
- [x] Run tests — all 200 tests pass