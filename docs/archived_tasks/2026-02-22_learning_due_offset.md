# Task: Add time offset to learning-due queries

## Goal
Allow `learning-due` and `learning-song-review` to look ahead into the future by a configurable offset (in seconds). For example, `--offset 7200` would find all songs due in the next 2 hours.

## Key Requirements
- Add `--offset` parameter (in seconds) to `learning-due` and `learning-song-review` CLI commands
- Default offset is 0 (maintains current behavior exactly)
- The offset shifts the "now" reference point forward: `now + offset_seconds`

## Changes Made

### `src/commands.rs`
- Replaced `DUE_WHERE` const with `build_due_where(offset_seconds: u32) -> String` function
- Updated `cmd_learning_due` signature: added `offset_seconds: u32` parameter
- Updated `cmd_learning_song_review` signature: added `offset_seconds: u32` parameter

### `src/main.rs`
- Added `--offset` CLI arg (u32, default 0) to `LearningDue` subcommand
- Added `--offset` CLI arg (u32, default 0) to `LearningSongReview` subcommand
- Pass offset to command functions

### `tests/test_learning.rs`
- Updated all existing `cmd_learning_due` and `cmd_learning_song_review` calls to pass `0` as offset
- Added 3 new tests:
  - `test_learning_due_offset_makes_not_yet_due_visible` — level 0 warm-up bypassed with offset
  - `test_learning_due_offset_higher_level` — higher level due with 2-day offset
  - `test_learning_due_offset_zero_same_as_default` — offset=0 is identical to default

### `e2e/run_tests.sh`
- Added e2e test: `learning-due --offset 400` finds newly created song (warm-up bypassed)

### Documentation
- `docs/cli-learning.md` — Added `--offset` to options tables, examples, SQL, and explanations for both `learning-due` and `learning-song-review`
- `.claude/skills/learning-with-jankenoboe/SKILL.md` — Added offset examples and description
- `.claude/skills/reviewing-due-songs/SKILL.md` — Added offset examples and note
- `README.md` — Added offset example in Learning section

## Progress
- [x] Analyze current implementation (commands.rs, main.rs)
- [x] Check docs and skills for context
- [x] Check existing tests
- [x] Check e2e tests
- [x] Implement `build_due_where()` function in commands.rs
- [x] Update `cmd_learning_due` and `cmd_learning_song_review` signatures
- [x] Add `--offset` CLI args in main.rs
- [x] Add unit tests for offset behavior
- [x] Update e2e tests
- [x] Update documentation (docs, skills, README.md)
- [x] Run clippy and fmt (clean)
- [x] Run tests (all 180 pass)