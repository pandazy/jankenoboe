# Import Missing Report: Grouped by Pattern with Resolved IDs

**Date:** 2026-03-09

## Goal

Enhance the import script's Missing Entities Report to group entries by their missing pattern and include already-resolved IDs, so follow-up procedures can reuse them without re-fetching.

## Changes

### `import_amq.py`
- Added `_missing_pattern(entry)` helper: returns a tuple of missing entity types (e.g., `("show", "song")`)
- Added `_resolved_label(pattern)` helper: generates human-readable label for what's resolved (e.g., `(artist resolved)`)
- Rewrote `print_missing_report()`:
  - Still prints deduplicated missing artists/shows/songs summary at the top
  - Now groups entries by missing pattern using `OrderedDict`
  - Each group has a header like `--- missing show, song (artist resolved) ---`
  - Each entry within a group shows resolved IDs with ✓ markers (`artist_id`, `show_id`, `song_id`)
  - Missing items shown with ✗ markers

### `test_import_amq.py`
- Added 6 new test assertions:
  - Group headers: `missing artist, show, song`, `missing show, song (artist resolved)`, `missing song (artist and show resolved)`
  - Resolved IDs: `artist_id` for eufonius entry, `artist_id` and `show_id` for Hitohira entry

### `SKILL.md`
- Documented the grouped report format with the four possible missing patterns

## Example Output

```
--- missing artist, show, song ---
  "snowspring" by ChoQMay from A Sign of Affection (Winter 2024)
    ✗ artist: ChoQMay
    ✗ show: A Sign of Affection (Winter 2024)
    ✗ song: snowspring (artist unresolved)

--- missing show, song (artist resolved) ---
  "Koi Suru Kokoro" by eufonius from Kashimashi: Girl Meets Girl (Winter 2006)
    ✓ artist_id: abc123
    ✗ show: Kashimashi: Girl Meets Girl (Winter 2006)
    ✗ song: Koi Suru Kokoro by eufonius

--- missing song (artist and show resolved) ---
  "Hitohira" by Hitomi Miyahara from The Fragrant Flower Blooms With Dignity (Summer 2025)
    ✓ artist_id: def456
    ✓ show_id: ghi789
    ✗ song: Hitohira by Hitomi Miyahara
```

## Test Results

All 29/29 tests passed.
