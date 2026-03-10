# Session: Documentation & API Planning
**Date:** February 14, 2026

## Summary
Explored datasource.db structure and documented import workflows, data quality management, and spaced repetition filtering rules.

## Completed Tasks

### 1. Created Import Workflow Documentation (`docs/import.md`)
- Documented AMQ song export format
- Defined entity matching rules:
  - **Show:** `name` + `vintage` (season)
  - **Artist:** `name` (with namesake handling)
  - **Song:** `name` + `artist_id`
- Created import processing flow diagram
- Documented conflict resolution for namesake artists

### 2. Data Quality Management
- Added duplicate detection API design (`/artist/duplicates`, `/show/duplicates`, `/song/duplicates`)
- Song ownership transfer via `PATCH /song/:id` and `POST /song/bulk-reassign`
- Artist merging workflow documentation

### 3. Real Data Examples
- Used "Minami" as real-world namesake example
  - Minami (ID: 14b7393a) - 5 songs: "Beautiful Soldier", "One Unit", "Patria", "SWITCH", "illuminate"
  - Minami (ID: 6136d7b3) - 3 songs: "Kawaki o Ameku", "Rude Lose Dance", "illuminate"

### 4. Due for Review Filter (`docs/concept.md`)
Documented the standard query for finding songs due for review:

```sql
graduated = 0 AND (
    -- Level 0 with last_level_up_at set: wait 300 seconds (5 minutes)
    (last_level_up_at > 0 AND level = 0 AND now >= last_level_up_at + 300)
    OR
    -- Level 0 newly created: fall back to updated_at + 300 seconds
    (last_level_up_at = 0 AND level = 0 AND now >= updated_at + 300)
    OR
    -- Level > 0: use level_up_path[level] days
    (level > 0 AND (level_up_path[level] * 86400 + last_level_up_at) <= now)
)
```

**Key insights:**
- Level 0 = newly added songs, 5-minute warm-up period
- Level > 0 uses `level_up_path[level]` days converted to seconds (× 86400)
- Current due count: 13 songs

### 5. Database Statistics
From datasource.db:
- Artists: 10,268
- Shows: 8,545
- Songs: 23,313
- rel_show_song: 24,953
- play_history: 60,093
- learning: 6,089

### 6. API Minimization with JankenSQLHub
Redesigned the API around **8 generic endpoints** (down from ~20) by leveraging JankenSQLHub's `#[table]`, `~[fields]`, `enum`, and `enumif` features:

| # | Method | Endpoint | Description |
|---|--------|----------|-------------|
| 1 | GET | `/:table/:id` | Get record by ID |
| 2 | GET | `/:table/search` | Search with table-specific filters |
| 3 | POST | `/:table` | Create a new record |
| 4 | PATCH | `/:table/:id` | Update a record |
| 5 | DELETE | `/:table/:id` | Delete a record |
| 6 | GET | `/learning/due` | Get songs due for review |
| 7 | GET | `/:table/duplicates` | Find duplicate records |
| 8 | POST | `/song/bulk-reassign` | Bulk reassign songs |

**Key design decisions:**
- `enum` constrains valid table names, `enumif` constrains valid fields per table
- `@param` defaults to string type — no need for `{"type": "string"}`
- Name range constraints based on actual data: artist `[1, 800]` (grouped special units can have ~50 performers), show/song `[1, 300]`
- All operations (import, learning, data quality) are covered by the 8 endpoints — see Operations Coverage in `docs/api.md`

### 7. Data Insights
- **Max level for graduated songs:** 19 (all 4,713 graduated songs)
- **Longest names:** artist 244 chars (special unit), song 160 chars, show 115 chars

## Files Modified
- `docs/api.md` - Complete rewrite with minimized generic endpoints, JankenSQLHub query definitions, operations coverage
- `docs/import.md` - Updated endpoint references to new generic patterns, added `POST /rel_show_song`
- `docs/concept.md` - Added "Due for Review Filter" section
- `README.md` - Updated API section with endpoint table, new curl examples
- `.clinerules` - Updated project context, architecture, module structure, JankenSQLHub integration notes

## Next Steps
1. Implement the 8 generic endpoints with JankenSQLHub query definitions
2. Create JSON query definition files for all queries
3. Write integration tests for each endpoint
4. Handle special behaviors (auto-timestamp, `last_level_up_at` on level change)
