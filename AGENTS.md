# AGENTS.md — Jankenoboe

> Development context for AI coding agents working on this repository. For project overview, installation, CLI usage, and agent skills, see [README.md](README.md).

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
├── commands/        # Subcommand implementations (split by category)
│   ├── mod.rs             # Module root, re-exports all public command functions
│   ├── querying.rs        # get, search, duplicates, shows-by-artist-ids, songs-by-artist-ids
│   ├── learning.rs        # learning-due, learning-batch, learning-song-review, levelup-ids, by-song-ids
│   ├── data_management.rs # create, update, delete, bulk-reassign
│   └── helpers.rs         # Shared utilities (e.g., json_value_to_sql)
├── db.rs            # Database connection management
├── easing.rs        # Fibonacci-based level_up_path generation
├── encoding.rs      # URL percent-decoding for --term and --data values
├── models.rs        # Business-layer validation wrappers
├── table_config.rs  # Centralized per-table field configuration (single source of truth)
├── lib.rs           # Library root
└── error.rs         # Error types and exit code mapping
```

### Module Responsibilities

| Module | Purpose | Key Functions |
|--------|---------|---------------|
| `main.rs` | CLI entry point and argument parsing | `main()`, Clap configuration |
| `commands/mod.rs` | Module root, re-exports all public command functions | Re-exports from submodules |
| `commands/querying.rs` | Read-only query commands | `cmd_get`, `cmd_search`, `cmd_duplicates`, `cmd_shows_by_artist_ids`, `cmd_songs_by_artist_ids` |
| `commands/learning.rs` | Spaced repetition commands | `cmd_learning_due`, `cmd_learning_batch`, `cmd_learning_song_review`, `cmd_learning_song_levelup_ids`, `cmd_learning_by_song_ids` |
| `commands/data_management.rs` | CRUD and bulk operations | `cmd_create`, `cmd_update`, `cmd_delete`, `cmd_bulk_reassign` |
| `commands/helpers.rs` | Shared utilities | `json_value_to_sql` |
| `db.rs` | Database connection management | Connection setup, datasource.db initialization |
| `easing.rs` | Fibonacci-based level_up_path generation | `generate_level_up_path()` |
| `encoding.rs` | URL percent-decoding for --term and --data values | `url_decode()` |
| `models.rs` | Business-layer validation wrappers | Table/field validation, parse helpers |
| `table_config.rs` | Centralized per-table field configuration | Single source of truth for selectable/searchable/creatable/updatable fields |
| `error.rs` | Error handling and exit codes | Custom error types, exit code mapping |

```
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
| `jankenoboe shows-by-artist-ids` | Get all shows where given artists have song performances |
| `jankenoboe songs-by-artist-ids` | Get all songs by given artists |

**Learning (Spaced Repetition):**
| Command | Description |
|---------|-------------|
| `jankenoboe learning-due` | Get songs due for review |
| `jankenoboe learning-batch` | Add songs to learning |
| `jankenoboe learning-song-review` | Generate HTML report of due songs |
| `jankenoboe learning-song-levelup-ids` | Level up specific learning records by ID |
| `jankenoboe learning-by-song-ids` | Get learning records by song IDs |
| `jankenoboe learning-song-stats` | Get learning stats per song (days spent learning) |

**Data Management:**
| Command | Description |
|---------|-------------|
| `jankenoboe create <table>` | Create a new record |
| `jankenoboe update <table> <id>` | Update a record |
| `jankenoboe delete <table> <id>` | Delete a record |
| `jankenoboe bulk-reassign` | Bulk reassign songs to a new artist |

### JankenSQLHub Integration

For full query syntax and parameter validation details, see the [JankenSQLHub usage skill](https://github.com/pandazy/jankensqlhub/blob/main/.claude/skills/using-jankensqlhub/SKILL.md).

- Use JankenSQLHub's parameter validation to prevent SQL injection
- `#[table]` with `enum` constraints for safe dynamic table names
- `~[fields]` with `enumif` for per-table field validation
- `enum`/`enumif` constraints to whitelist valid values per context
- `@param` defaults to string type — no need for `{"type": "string"}`
- Build queries using JankenSQLHub's `QueryDef` (JSON-configured)
- Execute queries using the appropriate runner (SQLite in this case)

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

Songs are imported from animemusicquiz.com JSON exports. The flow resolves artists (with namesake disambiguation), shows (by name + vintage), and songs (by name + artist) before creating play history records. See `docs/design/v1/import.md` for the full workflow.

## Documentation Map

| File | Content |
|------|---------|
| [README.md](README.md) | Quick start, installation, CLI overview, agent skills |
| [docs/cli.md](docs/cli.md) | CLI reference overview, operations coverage |
| [docs/cli-querying.md](docs/cli-querying.md) | Querying commands: get, search, duplicates |
| [docs/cli-learning.md](docs/cli-learning.md) | Learning commands: learning-due, learning-batch |
| [docs/cli-data-management.md](docs/cli-data-management.md) | Data management: create, update, delete, bulk-reassign |
| [docs/design/v1/concept.md](docs/design/v1/concept.md) | Data model, relationships, spaced repetition system |
| [docs/design/v1/import.md](docs/design/v1/import.md) | AMQ song export import process and conflict resolution |
| [docs/design/v1/structure.md](docs/design/v1/structure.md) | Directory layout, database schema, dependencies |
| [docs/design/v1/development.md](docs/design/v1/development.md) | Development guidelines, testing, code quality |

## Agent Skills (Claude)

The `.claude/skills/` directory contains [Claude Agent Skills](https://code.claude.com/docs/en/skills) for interacting with the database via CLI commands. See [README.md](README.md#agent-skills-claude) for the full skills table.

## Routines

To understand the repository's purpose, always check `README.md` first.

## Task Management

- Progress files: `_progress_<task_name>.md`
- When starting a task, read the progress file first (create if missing)
- Save progress if context window is exceeded, ensuring content is clear for another agent to resume
- When updating documents, check `README.md`, `AGENTS.md`, and `docs/*.md` for consistency

### TODO List Usage

When starting a new task, create a todo list to track progress:
- Use markdown checklist format: `- [ ]` for incomplete, `- [x]` for complete
- Include a comprehensive checklist of all steps needed
- Keep the list updated as you make progress

**Benefits:**
- Clear roadmap for implementation
- Progress tracking throughout the task
- Nothing gets forgotten or missed
- Users can see, monitor, and edit the plan
