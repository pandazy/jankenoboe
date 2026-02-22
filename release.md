# v2.1.1 — Drop Intel Mac build

## Changes

- **Removed x86_64 macOS (Intel) release build** — Only Apple Silicon (aarch64) macOS binaries are now published. Intel Mac users can run the Apple Silicon binary through Rosetta 2.
- **`install.sh` now rejects macOS x86_64** — The installer exits with a clear error message on Intel Macs instead of attempting to download a non-existent asset.
- **Updated docs** — `docs/development.md` cross-platform build table updated to reflect 3 targets.

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/install.sh | sh
```

## Platform Support

Pre-built binaries for:
- Linux x86_64
- Linux ARM64 (aarch64)
- macOS Apple Silicon (aarch64)

Note: Intel Mac users can run the Apple Silicon binary through Rosetta 2.