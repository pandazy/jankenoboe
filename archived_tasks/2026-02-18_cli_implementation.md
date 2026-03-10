# 2026-02-18: CLI Implementation (HTTP → CLI Rewrite)

## Task Summary
Rewrote the entire jankenoboe application from an **Axum HTTP server** to a **Clap CLI tool**. The previous codebase was an async HTTP API using Axum/Tokio; the new one is a synchronous CLI binary with subcommands, JSON output, and comprehensive test coverage.

## Scope of Changes
- **3,018 lines added, 275 lines removed** across 13 files
- Replaced Axum/Tokio/Tower HTTP stack with Clap CLI framework
- Implemented 10 CLI subcommands covering querying, learning, and data management
- Added 130 tests (from 0) with 96.74% line coverage
- Added `cargo-llvm-cov` coverage tooling

## Architecture Change

### Before (Axum HTTP Server)
- `main.rs`: Tokio async runtime, Axum router, TCP listener on port 3000
- Dependencies: `axum`, `tokio`, `tower`, `tower-http`, `tracing`, `tracing-subscriber`
- `lib.rs`: `create_app()` and `init_db()` exports
- `commands.rs`: 118 lines (HTTP route handlers)
- `models.rs`: 29 lines (request/response types)
- **No tests**

### After (Clap CLI Tool)
- `main.rs`: Clap argument parsing, subcommand dispatch, JSON stdout/stderr output
- Dependencies: `clap` (with `derive`), `uuid` — removed Axum/Tokio/Tower/Tracing
- `lib.rs`: Module re-exports for `commands`, `db`, `easing`, `error`, `models`
- `commands.rs`: 727 lines (10 subcommand implementations)
- `models.rs`: 303 lines (table validation, field whitelists, constants)
- **130 tests** across 4 test files

## Files Changed

### Source Files

| File | Before (lines) | After (lines) | Description |
|------|----------------|---------------|-------------|
| `src/main.rs` | 31 | 151 | Axum server → Clap CLI with 10 subcommands |
| `src/commands.rs` | 118 | 727 | HTTP handlers → CLI command implementations |
| `src/models.rs` | 29 | 303 | Minimal types → table/field validation, constants, unit tests |
| `src/db.rs` | 25 | 64 | DB init → `open_connection()`, `open_test_connection()`, unit tests |
| `src/error.rs` | 104 | 104 | Restructured `AppError` with `Display`, `From` impls, unit tests |
| `src/easing.rs` | 59 | 62 | Minor cleanup (existing Fibonacci logic preserved) |
| `src/lib.rs` | 16 | 5 | `create_app()`/`init_db()` → module re-exports |
| `Cargo.toml` | — | — | Replaced Axum/Tokio deps with Clap; added `tempfile` dev-dep |

### New Test Files

| File | Tests | Description |
|------|-------|-------------|
| `tests/test_querying.rs` | 31 | `get`, `search`, `duplicates` commands |
| `tests/test_data_management.rs` | 39 | `create`, `update`, `delete`, `bulk-reassign` commands |
| `tests/test_learning.rs` | 21 | `learning-due`, `learning-batch`, SQL injection tests |
| `tests/test_cli.rs` | 18 | Binary integration tests (error paths + success with temp DB) |
| **Unit tests in src/** | 21 | `error.rs` (7), `db.rs` (2), `models.rs` (10), `easing.rs` (2) |

### Other Files

| File | Description |
|------|-------------|
| `docs/development.md` | Added coverage instructions |

## CLI Commands Implemented

### Querying
- `jankenoboe get <table> <id> --fields <fields>` — Read record by ID with dynamic field selection
- `jankenoboe search <table> --term <json> --fields <fields>` — Search with table-specific filters and match modes (exact, exact-i, starts-with, ends-with, contains)
- `jankenoboe duplicates <table>` — Find duplicate records by case-insensitive name matching

### Learning (Spaced Repetition)
- `jankenoboe learning-due --limit <n>` — Get songs due for review based on Fibonacci intervals
- `jankenoboe learning-batch --song-ids <ids> [--relearn] [--start-level <n>]` — Batch add/re-learn songs

### Data Management
- `jankenoboe create <table> --data <json>` — Create records with field validation
- `jankenoboe update <table> <id> --data <json>` — Update records (auto-sets `updated_at`, `last_level_up_at` for learning level changes)
- `jankenoboe delete <table> <id>` — Delete records (artist, song only)
- `jankenoboe bulk-reassign` — Bulk reassign songs between artists (by song IDs or by artist)

## Key Design Decisions

1. **JSON output**: All commands output JSON to stdout (success) or stderr (errors) with exit code 0/1, enabling easy consumption by AI agents
2. **Table/field whitelisting**: Every command validates table names and field names against explicit allow-lists in `models.rs` to prevent SQL injection
3. **Direct SQL over JankenSQLHub**: While JankenSQLHub was available, direct parameterized SQL was used for clarity and because the CLI's validation layer already provides security
4. **`open_test_connection()`**: In-memory SQLite with schema initialization for fast, isolated integration tests
5. **Fibonacci-based spaced repetition**: Preserved from original codebase, with 20 levels and configurable `level_up_path`

## Dependencies Removed
- `axum` (HTTP framework)
- `tokio` (async runtime)
- `tower`, `tower-http` (middleware)
- `tracing`, `tracing-subscriber` (logging)

## Dependencies Added
- `clap` with `derive` feature (CLI argument parsing)
- `tempfile` (dev-dependency for CLI integration tests)