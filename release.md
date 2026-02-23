# v2.3.2 — Import script two-phase approach

## Changes

### AMQ import script refactored to two-phase approach
- **Phase 1 (Resolution):** Resolves all entities (artist, show, song) from the database. Entries are separated into "complete" (all found) and "missing" (any entity not in DB) groups.
- **Phase 2 (Processing):** For complete entries, automatically creates show-song links and play history records. Missing entries are skipped and reported.
- **Missing Entities Report:** Grouped, deduplicated output of missing artists, shows, and songs with per-entry detail breakdown.
- The script no longer creates any entities (artists, shows, or songs) — only links and play_history for entries where everything already exists.

### `--missing-only` flag for safe re-runs
- Skips entries where the show-song link already exists (previously processed), preventing duplicate play_history on re-runs.
- Workflow: run import → review missing report → manually create missing entities → re-run with `--missing-only`.

### Import integration test
- New `test_import_amq.py` with 20 assertions using a temp DB and the jankenoboe binary.
- Covers: first import, `--missing-only` skip behavior, duplicate detection, fix-and-rerun workflow.

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/install.sh | sh
```

## Platform Support

Pre-built binaries for:
- Linux x86_64
- Linux ARM64 (aarch64)
- macOS Apple Silicon (aarch64)