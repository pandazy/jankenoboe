# v2.0.0 — CLI-Based Executable with Agentic Workflow

A complete reimagining of Jankenoboe as a Rust CLI tool designed for both direct use and AI-agent-driven workflows.

## What's New

### Rust CLI Tool

Jankenoboe is now a fast, standalone command-line executable built in Rust. It provides validated database operations with structured JSON output, making it ideal for both human users and AI agents.

**Querying Commands:**
- `get` — Retrieve records by ID with dynamic field selection
- `search` — Search with table-specific filters (name, artist, show, vintage, etc.)
- `duplicates` — Detect duplicate records by name for data quality

**Learning (Spaced Repetition):**
- `learning-due` — Query songs due for review based on Fibonacci-based intervals
- `learning-batch` — Batch add songs to the learning system

**Data Management:**
- `create` / `update` / `delete` — Full CRUD operations across all tables
- `bulk-reassign` — Bulk reassign songs to a different artist

### Agent Skills for Agentic Workflow

Four Claude Agent Skills enable non-technical users to interact with the database through natural conversation:

- **querying-jankenoboe** — Search and read artists, shows, songs, learning status, and duplicates
- **learning-with-jankenoboe** — Spaced repetition operations: batch add, level up/down, graduate, re-learn
- **maintaining-jankenoboe-data** — CRUD operations, bulk reassignment, and duplicate merging
- **reviewing-due-songs** — Display due review songs with show names and media URLs

### Cross-Platform Support

Pre-built binaries for:
- Linux x86_64
- Linux ARM64 (aarch64)
- macOS Intel (x86_64)
- macOS Apple Silicon (aarch64)

### Security

- SQL injection prevention via [JankenSQLHub](https://github.com/pandazy/jankensqlhub) with `enum`/`enumif` constraints
- Parameterized queries with per-table field validation
- Whitelisted table names and field names

### Testing

- Unit and integration tests covering CLI commands, error handling, and SQL injection
- Docker-based end-to-end tests simulating real installation and usage
- CI on Linux and macOS via GitHub Actions