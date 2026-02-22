# v2.2.2 — Import skill: skip play_history duplicate check

## Changes

### Updated import skill documentation
- **Skip play_history duplicate checking** — the `importing-amq-songs` skill no longer instructs agents to check for duplicate `play_history` records before importing, simplifying the import workflow.

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/install.sh | sh
```

## Platform Support

Pre-built binaries for:
- Linux x86_64
- Linux ARM64 (aarch64)
- macOS Apple Silicon (aarch64)