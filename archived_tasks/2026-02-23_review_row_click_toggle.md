# Task: Review Report Row Click Toggle

**Date:** 2026-02-23  
**Status:** Completed

## Objective

Add a click-to-toggle color feature on each song card in the `learning-song-review` HTML template, allowing users to visually mark songs they have reviewed.

## Changes Made

### `templates/learning-song-review.html`

**CSS additions:**
- Added `cursor: pointer`, `transition` (background + border-left-color, 0.2s), and `user-select: none` to `.song-card`
- Added `.song-card.reviewed` class with contrast green color scheme:
  - Background: `#1a3a2a` (dark green, vs original `#16213e` dark blue)
  - Border-left: `#4caf50` (green, vs original `#e94560` red)
  - Song name text: `#a0d8a0`
  - Meta text: `#7ab87a`
  - Shows text: `#6dbd8a`

**JavaScript additions:**
- `reviewedSet` object tracks which song indices (by global array index) are marked as reviewed
- Each `.song-card` element gets a `data-idx` attribute for tracking
- After rendering, click listeners are attached to each card to toggle the `.reviewed` CSS class
- Reviewed state persists across page navigation within the same browser session (stored in `reviewedSet`)

## How It Works

1. User opens the generated HTML review report in a browser
2. Clicking any song card toggles it to a green "reviewed" color scheme
3. Clicking again toggles it back to the default dark blue/red scheme
4. Marks persist when navigating between pages (but not across browser sessions, as state is in-memory only)

## Files Modified

| File | Change |
|------|--------|
| `templates/learning-song-review.html` | Added reviewed toggle CSS + JS |