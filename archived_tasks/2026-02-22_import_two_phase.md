# 2026-02-22: Import Script Two-Phase Approach

## Task
Refactor `import_amq.py` to separate complete entries (all entities found in DB) from missing entries, process only the complete ones, and report the missing ones for manual handling. Add `--missing-only` flag for safe re-runs.

## Changes

### `.claude/skills/importing-amq-songs/scripts/import_amq.py`
- Refactored to two-phase approach:
  - **Phase 1 (Resolution):** Resolves artist, show, and song for each entry. Separates into "complete" and "missing" groups.
  - **Phase 2 (Processing):** For complete entries, creates show-song links and play_history. Missing entries are skipped.
- Added `--missing-only` flag: skips entries where the show-song link already exists (already processed in a previous run), preventing duplicate play_history on re-runs.
- Removed `create_song()` function — the script no longer creates any entities.
- Added `resolve_entry()`, `print_missing_report()`, `process_complete_entries()` functions.
- Missing Entities Report: grouped, deduplicated output of missing artists/shows/songs.

### `.claude/skills/importing-amq-songs/SKILL.md`
- Updated Option A documentation to describe the two-phase behavior.
- Documented the `--missing-only` flag and re-run workflow.

### `.claude/skills/importing-amq-songs/scripts/test_import_amq.py` (new)
- Integration test using temp DB and the jankenoboe binary.
- 20 test assertions covering:
  - First import: complete entries processed, missing reported
  - Re-run with `--missing-only`: already-linked entries skipped
  - Re-run without flag: duplicates demonstrated
  - Fix missing + re-run `--missing-only`: only new entries processed

## Formatted
- Both Python files formatted with `black`.