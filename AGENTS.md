# AGENTS.md — Jankenoboe

> Context file for AI coding agents working on this repository.

## Project Summary

Jankenoboe is an anime song learning system powered by a local SQLite database. It tracks songs encountered in quiz games on [animemusicquiz.com](https://animemusicquiz.com) and uses spaced repetition to help memorize them.

Non-technical users interact through AI agents (e.g., Claude with Agent Skills) that call `jankenoboe` CLI commands. The CLI is a Rust binary that provides fast, validated database operations with JSON output, using [JankenSQLHub](https://github.com/pandazy/jankensqlhub) for parameterized SQL query management with `enum`/`enumif` constraints for security.

**Database:** The SQLite database file lives outside the project directory (e.g., `~/db/datasource.db`). Initialize with `sqlite3 ~/db/datasource.db < docs/init-db.sql`. Set `JANKENOBOE_DB` to the database path before using the CLI.

The project name combines "janken" (the creator's alias on AMQ) with "oboe" (覚え, memory/memorization in Japanese).

## Build, Test, and Run

```bash
cargo build --release                # Build optimized binary
cargo install --path .               # Install to ~/.cargo/bin/
cargo test                           # Run unit and integration tests
cargo clippy --fix --allow-dirty     # Fix compiler warnings (run after changes)
cargo fmt                            # Format code (run after changes)
make e2e                             # Run e2e tests in Docker (build + test)
make clean-e2e                       # Remove e2e Docker image
# CI: GitHub Actions runs e2e on Linux + macOS on every pull request
# CD: Update release.md, then push a v* tag to trigger cross-platform release builds
# The release workflow uses release.md as the GitHub Release body
git tag v1.0.0 && git push origin v1.0.0
```

### Upgrading

Re-run the install script to upgrade — it always fetches and installs the latest release, overwriting the existing binary:

```bash
curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/install.sh | sh
```

### Uninstalling

```bash
sh uninstall.sh                      # Remove binary from ~/.local/bin and ~/.cargo/bin
cargo uninstall jankenoboe           # If installed via cargo
```

**Prerequisites:** Rust 1.85+, [JankenSQLHub](https://github.com/pandazy/jankensqlhub) (installed via crates.io), Docker (for e2e tests)

## Architecture

```
src/
├── main.rs          # Entry point, CLI argument parsing, subcommand dispatch
├── commands.rs      # Subcommand implementations
├── db.rs            # Database connection management
├── easing.rs        # Fibonacci-based level_up_path generation
├── encoding.rs      # URL percent-decoding for --term and --data values
├── models.rs        # Business-layer validation wrappers
├── table_config.rs  # Centralized per-table field configuration (single source of truth)
├── lib.rs           # Library root
└── error.rs         # Error types and exit code mapping

tools/
└── url_encode.py # Python helper to URL percent-encode values for CLI args

e2e/
├── Dockerfile    # Multi-stage Docker build for e2e testing
└── run_tests.sh  # Shell-based e2e test suite
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

### CLI Commands

Commands are organized by category:

**Querying:**
| Command | Description |
|---------|-------------|
| `jankenoboe get <table> <id>` | Get record by ID |
| `jankenoboe search <table>` | Search with table-specific filters |
| `jankenoboe duplicates <table>` | Find duplicate records by name |

**Learning (Spaced Repetition):**
| Command | Description |
|---------|-------------|
| `jankenoboe learning-due` | Get songs due for review |
| `jankenoboe learning-batch` | Add songs to learning |
| `jankenoboe learning-song-review` | Generate HTML report of due songs |
| `jankenoboe learning-song-levelup-ids` | Level up specific learning records by ID |

**Data Management:**
| Command | Description |
|---------|-------------|
| `jankenoboe create <table>` | Create a new record |
| `jankenoboe update <table> <id>` | Update a record |
| `jankenoboe delete <table> <id>` | Delete a record |
| `jankenoboe bulk-reassign` | Bulk reassign songs to a new artist |

### JankenSQLHub Integration

- `#[table]` with `enum` constraints for safe dynamic table names
- `~[fields]` with `enumif` for per-table field validation
- `@param` defaults to string type
- All queries are JSON-configured via `QueryDef`

## Coding Conventions

- **Minimalism:** Apply Occam's Razor. Keep functions small and single-purpose.
- **Error handling:** Prefer explicit error handling over panics. Use pattern matching.
- **Types:** If a variable cannot be `None` in its flow, do not define it as `Option`.
- **CLI output:** JSON to stdout for success, JSON to stderr for errors. Exit code 0 for success, 1 for errors.
- **Naming:** Use meaningful names for functions, variables, and types.
- **Documentation:** Add doc comments for public APIs.

## Testing Conventions

- Assert exact values — no vague assertions like `option.is_some()` or `result.is_ok()`
- No conditional assertions or loops (except iterating arrays with obvious elements)
- Keep tests simple, readable, and not over-engineered
- Prioritize SQL injection security testing
- Test CLI commands with various parameter combinations and error cases
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

Levels are 0-indexed in the database/CLI but displayed as 1-indexed to users. The `+1` transformation is display-only.

### AMQ Import Workflow

Songs are imported from animemusicquiz.com JSON exports. The flow resolves artists (with namesake disambiguation), shows (by name + vintage), and songs (by name + artist) before creating play history records. See `docs/import.md` for the full workflow.

## Documentation Map

| File | Content |
|------|---------|
| [README.md](README.md) | Quick start, installation, CLI overview, agent skills |
| [docs/cli.md](docs/cli.md) | CLI reference overview, operations coverage |
| [docs/cli-querying.md](docs/cli-querying.md) | Querying commands: get, search, duplicates |
| [docs/cli-learning.md](docs/cli-learning.md) | Learning commands: learning-due, learning-batch |
| [docs/cli-data-management.md](docs/cli-data-management.md) | Data management: create, update, delete, bulk-reassign |
| [docs/concept.md](docs/concept.md) | Data model, relationships, spaced repetition system |
| [docs/import.md](docs/import.md) | AMQ song export import process and conflict resolution |
| [docs/structure.md](docs/structure.md) | Directory layout, database schema, dependencies |
| [docs/development.md](docs/development.md) | Development guidelines, testing, code quality |

## Agent Skills (Claude)

The `skills/` directory contains [Claude Agent Skills](https://platform.claude.com/docs/en/agents-and-tools/agent-skills/overview) for interacting with the database via CLI commands.

| Skill | File | Description |
|-------|------|-------------|
| querying-jankenoboe | [skills/querying-jankenoboe/SKILL.md](skills/querying-jankenoboe/SKILL.md) | Search/read: artists, shows, songs, learning, due reviews, duplicates |
| learning-with-jankenoboe | [skills/learning-with-jankenoboe/SKILL.md](skills/learning-with-jankenoboe/SKILL.md) | Spaced repetition: batch add, level up/down, graduate, re-learn |
| maintaining-jankenoboe-data | [skills/maintaining-jankenoboe-data/SKILL.md](skills/maintaining-jankenoboe-data/SKILL.md) | CRUD: create/update/delete, bulk reassign, merge duplicates |
| reviewing-due-songs | [skills/reviewing-due-songs/SKILL.md](skills/reviewing-due-songs/SKILL.md) | Display due review songs with show names, song names, and media URLs |
| importing-amq-songs | [skills/importing-amq-songs/SKILL.md](skills/importing-amq-songs/SKILL.md) | Import AMQ song exports: resolve artists, shows, songs, create play history |

## Task Management

- Progress files: `_progress_<task_name>.md`
- When starting a task, read the progress file first (create if missing)
- Save progress if context window is exceeded, ensuring content is clear for another agent to resume
- When updating documents, check `README.md`, `AGENTS.md`, `.clinerules`, and `docs/*.md` for consistency
