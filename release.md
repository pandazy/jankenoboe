# v2.3.0 — Look-ahead offset for learning-due queries

## Changes

### New `--offset` parameter for `learning-due` and `learning-song-review`
- Added `--offset <seconds>` option that shifts the reference time forward into the future
- Allows checking which songs will be due soon, e.g., `--offset 7200` finds songs due within the next 2 hours
- Default is `0` (no change from previous behavior — compares against "now")
- Works with minute-level precision, e.g., `--offset 120` for the next 2 minutes

### Examples
```bash
jankenoboe learning-due --offset 7200          # due within next 2 hours
jankenoboe learning-due --offset 120 --limit 50 # due within next 2 minutes
jankenoboe learning-song-review --offset 7200   # HTML report including near-due songs
```

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/install.sh | sh
```

## Platform Support

Pre-built binaries for:
- Linux x86_64
- Linux ARM64 (aarch64)
- macOS Apple Silicon (aarch64)