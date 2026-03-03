# Task: Initialize Skill & Documentation Reorganization

**Date:** 2026-03-02

## Summary

Extracted the `JANKENOBOE_DB` environment check from all 5 agent skills into a dedicated `initialize` skill, and reorganized design documentation into versioned folders to reduce agent noise.

## Changes

### New: `initialize` Skill
- Created `.claude/skills/initialize/SKILL.md` — a 4-step setup flow:
  1. Verify CLI installation
  2. Check `JANKENOBOE_DB` (with "do NOT set if already set" guard)
  3. Verify database file exists
  4. First-time database creation only (with corruption warning)
- Includes prerequisites (SQLite 3) and optional initial data import link

### Updated Skills (5 files)
All skills replaced their inline "Setup" sections with a single-line reference:
- `.claude/skills/querying-jankenoboe/SKILL.md`
- `.claude/skills/learning-with-jankenoboe/SKILL.md`
- `.claude/skills/maintaining-jankenoboe-data/SKILL.md`
- `.claude/skills/reviewing-due-songs/SKILL.md`
- `.claude/skills/importing-amq-songs/SKILL.md`

### Documentation Reorganization
- Moved `docs/structure.md`, `docs/development.md`, `docs/import.md`, `docs/concept.md` → `docs/design/v1/`
- Moved and renamed `docs/amq_song_export-sample.json` → `docs/design/v1/amq_song_export-small.json`
- Updated all references in `AGENTS.md`, `README.md`, `.claude/skills/initialize/SKILL.md`, `docs/design/v1/import.md`

### README.md Simplification
- Removed redundant Prerequisites, detailed Upgrading/Uninstalling sections, and 4-step Setup walkthrough
- Replaced with compact Installation section and a Setup section that links to the initialize skill
- Added `initialize` to the skills table

### AGENTS.md Updates
- Added `initialize` to the Agent Skills table
- Updated Documentation Map paths to `docs/design/v1/`

## Files Modified
- `.claude/skills/initialize/SKILL.md` (new)
- `.claude/skills/querying-jankenoboe/SKILL.md`
- `.claude/skills/learning-with-jankenoboe/SKILL.md`
- `.claude/skills/maintaining-jankenoboe-data/SKILL.md`
- `.claude/skills/reviewing-due-songs/SKILL.md`
- `.claude/skills/importing-amq-songs/SKILL.md`
- `AGENTS.md`
- `README.md`
- `docs/design/v1/import.md`

## Files Moved
- `docs/structure.md` → `docs/design/v1/structure.md`
- `docs/development.md` → `docs/design/v1/development.md`
- `docs/import.md` → `docs/design/v1/import.md`
- `docs/concept.md` → `docs/design/v1/concept.md`
- `docs/amq_song_export-sample.json` → `docs/design/v1/amq_song_export-small.json`

## Files Deleted
- `.claude/skills/env-check/SKILL.md` (intermediate, replaced by initialize)