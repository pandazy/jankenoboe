# Task: Add batch-get command

**Date:** 2026-03-09
**Status:** ✅ Complete

## Summary

Added `batch-get` command — the batch version of `get` by ID. Accepts a table name, comma-separated IDs (`--ids`), and fields (`--fields`), returning `{"count": N, "results": [...]}`.

## Usage

```bash
jankenoboe batch-get <table> --ids uuid-1,uuid-2,uuid-3 --fields id,name
```

## Implementation

- Uses JankenSQLHub's `:[ids]` list parameter: `SELECT ~[fields] FROM #[table] WHERE id IN :[ids]`
- Same table/field validation as `get` (reuses `GET_TABLES` and `build_selectable_enumif`)
- Nonexistent IDs are silently ignored (no error)
- Output: `{"count": N, "results": [...]}`

## Files Changed

| File | Change |
|------|--------|
| `src/commands/querying.rs` | Added `cmd_batch_get` function |
| `src/commands/mod.rs` | Exported `cmd_batch_get` |
| `src/main.rs` | Added `BatchGet` subcommand variant and match arm |
| `tests/test_querying.rs` | Added 10 tests |
| `docs/cli-querying.md` | Full technical reference with JankenSQLHub query definition and error cases |
| `docs/cli.md` | Added to querying commands table |
| `AGENTS.md` | Updated module responsibilities and CLI commands tables |
| `README.md` | Added usage example |
| `.claude/skills/querying-jankenoboe/SKILL.md` | Added "Batch Get by IDs" section |

## Tests Added (10)

- `test_batch_get_multiple_artists` — fetch 2 of 3 artists
- `test_batch_get_single_id` — single ID works
- `test_batch_get_songs` — songs with artist_id field
- `test_batch_get_nonexistent_ids_ignored` — mixed real + fake IDs
- `test_batch_get_all_nonexistent` — all fake IDs → empty results
- `test_batch_get_empty_ids` — error: ids cannot be empty
- `test_batch_get_empty_fields` — error: fields cannot be empty
- `test_batch_get_invalid_table` — error: Invalid table
- `test_batch_get_invalid_field` — error: Invalid field
- `test_batch_get_learning_records` — learning table with level/graduated
