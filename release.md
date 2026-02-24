# v2.4.1 — Improved agent skill setup instructions

## Improvements

### Smarter `JANKENOBOE_DB` detection in agent skills
- All 5 Claude agent skills now check `echo $JANKENOBOE_DB` before running commands
- If the environment variable is already set, agents proceed directly without asking
- If not set, agents ask the user for the database path and offer two options:
  - Export for the session: `export JANKENOBOE_DB=/path/to/datasource.db`
  - Prefix individual commands: `JANKENOBOE_DB=/path/to/datasource.db jankenoboe ...`
- Previously, skills always asked the user for the database path regardless of whether it was already configured

### Updated skills
- `querying-jankenoboe`
- `learning-with-jankenoboe`
- `maintaining-jankenoboe-data`
- `reviewing-due-songs`
- `importing-amq-songs`

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/install.sh | sh
```

## Platform Support

Pre-built binaries for:
- Linux x86_64
- Linux ARM64 (aarch64)
- macOS Apple Silicon (aarch64)