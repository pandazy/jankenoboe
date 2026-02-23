# v2.3.3 — macOS compatibility, cleanup, and due report improvements

## Changes

### Due song review: file extension in media links
- Media links in the `learning-song-review` HTML report now show the file extension (e.g., "Media 1 (.mp3)", "Media 2 (.webm)")
- URLs without a recognizable extension still display as "Media 1", "Media 2", etc.

### Shell script macOS compatibility
- Replaced `mapfile -t` (Bash 4+ only) with portable `while IFS= read -r` loops in `check_artists.sh` and `check_shows.sh`
- macOS ships with Bash 3.2 which does not support `mapfile`

### Python cache cleanup
- Removed committed `__pycache__/` directory from git tracking
- Added `__pycache__/` and `*.pyc` to `.gitignore`

### Documentation
- Added Python 3 prerequisite to the AMQ import skill setup section

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/install.sh | sh
```

## Platform Support

Pre-built binaries for:
- Linux x86_64
- Linux ARM64 (aarch64)
- macOS Apple Silicon (aarch64)