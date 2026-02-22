# v2.3.1 — Search term key validation refactor

## Changes

### Simplified search term key validation
- `--term` keys are now validated directly in Rust against per-table `searchable` config instead of using JankenSQLHub's `enumif` constraints
- Removed the hacky `col_{}` / `val_{}` dynamic identifier pattern from `cmd_search`
- Removed `build_searchable_enumif` from `table_config.rs` (no longer needed)
- `--fields` (response fields) still uses JankenSQLHub's `enumif` for validation

### Improved error messages
- Invalid term key errors now include the allowed keys: `"Invalid term key for artist: id. Allowed: name, name_context"`
- Invalid table errors from term key validation now include allowed tables: `"Invalid table in term key validation: bad_table. Allowed: artist, show, ..."`

### Code quality
- Added `allowed_term_keys()` function in `models.rs` as a clean accessor for searchable fields
- SQL injection test for search term keys now asserts exact error messages instead of vague `.is_err()`
- Test names use consistent "term key" terminology

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/install.sh | sh
```

## Platform Support

Pre-built binaries for:
- Linux x86_64
- Linux ARM64 (aarch64)
- macOS Apple Silicon (aarch64)