# macOS Compatibility and Cleanup

**Date:** 2026-02-22

## Summary

Fixed macOS compatibility issues in the AMQ import shell scripts and cleaned up Python cache artifacts.

## Changes

### Shell script macOS compatibility
- Replaced `mapfile -t` (Bash 4+ only) with portable `while IFS= read -r` loops in:
  - `.claude/skills/importing-amq-songs/scripts/check_artists.sh`
  - `.claude/skills/importing-amq-songs/scripts/check_shows.sh`
- macOS ships with Bash 3.2 which does not support `mapfile`

### Python cache cleanup
- Removed committed `__pycache__/` directory from git tracking
- Added `__pycache__/` and `*.pyc` to `.gitignore`

### Documentation
- Added Python 3 prerequisite to `.claude/skills/importing-amq-songs/SKILL.md` Setup section
- Includes install instruction: `brew install python3` for macOS users

## Testing

- `test_import_amq.py`: 20/20 passed
- `check_artists.sh`: Correctly processed 49 unique artists from sample export
- `check_shows.sh`: Correctly processed 50 unique shows from sample export