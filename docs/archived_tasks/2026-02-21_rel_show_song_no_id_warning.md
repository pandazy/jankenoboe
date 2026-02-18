# Task: Add `rel_show_song` No-ID Warning and Refactor Import Docs

**Date:** 2026-02-21

## Problem

AI models were attempting to search the `rel_show_song` table using an `id` field (e.g., `--fields id,show_id,song_id`), which fails because `rel_show_song` has no `id` column. Unlike all other tables in the schema, it uses a composite unique constraint on `(show_id, song_id)`.

## Changes Made

### 1. `skills/importing-amq-songs/SKILL.md` — Step 4

Added a warning block before the search command:

> ⚠️ The `rel_show_song` table has **no `id` column**. Do NOT include `id` in `--fields`. Use `show_id`, `song_id`, `media_url`, or `created_at`.

### 2. `docs/import.md` — Refactored to Focus on Concepts

Since the skill file (`skills/importing-amq-songs/SKILL.md`) already contains all step-by-step CLI commands, `docs/import.md` was refactored to avoid duplication:

- Added a link to the skill file for CLI command details
- Added a "Show–Song Link (`rel_show_song`)" subsection under Entity Matching Rules with the no-`id` warning
- Replaced verbose step-by-step CLI sections with a concise 5-step conceptual summary
- Removed the "Database Operations Required" section (fully covered by the skill)
- Simplified Conflict Resolution and Data Quality sections to explain concepts without repeating CLI examples

## Files Modified

- `skills/importing-amq-songs/SKILL.md`
- `docs/import.md`