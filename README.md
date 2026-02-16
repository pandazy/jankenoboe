# Jankenoboe - Anime Song Learning System

An anime song learning system that tracks songs from [animemusicquiz.com](https://animemusicquiz.com) and uses spaced repetition to help memorize them. The name combines "janken" (the creator's alias on AMQ) with "oboe" (覚え, memory/memorization in Japanese).

**Two ways to use it:**
- **With an AI agent:** Non-technical users can interact through AI agents (e.g., Claude with [Agent Skills](#agent-skills-claude)) that run `sqlite3` queries directly — no server needed.
- **With the HTTP API:** A Rust service provides REST endpoints for programmatic access and efficiency.

## Prerequisites

- **SQLite 3** — required for both access modes
  - macOS: `brew install sqlite` (or pre-installed)
  - Ubuntu/Debian: `sudo apt install sqlite3`
  - Windows: download from [sqlite.org](https://www.sqlite.org/download.html)
- **Rust 1.70+** — only needed for the HTTP API mode
- **JankenSQLHub** — cloned at `../jankensqlhub` (only for HTTP API mode)

## Setup

### 1. Create the database

The database file lives **outside** the project directory (e.g., `~/db/datasource.db`). Create it from the schema:

```bash
mkdir -p ~/db
sqlite3 ~/db/datasource.db < docs/init-db.sql
```

This creates all tables, indexes, and constraints. See [docs/init-db.sql](docs/init-db.sql) for the full schema.

### 2. Start using it

**Agent mode (no server needed):** Point your AI agent (e.g., Claude) to the `skills/` directory and tell it the path to your database file. The agent will run `sqlite3` queries directly.

**HTTP API mode** *(under construction)*:

```bash
export JANKENOBOE_DB=~/db/datasource.db
cargo run
```

The service starts on `http://localhost:3000`.

## API

The API uses **generic CRUD endpoints** powered by [JankenSQLHub](https://github.com/pandazy/jankensqlhub) to minimize endpoint count while maintaining security through `enum`/`enumif` parameter validation.

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/:table/:id` | Get record by ID |
| GET | `/:table/search` | Search with table-specific filters |
| POST | `/:table` | Create a new record |
| PATCH | `/:table/:id` | Update a record |
| DELETE | `/:table/:id` | Delete a record |
| GET | `/learning/due` | Get songs due for review |
| GET | `/:table/duplicates` | Find duplicate records |
| POST | `/song/bulk-reassign` | Bulk reassign songs to new artist |
| POST | `/learning/batch` | Add songs to learning system |

**Tables:** `artist`, `show`, `song`, `play_history`, `learning`, `rel_show_song`

**Examples:**

```bash
# Get a song by ID
curl "http://localhost:3000/song/3b105bd4-c437-4720-a373-660bd5d68532?fields=id,name,artist_id"

# Search artist by name
curl "http://localhost:3000/artist/search?fields=id,name&name=minami"

# List songs by artist
curl "http://localhost:3000/song/search?fields=id,name&artist_id=2196b222-ed04-4260-90c8-d18382bf8900"

# Add songs to learning
curl -X POST "http://localhost:3000/learning/batch" \
  -H "Content-Type: application/json" \
  -d '{"song_ids": ["3b105bd4-c437-4720-a373-660bd5d68532"]}'

# Level up a learning record
curl -X PATCH "http://localhost:3000/learning/bb9d3b38-9c28-4d11-aecd-6d2650724b98" \
  -H "Content-Type: application/json" \
  -d '{"level": 8}'
```

## Agent Skills (Claude)

The `skills/` directory contains [Claude Agent Skills](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/overview) for interacting with the Jankenoboe database. Each skill asks for the SQLite database path upfront and supports two access modes:

- **HTTP mode**: Uses the REST API when the local server is running (`localhost:3000`)
- **SQL mode**: Falls back to direct `sqlite3` CLI queries when the server is unavailable

| Skill | Description |
|-------|-------------|
| [querying-jankenoboe](skills/querying-jankenoboe/SKILL.md) | Search and read artists, shows, songs, learning records, due reviews, duplicates |
| [learning-with-jankenoboe](skills/learning-with-jankenoboe/SKILL.md) | Spaced repetition: add songs, level up/down, graduate, check due reviews |
| [maintaining-jankenoboe-data](skills/maintaining-jankenoboe-data/SKILL.md) | CRUD operations: create/update/delete records, bulk reassign, merge duplicates |
| [reviewing-due-songs](skills/reviewing-due-songs/SKILL.md) | Display due review songs with show names, song names, and media URLs |

## Documentation

- [AGENTS.md](AGENTS.md) - AI agent context: project summary, conventions, architecture
- [API Reference](docs/api.md) - Endpoint list, request/response formats, and planned APIs
- [Core Concepts](docs/concept.md) - Data model, relationships, and spaced repetition system
- [Import Workflow](docs/import.md) - AMQ song export import process and conflict resolution
- [Project Structure](docs/structure.md) - Directory layout, database schema, and dependencies
- [Development](docs/development.md) - Guidelines, testing, and code quality standards
