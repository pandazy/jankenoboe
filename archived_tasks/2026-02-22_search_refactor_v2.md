# Search Refactor: Remove col_{} Hacky Logic

## Goal
Replace the hacky `col_{}` JankenSQLHub enumif pattern in `cmd_search` with direct Rust validation of `--term` keys against `table_config::searchable`.

## Changes Made
- [x] Added `allowed_term_keys()` in `models.rs` — validates --term keys against searchable config
- [x] Refactored `cmd_search` — validates term keys in Rust, embeds whitelisted column names directly in SQL
- [x] Removed `build_searchable_enumif` from `table_config.rs` (no longer needed)
- [x] Removed redundant `validate_table` call from `cmd_search` (handled by `allowed_term_keys`)
- [x] Improved error messages with context ("term key validation") and allowed values
- [x] Renamed test functions to use "term key" terminology consistently
- [x] Fixed SQL injection test to assert exact error instead of vague `.is_err()`
- [x] Copied `using-jankensqlhub` skill into `.claude/skills/`
- [x] Removed outdated "Patterns Learned from Jankenoboe" section from skill
- [x] All 179 tests pass, zero clippy warnings