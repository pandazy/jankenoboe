# Search Refactor: JankenSQLHub Integration

## Goal
Replace manual SQL construction and field/column validation in `cmd_search` with JankenSQLHub-based query execution, using `enumif` constraints to validate both search columns and output fields.

## What Was Done

### Phase 1: Centralized Config + JankenSQLHub for Search
- Created `src/table_config.rs` — single source of truth for per-table field metadata
- Refactored `models.rs` to delegate field lookups to `table_config`
- Refactored `cmd_search` to build dynamic JankenSQLHub `QueryDefinitions`
- Refactored `cmd_get` to use `table_config` helpers for `enumif`/`enum`

### Phase 2: Eliminate Manual Validation in Search
- **Removed `allowed_columns` validation** — column names are now `#[col_N]` identifier parameters with `enumif` constraints per table, validated by JankenSQLHub
- **Removed `allowed_fields` validation in search** — `~[fields]` uses `enumif` per table, validated by JankenSQLHub
- **Removed `search_fields()` and `search_columns()`** from `models.rs` (no longer called)
- **Removed their unit tests** (4 tests removed)
- Updated `test_search_invalid_column` to match JankenSQLHub's error behavior

### Key Design Decisions
- `cmd_search` is self-contained with inline `enumif` definitions — not coupled to `table_config`
- JankenSQLHub handles all field/column/table validation for search via constraints
- `table_config.searchable` field retained as documentation/reference
- Search treats itself as a separate compartment from get/create/update

## Files Changed
- `src/table_config.rs` — Centralized field config
- `src/models.rs` — Removed `search_fields()`, `search_columns()`, 4 tests
- `src/commands.rs` — `cmd_search` uses `#[col_N]` with `enumif` + `~[fields]` with `enumif`, `cmd_get` uses centralized config
- `src/lib.rs` — Added `pub mod table_config`
- `tests/test_querying.rs` — Updated `test_search_invalid_column` assertion
- `docs/structure.md`, `AGENTS.md`, `.clinerules` — Updated

## Test Results
- 176 tests pass, 0 failures, 0 warnings
- Removed 4 now-unnecessary models tests (search_fields/search_columns)

## Status: COMPLETE