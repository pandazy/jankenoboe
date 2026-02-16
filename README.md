# Janken - Anime Music Quiz Learning API

A Rust HTTP service that provides API support for learning anime songs to improve performance in anime music quiz games like [animemusicquiz.com](https://animemusicquiz.com).

## Quick Start

```bash
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

## Documentation

- [API Reference](docs/api.md) - Endpoint list, request/response formats, and planned APIs
- [Core Concepts](docs/concept.md) - Data model, relationships, and spaced repetition system
- [Import Workflow](docs/import.md) - AMQ song export import process and conflict resolution
- [Project Structure](docs/structure.md) - Directory layout, database schema, and dependencies
- [Development](docs/development.md) - Guidelines, testing, and code quality standards
