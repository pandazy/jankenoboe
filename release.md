# v2.1.0 — Race-condition-safe batch level up

## New Features

- **`learning-song-levelup-ids`** — Level up specific learning records by their IDs. Accepts comma-separated learning UUIDs via `--ids`. This is the race-condition-safe way to level up songs after reviewing a report.
- **`learning-song-review` now returns `learning_ids`** — The JSON output includes an array of learning record UUIDs, enabling the review→levelup workflow without race conditions.

## Breaking Changes

- **Removed `learning-song-levelup-due`** — This command re-queried for due songs at execution time, which could level up unreviewed songs if new songs became due between report generation and level-up. Use `learning-song-levelup-ids` with IDs from `learning-song-review` instead.

## Recommended Workflow

```bash
# Step 1: Generate review report (captures learning_ids)
out=$(jankenoboe learning-song-review --output ~/review.html)

# Step 2: User reviews the HTML report in browser

# Step 3: Level up exactly those songs (no race condition)
ids=$(echo "$out" | jq -r '.learning_ids | join(",")')
jankenoboe learning-song-levelup-ids --ids "$ids"
```

## Installation

Download the appropriate binary for your platform from the release assets and install using:

```bash
curl -fsSL https://raw.githubusercontent.com/pandazy/jankenoboe/main/install.sh | sh
```

## Platform Support

Pre-built binaries for:
- Linux x86_64
- Linux ARM64 (aarch64)
- macOS Apple Silicon (aarch64)

Note: Intel Mac users can run the Apple Silicon binary through Rosetta 2.