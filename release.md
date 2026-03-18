## v2.7.0

### Improved: Interactive namesake artist disambiguation during import

The AMQ import script now interactively prompts when multiple artists share the same name. Instead of silently picking the first match, the script pauses and displays each artist's existing songs, letting the user select the correct one, create a new namesake artist, or skip the entry. This prevents misattributed songs during import.

### New: `display_level` field in learning responses

`learning-due` and `learning-by-song-ids` now return a `display_level` field (`level + 1`, 1-indexed) alongside the stored `level` (0-indexed). This makes the stored vs. displayed level convention explicit — no more manual `+1` arithmetic in agents or scripts.

### Improved: Due song review report

The HTML review report (`learning-song-review`) has been redesigned:

- **Shows sourced from play_history** — show data and media URLs now come directly from `play_history` instead of `rel_show_song`, so the report reflects actual quiz encounters
- **Grouped media URLs per show** — each show is displayed as a distinct group with its own media links, making it easy to identify which URLs belong to which show
- **Vintage display** — each show now displays its vintage (e.g., "Winter 2024") next to the name

### Improved: Import skill simplified

- Streamlined AMQ import workflow to a single script (`import_amq.py`) — removed the separate analyze/verify step and unused helper scripts
- Missing entities report now generates ready-to-run `jankenoboe create` CLI commands with all available fields (`name_romaji`, `vintage`, `s_type`)
- Replaced `tools/url_encode.py` with inline Python one-liners across all docs and skills to avoid REPL-mode hangs

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/install.sh | sh
```

## Platform Support

Pre-built binaries for:
- Linux x86_64
- Linux ARM64 (aarch64)
- macOS Apple Silicon (aarch64)
