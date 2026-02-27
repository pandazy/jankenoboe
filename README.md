# Jankenoboe - Anime Song Learning System

An anime song learning system that tracks songs from [animemusicquiz.com](https://animemusicquiz.com) and uses spaced repetition to help memorize them. The name combines "janken" (the creator's alias on AMQ) with "oboe" (覚え, memory/memorization in Japanese).

Non-technical users interact through AI agents (e.g., Claude with [Agent Skills](#agent-skills-claude)) that call `jankenoboe` CLI commands. The CLI is a Rust binary that provides fast, validated database operations with JSON output.

## Agent Skills (Claude)

The `.claude/skills/` directory contains [Claude Agent Skills](https://code.claude.com/docs/en/skills) for interacting with the Jankenoboe database. Each skill uses the `jankenoboe` CLI binary for validated, fast operations with JSON output.

| Skill | Description |
|-------|-------------|
| [querying-jankenoboe](.claude/skills/querying-jankenoboe/SKILL.md) | Search and read artists, shows, songs, learning records, due reviews, duplicates |
| [learning-with-jankenoboe](.claude/skills/learning-with-jankenoboe/SKILL.md) | Spaced repetition: add songs, level up/down, graduate, check due reviews |
| [maintaining-jankenoboe-data](.claude/skills/maintaining-jankenoboe-data/SKILL.md) | CRUD operations: create/update/delete records, bulk reassign, merge duplicates |
| [reviewing-due-songs](.claude/skills/reviewing-due-songs/SKILL.md) | Display due review songs with show names, song names, and media URLs |
| [importing-amq-songs](.claude/skills/importing-amq-songs/SKILL.md) | Import AMQ song exports: resolve artists, shows, songs, create play history |

## Prerequisites

- **SQLite 3** — the database engine
  - macOS: `brew install sqlite` (or pre-installed)
  - Ubuntu/Debian: `sudo apt install sqlite3`

## Installation

### Option 1: Install from GitHub (recommended)

Download the pre-built binary for your platform:

```bash
curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/install.sh | sh
```

This detects your OS/architecture and installs `jankenoboe` to `~/.local/bin/`. Make sure `~/.local/bin` is in your `PATH`.

### Option 2: Install via Cargo

If you have Rust 1.70+ installed:

```bash
cargo install --git https://github.com/pandazy/jankenoboe.git
```

This builds from source and installs `jankenoboe` to `~/.cargo/bin/`.

### Option 3: Build from source

```bash
git clone https://github.com/pandazy/jankenoboe.git
cd jankenoboe
cargo build --release
# Binary at target/release/jankenoboe — copy it to your PATH
cp target/release/jankenoboe ~/.local/bin/
```

## Upgrading

To upgrade to the latest version, simply re-run the install script. It always fetches and installs the latest release, overwriting any existing binary — there is no version check or "already up to date" skip.

```bash
curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/install.sh | sh
```

If you installed via Cargo:

```bash
cargo install --git https://github.com/pandazy/jankenoboe.git --force
```

## Uninstallation

### If installed via install.sh or manual copy

```bash
curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/uninstall.sh | sh
```

Or run locally: `sh uninstall.sh`

This removes the `jankenoboe` binary from `~/.local/bin/` and `~/.cargo/bin/`. Your database and shell configuration are preserved.

### If installed via Cargo

```bash
cargo uninstall jankenoboe
```

### Optional cleanup

After uninstalling, you may also want to:

```bash
# Remove the database
rm ~/db/datasource.db

# Remove JANKENOBOE_DB from your shell profile (~/.zshrc, ~/.bashrc, etc.)
```

## Setup

### 1. Create the database

The database file lives **outside** the project directory (e.g., `~/db/datasource.db`). Create it from the schema:

```bash
mkdir -p ~/db
sqlite3 ~/db/datasource.db < docs/init-db.sql
```

This creates all tables, indexes, and constraints. See [docs/init-db.sql](docs/init-db.sql) for the full schema.

### 2. Set the database path

Add to your shell profile (`~/.zshrc`, `~/.bashrc`, etc.):

```bash
export JANKENOBOE_DB=~/db/datasource.db
```

### 3. Import your song data

1. Play a round on [animemusicquiz.com](https://animemusicquiz.com)
2. After the game, export your song list as JSON (see [sample export](docs/amq_song_export-sample.json) for the format)
3. Give the exported JSON file to your AI agent and ask it to import the songs — the agent will resolve artists, shows, and songs, creating records as needed (see [Import Workflow](docs/import.md) for details)

### 4. Start using it

Point your AI agent (e.g., Claude) to the `.claude/skills/` directory and tell it the path to your database file. The agent uses `jankenoboe` CLI commands to interact with the database. You can also use the CLI directly (see [CLI](#cli) below).

## CLI

The CLI uses subcommands organized by functionality. All commands output JSON to stdout. Set `JANKENOBOE_DB` to your database path before use.

### Querying

```bash
# Get a song by ID
jankenoboe get song 3b105bd4-c437-4720-a373-660bd5d68532 --fields id,name,artist_id

# Search artist by name (case-insensitive)
jankenoboe search artist --fields id,name --term '{"name": {"value": "minami", "match": "exact-i"}}'

# List songs by artist
jankenoboe search song --fields id,name --term '{"artist_id": {"value": "2196b222-ed04-4260-90c8-d18382bf8900"}}'

# Find duplicate artists
jankenoboe duplicates artist

# Find all shows where specific artists perform
jankenoboe shows-by-artist-ids --artist-ids artist-uuid-1,artist-uuid-2

# Get all songs by specific artists
jankenoboe songs-by-artist-ids --artist-ids artist-uuid-1,artist-uuid-2
```

### Learning (Spaced Repetition)

```bash
# Get songs due for review (--offset for look-ahead in seconds)
jankenoboe learning-due
jankenoboe learning-due --offset 7200  # due within next 2 hours

# Add songs to learning
jankenoboe learning-batch --song-ids 3b105bd4-c437-4720-a373-660bd5d68532

# Generate an HTML report of due songs (with show names, media URLs)
jankenoboe learning-song-review
jankenoboe learning-song-review --output ~/reports/review.html

# Level up specific learning records by ID (race-condition safe)
jankenoboe learning-song-levelup-ids --ids learning-uuid-1,learning-uuid-2

# Get learning records by song IDs
jankenoboe learning-by-song-ids --song-ids song-uuid-1,song-uuid-2

# Get learning stats per song (days spent learning)
jankenoboe learning-song-stats --song-ids song-uuid-1,song-uuid-2

# Level up a learning record
jankenoboe update learning bb9d3b38-9c28-4d11-aecd-6d2650724b98 --data '{"level": 8}'
```

### URL Percent-Encoding

String values in `--term` and `--data` are automatically URL percent-decoded. This avoids shell quoting issues with special characters like `'`, `"`, `(`, `)`, `&`, `!`, and spaces. Use the included Python helper to encode values:

```bash
# Encode a value
python3 tools/url_encode.py "it's a test"
# Output: it%27s%20a%20test

# Use in search
jankenoboe search artist --fields id,name --term '{"name":{"value":"it%27s%20a%20test"}}'

# Use in create
jankenoboe create artist --data '{"name":"Ado%27s%20Music"}'
```

Plain text (without `%` sequences) works unchanged. Keys and non-string values (numbers, booleans) are not decoded.

### Data Management

```bash
# Create an artist
jankenoboe create artist --data '{"name": "ChoQMay"}'

# Update a record
jankenoboe update song abc123 --data '{"artist_id": "new-artist-id"}'

# Delete a record
jankenoboe delete artist abc123

# Bulk reassign songs to a new artist
jankenoboe bulk-reassign --song-ids song1,song2 --new-artist-id correct-artist-id
```

**Tables:** `artist`, `show`, `song`, `play_history`, `learning`, `rel_show_song`

See the full [CLI Reference](docs/cli.md) for all commands, options, and query definitions.

## Documentation

- [AGENTS.md](AGENTS.md) - AI agent context: project summary, conventions, architecture
- [CLI Reference](docs/cli.md) - Command overview, operations coverage, exit codes
  - [Querying Commands](docs/cli-querying.md) - get, search, duplicates
  - [Learning Commands](docs/cli-learning.md) - learning-due, learning-batch
  - [Data Management Commands](docs/cli-data-management.md) - create, update, delete, bulk-reassign
- [Core Concepts](docs/concept.md) - Data model, relationships, and spaced repetition system
- [Import Workflow](docs/import.md) - AMQ song export import process and conflict resolution
- [Project Structure](docs/structure.md) - Directory layout, database schema, and dependencies
- [Development](docs/development.md) - Guidelines, testing, and code quality standards