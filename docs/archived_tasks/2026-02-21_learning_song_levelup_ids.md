# Task: Replace learning-song-levelup-due with learning-song-levelup-ids

**Date:** 2026-02-21

## Problem

The `learning-song-levelup-due` command had a race condition: it re-queried for due songs at execution time. If new songs became due between generating a review report and running the level-up command, those unreviewed songs would also be leveled up incorrectly.

## Solution

1. **Added `learning-song-levelup-ids`** — accepts specific learning record UUIDs via `--ids`, levels up exactly those records regardless of due status
2. **Updated `learning-song-review`** — now returns `learning_ids` array in its JSON output, capturing the exact set of due songs at report generation time
3. **Removed `learning-song-levelup-due`** — eliminated the unsafe command entirely

## Race-condition-safe workflow

```bash
# Step 1: Generate review (captures learning_ids at this moment)
out=$(jankenoboe learning-song-review --output ~/review.html)

# Step 2: User reviews the HTML report

# Step 3: Level up exactly those songs (no race condition)
ids=$(echo "$out" | jq -r '.learning_ids | join(",")')
jankenoboe learning-song-levelup-ids --ids "$ids"
```

## Files Changed

### Implementation
- `src/main.rs` — Replaced `LearningSongLevelupDue` with `LearningSongLevelupIds` subcommand
- `src/commands.rs` — Removed `cmd_learning_song_levelup_due`, added `cmd_learning_song_levelup_ids`, updated `cmd_learning_song_review` to return `learning_ids`

### Tests
- `tests/test_learning.rs` — Replaced 4 levelup-due tests with 10 levelup-ids tests + 2 review learning_ids tests
- `e2e/run_tests.sh` — Replaced levelup-due e2e tests with levelup-ids + review→levelup flow tests

### Documentation
- `README.md` — Updated CLI examples
- `AGENTS.md` — Updated CLI commands table
- `.clinerules` — Updated CLI commands list
- `docs/cli.md` — Updated command tables and operations coverage
- `docs/cli-learning.md` — Removed levelup-due section, added levelup-ids section, updated review output format
- `skills/reviewing-due-songs/SKILL.md` — Updated workflow to recommend levelup-ids, removed levelup-due alternative
- `skills/learning-with-jankenoboe/SKILL.md` — Added batch levelup by IDs section, updated review workflow

## Key Design Decisions

- `learning-song-levelup-ids` does **not** check due status — it trusts the caller to pass valid IDs (typically from `learning-song-review` output)
- Validates all IDs exist and are not already graduated before any updates
- All updates in a single transaction for atomicity
- Rejects graduated records with a clear error message