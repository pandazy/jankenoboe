# Task: Search Show by name_romaji

**Date:** 2026-02-28
**Version:** 2.5.1

## Objective

Allow the `search show` command to filter by the `name_romaji` field, enabling lookups of shows by their Japanese romaji names.

## Problem

The `show` table had a `name_romaji` column that was selectable (via `--fields`), creatable, and updatable, but was not included in the searchable fields. Users could not search for shows by their romaji names.

## Changes

### Code

- **`src/table_config.rs`** — Added `"name_romaji"` to `SHOW.searchable`
  - Before: `["name", "vintage"]`
  - After: `["name", "name_romaji", "vintage"]`

No changes needed in `querying.rs`, `models.rs`, or `main.rs` — the search infrastructure dynamically builds queries from the `searchable` config, so adding the field to the whitelist was sufficient.

### Tests

- **`tests/test_querying.rs`**:
  - Added `insert_show_full` helper (accepts optional `name_romaji`)
  - `test_search_show_by_name_romaji_exact_i` — exact case-insensitive match
  - `test_search_show_by_name_romaji_contains` — partial match via contains
  - `test_search_show_by_name_romaji_and_vintage` — combined AND search (name_romaji + vintage)

- **`e2e/run_tests.sh`** — Added 2 assertions in the Show CRUD section:
  - Search by `name_romaji` with `exact-i` match mode
  - Search by `name_romaji` with `contains` match mode

### Documentation

- **`docs/cli-querying.md`** — Updated searchable columns table, added examples, updated enumif JSON
- **`.claude/skills/querying-jankenoboe/SKILL.md`** — Updated searchable columns table, added search examples

### Release

- **`Cargo.toml`** — Version bumped from 2.5.0 to 2.5.1
- **`release.md`** — Updated with v2.5.1 release notes

## Usage Examples

```bash
# Find show by romaji name (case-insensitive)
jankenoboe search show --fields id,name,name_romaji --term '{"name_romaji":{"value":"yubisaki to renren","match":"exact-i"}}'

# Find shows whose romaji name contains "kimi"
jankenoboe search show --fields id,name,name_romaji,vintage --term '{"name_romaji":{"value":"kimi","match":"contains"}}'
```

## Files Modified

| File | Change |
|------|--------|
| `src/table_config.rs` | Added `name_romaji` to show's searchable fields |
| `tests/test_querying.rs` | Added `insert_show_full` helper + 3 tests |
| `e2e/run_tests.sh` | Added 2 e2e assertions for name_romaji search |
| `docs/cli-querying.md` | Updated tables, examples, enumif JSON |
| `.claude/skills/querying-jankenoboe/SKILL.md` | Updated tables and examples |
| `Cargo.toml` | Version 2.5.0 → 2.5.1 |
| `release.md` | v2.5.1 release notes |