# Project Structure

## Directory Layout

```
src/
├── main.rs          # Entry point, CLI argument parsing, subcommand dispatch
├── commands.rs      # Subcommand implementations (get, search, create, update, delete, etc.)
├── db.rs            # Database connection management
├── easing.rs        # Fibonacci-based level_up_path generation
├── encoding.rs      # URL percent-decoding for --term and --data values
├── models.rs        # Input/output structures and business-layer validation
├── table_config.rs  # Centralized per-table field configuration (single source of truth)
├── lib.rs           # Library root
└── error.rs         # Error types and exit code mapping

docs/
├── cli.md              # CLI reference overview and operations coverage
├── cli-querying.md     # Querying commands: get, search, duplicates
├── cli-learning.md     # Learning commands: learning-due, learning-batch
├── cli-data-management.md  # Data management: create, update, delete, bulk-reassign
├── concept.md          # Core concepts and data model
├── structure.md        # Project structure and database schema (this file)
├── development.md      # Development guidelines
├── import.md           # AMQ song import workflow
└── archived_tasks/     # Completed task documentation

skills/
├── querying-jankenoboe/SKILL.md       # Search/read: artists, shows, songs, learning, duplicates
├── learning-with-jankenoboe/SKILL.md  # Spaced repetition: batch add, level up/down, graduate
├── maintaining-jankenoboe-data/SKILL.md  # CRUD: create/update/delete, bulk reassign, merge
└── reviewing-due-songs/SKILL.md       # Display due review songs for practice

templates/
└── learning-song-review.html  # HTML template for due song review report

tools/
└── url_encode.py        # Python helper to URL percent-encode values for CLI args

e2e/
├── Dockerfile           # Multi-stage Docker build for e2e testing
└── run_tests.sh         # Shell-based e2e test suite

.github/workflows/
├── e2e.yml              # CI: e2e tests, unit tests, clippy (on push/PR to main)
└── release.yml          # CD: cross-platform binary builds + GitHub Release (on v* tags)

install.sh               # Cross-platform installer script
uninstall.sh             # Cross-platform uninstaller script
release.md               # Release notes (used as GitHub Release body on v* tags)
Makefile                 # Docker e2e test targets (make e2e, make clean-e2e)
.dockerignore            # Excludes target/, .git/, etc. from Docker build context
```

## Database Schema

The database is a SQLite file stored **outside** the project directory (e.g., `~/db/datasource.db`). Initialize with:

```bash
sqlite3 ~/db/datasource.db < docs/init-db.sql
```

See [docs/init-db.sql](init-db.sql) for the full schema definition.

### Tables

**artist** (10,268 records)
| Column | Type | Description |
|--------|------|-------------|
| id | TEXT | UUID primary key |
| name | TEXT | Artist name |
| name_context | TEXT | Additional context |
| created_at | INTEGER | Unix timestamp |
| updated_at | INTEGER | Unix timestamp |
| status | INTEGER | 0=normal, 1=deleted |

**show** (8,545 records)
| Column | Type | Description |
|--------|------|-------------|
| id | TEXT | UUID primary key |
| name | TEXT | English name |
| name_romaji | TEXT | Romaji name |
| vintage | TEXT | Season (e.g., "Spring 2010") |
| s_type | TEXT | Type (TV, Movie, OVA, etc.) |
| created_at | INTEGER | Unix timestamp |
| updated_at | INTEGER | Unix timestamp |
| status | INTEGER | 0=normal, 1=deleted |

**song** (23,313 records)
| Column | Type | Description |
|--------|------|-------------|
| id | TEXT | UUID primary key |
| name | TEXT | Song title |
| name_context | TEXT | Additional context |
| artist_id | TEXT | FK to artist |
| created_at | INTEGER | Unix timestamp |
| updated_at | INTEGER | Unix timestamp |
| status | INTEGER | 0=normal, 1=deleted |

**rel_show_song** (24,953 records)
| Column | Type | Description |
|--------|------|-------------|
| show_id | TEXT | FK to show |
| song_id | TEXT | FK to song |
| media_url | TEXT | Optional media link |
| created_at | INTEGER | Unix timestamp |

*Unique constraint on (show_id, song_id)*

**play_history** (60,093 records)
| Column | Type | Description |
|--------|------|-------------|
| id | TEXT | UUID primary key |
| show_id | TEXT | FK to show |
| song_id | TEXT | FK to song |
| created_at | INTEGER | Unix timestamp |
| media_url | TEXT | Optional media link |
| status | INTEGER | 0=normal, 1=deleted |

**learning** (6,089 records)
| Column | Type | Description |
|--------|------|-------------|
| id | TEXT | UUID primary key |
| song_id | TEXT | FK to song |
| level | INTEGER | Current level (0-19+) |
| created_at | INTEGER | Unix timestamp |
| updated_at | INTEGER | Unix timestamp |
| last_level_up_at | INTEGER | Unix timestamp |
| level_up_path | TEXT | JSON array of wait days (memory curve) |
| graduated | INTEGER | 0=in progress, 1=graduated |

### Indexes

- `idx_learning_song_id` on `learning(song_id)`
- `idx_rel_show_song_song_id` on `rel_show_song(song_id)`
- `idx_rel_show_song_show_id` on `rel_show_song(show_id)`

## Dependencies

- **Clap** - CLI argument parsing
- **SQLite** - Database (via rusqlite)
- **JankenSQLHub** - Parameterized SQL query management
- **Serde** - JSON serialization
- **UUID** - Record ID generation