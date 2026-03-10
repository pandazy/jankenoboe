# Progress: Agent Skills Creation

## Task
Create Claude Agent Skills corresponding to Jankenoboe APIs with:
- Database path prompt at the beginning of each interaction
- HTTP API mode (when local server is running on localhost:3000)
- Raw SQL fallback mode (direct sqlite3 CLI when server is down)

## Status: Complete ✅

## Steps
- [x] Analyze project structure and APIs
- [x] Read Claude Agent Skills documentation (best practices)
- [x] Create skill: querying-jankenoboe (read/search operations)
- [x] Create skill: learning-with-jankenoboe (spaced repetition workflow)
- [x] Create skill: maintaining-jankenoboe-data (CRUD mutations, data quality)
- [x] Update documentation (README)
- [x] Verify skill structure

## Design Decisions
- **Three focused skills** instead of one monolithic skill (better discoverability, lower token cost per skill)
- **Gerund naming** per Claude Agent Skills best practices
- **Third-person descriptions** as required by the platform
- **Dual mode**: Each skill documents both HTTP API calls and equivalent raw SQL
- **Database path**: Each skill instructs Claude to ask for the SQLite path before any operation

## Files Created
- `skills/querying-jankenoboe/SKILL.md` — Search/read: artists, shows, songs, learning, due reviews, duplicates
- `skills/learning-with-jankenoboe/SKILL.md` — Spaced repetition: batch add, level up/down, graduate, re-learn
- `skills/maintaining-jankenoboe-data/SKILL.md` — CRUD: create/update/delete, bulk reassign, merge duplicates
- `_progress_agent_skills.md` — This progress file

## Files Updated
- `README.md` — Added Agent Skills section with skill table and access mode description