# v2.0.1 — Bug Fix: Enable learning table search

## Bug Fixes

- **Fixed learning table search** — The `learning` table can now be queried with the `search` command. Previously, attempting to search the learning table would fail due to a missing table configuration.

## Installation

Download the appropriate binary for your platform from the release assets and install using:

```bash
curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/install.sh | sh
```

## Platform Support

Pre-built binaries for:
- Linux x86_64
- Linux ARM64 (aarch64)
- macOS Apple Silicon (aarch64)

Note: Intel Mac users can run the Apple Silicon binary through Rosetta 2.
