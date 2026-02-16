# AGENTS.md — Jankenoboe

> Context file for AI coding agents working on this repository.

## Project Summary

Jankenoboe is an anime song learning system powered by a local SQLite database. It tracks songs encountered in quiz games on [animemusicquiz.com](https://animemusicquiz.com) and uses spaced repetition to help memorize them.

**Two access modes:**
- **Agent-guided SQL mode:** Non-technical users interact through AI agents (e.g., Claude with Agent Skills) that execute `sqlite3` queries directly against the database. No server or coding required.
- **HTTP API mode** *(under construction)*: A Rust HTTP service (Axum) with generic CRUD endpoints provides programmatic access for efficiency. It uses [JankenSQLHub](https://github.com/pandazy/jankensqlhub) for parameterized SQL query management with `enum`/`enumif` constraints for security.

**Database:** The SQLite database file lives outside the project directory (e.g., `~/db/datasource.db`). Initialize with `sqlite3 ~/db/datasource.db < docs/init-db.sql`. For the HTTP API, set `JANKENOBOE_DB` to the database path.

The project name combines "janken" (the creator's alias on AMQ) with "oboe" (覚え, memory/memorization in Japanese).

## Build, Test, and Run

```bash
cargo run                           # Start HTTP server on localhost:3000
cargo test                          # Run unit and integration tests
cargo clippy --fix --allow-dirty    # Fix compiler warnings (run after changes)
cargo fmt                           # Format code (run after changes)
```

**Prerequisites:** Rust 1.70+, JankenSQLHub library at `../jankensqlhub`

## Architecture

```
src/
├── main.rs       # Entry point, server init, route setup
├── handlers.rs   # HTTP request handlers (generic CRUD + special endpoints)
├── db.rs         # Database connection management
├── easing.rs     # Fibonacci-based level_up_path generation
├── models.rs     # Request/response structures
├── lib.rs        # Library root
└── error.rs      # Error types and HTTP error responses
```

### Data Model

```
artist ──< song >── rel_show_song ──< show
                         │
                    play_history
                         │
                     learning
```

**Tables:** `artist`, `show`, `song`, `play_history`, `learning`, `rel_show_song`

- Each **song** belongs to one **artist**
- **Shows** and **songs** have a many-to-many relationship via `rel_show_song`
- **Learning** tracks spaced repetition progress per song (20 levels, Fibonacci-based intervals)
- **Play history** records song encounters during quiz games

### API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/:table/:id` | Read by ID with dynamic field selection |
| GET | `/:table/search` | Search with table-specific filters |
| POST | `/:table` | Create records |
| PATCH | `/:table/:id` | Update records |
| DELETE | `/:table/:id` | Delete records (artist, song only) |
| GET | `/learning/due` | Spaced repetition due-for-review query |
| GET | `/:table/duplicates` | Data quality duplicate detection |
| POST | `/song/bulk-reassign` | Bulk song artist reassignment |
| POST | `/learning/batch` | Batch add songs to learning |

### JankenSQLHub Integration

- `#[table]` with `enum` constraints for safe dynamic table names
- `~[fields]` with `enumif` for per-table field validation
- `@param` defaults to string type
- All queries are JSON-configured via `QueryDef`

## Coding Conventions

- **Minimalism:** Apply Occam's Razor. Keep functions small and single-purpose.
- **Error handling:** Prefer explicit error handling over panics. Use pattern matching.
- **Types:** If a variable cannot be `None` in its flow, do not define it as `Option`.
- **HTTP:** Follow Axum best practices. Use proper HTTP status codes.
- **Naming:** Use meaningful names for functions, variables, and types.
- **Documentation:** Add doc comments for public APIs.

## Testing Conventions

- Assert exact values — no vague assertions like `option.is_some()` or `result.is_ok()`
- No conditional assertions or loops (except iterating arrays with obvious elements)
- Keep tests simple, readable, and not over-engineered
- Prioritize SQL injection security testing
- Test endpoints with various parameter combinations and error cases
- Do not use special math constants (PI, E) — they trigger linting errors

## Key Domain Concepts

### Spaced Repetition

Songs progress through 20 levels (stored 0–19, displayed 1–20). The `level_up_path` JSON array stores wait-days per level:

```
[1,1,1,1,1,1,1,2,3,5,7,13,19,32,52,84,135,220,355,574]
```

- **Level 0:** 5-minute warm-up before first review
- **Level 1–6:** 1-day intervals
- **Level 7+:** Fibonacci-based growth up to 574 days
- **Graduation:** `graduated = 1` when fully memorized
- **Re-learn:** Graduated songs restart at level 7 (display 8) by default

### Level Display Convention

Levels are 0-indexed in the database/API but displayed as 1-indexed to users. The `+1` transformation is display-only.

### AMQ Import Workflow

Songs are imported from animemusicquiz.com JSON exports. The flow resolves artists (with namesake disambiguation), shows (by name + vintage), and songs (by name + artist) before creating play history records. See `docs/import.md` for the full workflow.

## Documentation Map

| File | Content |
|------|---------|
| [README.md](README.md) | Quick start, API overview, agent skills |
| [docs/api.md](docs/api.md) | Full API reference with request/response formats |
| [docs/concept.md](docs/concept.md) | Data model, relationships, spaced repetition system |
| [docs/import.md](docs/import.md) | AMQ song export import process and conflict resolution |
| [docs/structure.md](docs/structure.md) | Directory layout, database schema, dependencies |
| [docs/development.md](docs/development.md) | Development guidelines, testing, code quality |

## Agent Skills (Claude)

The `skills/` directory contains [Claude Agent Skills](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/overview) for interacting with the database. Each skill supports HTTP mode (REST API on `localhost:3000`) and SQL mode (direct `sqlite3` CLI fallback).

| Skill | File | Description |
|-------|------|-------------|
| querying-jankenoboe | [skills/querying-jankenoboe/SKILL.md](skills/querying-jankenoboe/SKILL.md) | Search/read: artists, shows, songs, learning, due reviews, duplicates |
| learning-with-jankenoboe | [skills/learning-with-jankenoboe/SKILL.md](skills/learning-with-jankenoboe/SKILL.md) | Spaced repetition: batch add, level up/down, graduate, re-learn |
| maintaining-jankenoboe-data | [skills/maintaining-jankenoboe-data/SKILL.md](skills/maintaining-jankenoboe-data/SKILL.md) | CRUD: create/update/delete, bulk reassign, merge duplicates |
| reviewing-due-songs | [skills/reviewing-due-songs/SKILL.md](skills/reviewing-due-songs/SKILL.md) | Display due review songs with show names, song names, and media URLs |

## Task Management

- Progress files: `_progress_<task_name>.md`
- When starting a task, read the progress file first (create if missing)
- Save progress if context window is exceeded, ensuring content is clear for another agent to resume
- When updating documents, check `README.md`, `.clinerules`, and `docs/*.md` for consistency