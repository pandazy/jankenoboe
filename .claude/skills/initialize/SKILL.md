---
name: initialize
description: Set up the Jankenoboe environment before running any CLI commands. Checks CLI installation, verifies database path, and guides first-time database creation. All other jankenoboe skills depend on this skill.
---

## Step 1: Verify CLI Installation

The `jankenoboe` CLI must be installed. Check if it's available:

```bash
jankenoboe --version
```

If not installed, see the [README installation instructions](../../../README.md#installation).

## Step 2: Check Database Path

Check if `JANKENOBOE_DB` is already set:
```bash
echo $JANKENOBOE_DB
```

- **If it prints a path** (e.g., `/Users/you/db/datasource.db`): the database is already configured — skip to [Step 3](#step-3-verify-database-exists).
- **If it prints nothing (empty)**: ask the user for the database path, then either:
  - Export it for the session: `export JANKENOBOE_DB=/path/to/datasource.db`
  - Or prefix each command: `JANKENOBOE_DB=/path/to/datasource.db jankenoboe ...`

> **Do NOT set `JANKENOBOE_DB` if it is already set.** Only ask the user when the variable is empty.

## Step 3: Verify Database Exists

Check if the database file exists at the configured path:

```bash
ls -la "$JANKENOBOE_DB"
```

- **If the file exists**: the environment is ready — proceed with `jankenoboe` commands.
- **If the file does NOT exist**: this is a first-time setup — continue to [Step 4](#step-4-first-time-database-creation-only).

## Step 4: First-Time Database Creation Only

> ⚠️ **WARNING:** Only run this step if the database file does NOT exist. Running `init-db.sql` against an existing database will corrupt or destroy existing data.

Create the database from the schema:

```bash
mkdir -p "$(dirname "$JANKENOBOE_DB")"
sqlite3 "$JANKENOBOE_DB" < docs/init-db.sql
```

This creates all tables, indexes, and constraints. See [docs/init-db.sql](../../../docs/init-db.sql) for the full schema.

### Import Initial Song Data (Optional)

After creating a fresh database, the user may want to import songs from [animemusicquiz.com](https://animemusicquiz.com):

1. Export song list as JSON from AMQ (see [sample export](../../../docs/design/v1/amq_song_export-small.json))
2. Use the [importing-amq-songs skill](../importing-amq-songs/SKILL.md) to import the data

See [docs/design/v1/import.md](../../../docs/design/v1/import.md) for the full import workflow.

## Prerequisites

- **SQLite 3** — the database engine
  - macOS: `brew install sqlite` (or pre-installed)
  - Ubuntu/Debian: `sudo apt install sqlite3`