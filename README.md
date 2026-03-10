# Jankenoboe - Anime Song Learning System

An anime song learning system that tracks songs from [animemusicquiz.com](https://animemusicquiz.com) and uses spaced repetition to help memorize them. The name combines "janken" (the creator's alias on AMQ) with "oboe" (覚え, memory/memorization in Japanese).

Users interact through AI agents (e.g., Claude with [Agent Skills](#agent-skills-claude)) that call `jankenoboe` CLI commands. The CLI is a Rust binary that provides fast, validated database operations with JSON output.

## Agent Skills (Claude)

The `.claude/skills/` directory contains [Claude Agent Skills](https://code.claude.com/docs/en/skills) for interacting with the Jankenoboe database. Each skill uses the `jankenoboe` CLI binary for validated, fast operations with JSON output.

| Skill | Description |
|-------|-------------|
| [initialize](.claude/skills/initialize/SKILL.md) | Set up environment: verify CLI, check `JANKENOBOE_DB`, guide first-time DB creation (referenced by all other skills) |
| [querying-jankenoboe](.claude/skills/querying-jankenoboe/SKILL.md) | Search and read artists, shows, songs, learning records, due reviews, duplicates |
| [learning-with-jankenoboe](.claude/skills/learning-with-jankenoboe/SKILL.md) | Spaced repetition: add songs, level up/down, graduate, check due reviews |
| [maintaining-jankenoboe-data](.claude/skills/maintaining-jankenoboe-data/SKILL.md) | CRUD operations: create/update/delete records, bulk reassign, merge duplicates |
| [reviewing-due-songs](.claude/skills/reviewing-due-songs/SKILL.md) | Display due review songs with show names, song names, and media URLs |
| [importing-amq-songs](.claude/skills/importing-amq-songs/SKILL.md) | Import AMQ song exports: resolve artists, shows, songs, create play history |

## Installation

### Option 1: Install from GitHub (recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/install.sh | sh
```

This detects your OS/architecture and installs `jankenoboe` to `~/.local/bin/`. Make sure `~/.local/bin` is in your `PATH`.

### Option 2: Install via Cargo

```bash
cargo install --git https://github.com/pandazy/jankenoboe.git
```

### Option 3: Build from source

```bash
git clone https://github.com/pandazy/jankenoboe.git
cd jankenoboe
cargo build --release
cp target/release/jankenoboe ~/.local/bin/
```

### Upgrading

Re-run the install script — it always fetches the latest release:

```bash
curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/install.sh | sh
```

### Uninstalling

```bash
curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/uninstall.sh | sh
```

Or if installed via Cargo: `cargo uninstall jankenoboe`

## Setup

For first-time setup (database creation, environment variable, initial data import), follow the [initialize skill](.claude/skills/initialize/SKILL.md). It walks through prerequisites, database path configuration, and optional song import.

If you're using an AI agent (e.g., Claude), point it to the `.claude/skills/` directory — the agent will follow the initialize skill automatically before running any commands.

## CLI

The CLI uses subcommands organized by functionality. All commands output JSON to stdout. Set `JANKENOBOE_DB` to your database path before use.

### Querying

```bash
# Get a song by ID
jankenoboe get song 3b105bd4-c437-4720-a373-660bd5d68532 --fields id,name,artist_id

# Get multiple artists by IDs
jankenoboe batch-get artist --ids uuid-1,uuid-2,uuid-3 --fields id,name

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

# Directly graduate specific learning records (set level to max and graduated)
jankenoboe learning-song-graduate-ids --ids learning-uuid-1,learning-uuid-2

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
- [Core Concepts](docs/design/v1/concept.md) - Data model, relationships, and spaced repetition system
- [Import Workflow](docs/design/v1/import.md) - AMQ song export import process and conflict resolution
- [Project Structure](docs/design/v1/structure.md) - Directory layout, database schema, and dependencies
- [Development](docs/design/v1/development.md) - Guidelines, testing, and code quality standards
