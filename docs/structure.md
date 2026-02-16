# Project Structure

## Directory Layout

```
src/
├── main.rs          # Entry point, server initialization, route setup
├── handlers.rs      # HTTP request handlers
├── db.rs            # Database connection management
├── easing.rs        # Fibonacci-based level_up_path generation
├── models.rs        # Request/response structures
└── error.rs         # Error types and HTTP error responses

docs/
├── concept.md       # Core concepts and data model
├── structure.md     # Project structure and database schema
├── development.md   # Development guidelines
└── archived_tasks/  # Completed task documentation

tests/
└── integration_test.rs  # Integration tests
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

- **Axum** - HTTP framework
- **SQLite** - Database (via rusqlite)
- **JankenSQLHub** - Parameterized SQL query management
- **Serde** - JSON serialization
- **Tokio** - Async runtime