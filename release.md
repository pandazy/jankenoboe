# v2.2.0 — Search refactoring with JankenSQLHub integration

## Changes

### Centralized field configuration
- **`table_config.rs` is now the single source of truth** for all per-table field constraints (selectable, searchable, creatable, updatable). Adding a searchable field to a table requires changing only one line.
- **New `build_searchable_enumif()` builder** — generates JankenSQLHub `enumif` constraints for search column validation from the centralized config.
- **`build_table_enum()` and `build_selectable_enumif()`** now used by both `cmd_get` and `cmd_search`, replacing inline hardcoded maps.

### Search command refactored
- **Meaningful parameter names** — search parameters now use `col_{name}`/`val_{name}` (e.g., `col_vintage`, `val_vintage`) instead of opaque `c0`/`v0` indices, producing clearer error messages.
- **Shared constraints** — all JankenSQLHub constraints (table enum, fields enumif, searchable enumif) are derived from `table_config` instead of duplicated inline JSON.

### Improved error handling
- **JankenSQLHub error metadata preserved** — `From<anyhow::Error>` for `AppError` now uses `downcast_ref::<JankenError>()` to extract structured error data (error name, param name, rejected value) instead of losing it with `.to_string()`.
- **All 8 `query_run_sqlite` calls** changed from `.map_err(|e| AppError::Internal(e.to_string()))` to `.map_err(AppError::from)`, ensuring metadata flows through to error output.

### Test improvements
- **All error assertions in `test_querying.rs` now check exact messages** — zero `is_err()`/`is_ok()` calls remain. Each error test asserts specific error content (error type, rejected value, allowed values).
- **`test_search_invalid_table`** uses a truly invalid table (`"bad_table"`) instead of `"learning"` (which is now a valid search table).
- **`test_search_invalid_column`** asserts both `PARAMETER_TYPE_MISMATCH` and the rejected column name.

### Learning table now searchable
- `learning` table added to `SEARCH_TABLES` with searchable fields: `song_id`, `level`, `graduated`, `last_level_up_at`, `level_up_path`.

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/install.sh | sh
```

## Platform Support

Pre-built binaries for:
- Linux x86_64
- Linux ARM64 (aarch64)
- macOS Apple Silicon (aarch64)