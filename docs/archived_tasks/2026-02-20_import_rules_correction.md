# Task: Import Rules Correction

**Date:** 2026-02-20
**Status:** Completed

## Summary

Corrected and improved the AMQ song import workflow in `docs/import.md` with three main changes:

1. **Show matching logic clarified**: AMQ's `animeNames.english` is always compared against our `name` field for primary matching using `exact-i` (case-insensitive). The `romaji` name is optional supplementary info stored in `name_romaji` when creating a show, but not used as the primary search criterion. When a show is found with different casing, the name is updated to the latest casing.

2. **Show–Song link (rel_show_song) creation**: Added explicit Step 4 to check and create `rel_show_song` links between shows and songs after all entities exist. No `media_url` is stored in `rel_show_song` during import.

3. **Play history records with media_url**: Step 5 creates `play_history` records that include `media_url` from AMQ's `videoUrl` field.

## Additional Improvements

- Each import step now includes inline CLI commands (search + create) to save inference effort for AI agents
- Match type per entity: `exact` for artist and song names (case-sensitive), `exact-i` for show names (case-insensitive)
- Show name casing auto-update when AMQ provides a different case than stored

## Files Modified

- `docs/import.md` — Corrected entity matching rules, added inline CLI commands per step, updated match types