#!/usr/bin/env python3
"""
Import AMQ songs into Jankenoboe database.

Two-phase approach:
  Phase 1: Resolve all entities (artist, show, song) from the database.
           Separate into "complete" (all found) vs "missing" groups.
  Phase 2: For complete groups, link show-song and create play_history.
           For missing groups, output a report for manual handling.
"""
import json
import subprocess
import sys
from urllib.parse import quote


def url_encode(text):
    """URL percent-encode text for CLI arguments."""
    return quote(text, safe="")


def run_jankenoboe(args):
    """Run jankenoboe CLI command and return parsed JSON result."""
    cmd = ["jankenoboe"] + args
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        return None
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError:
        return None


def get_artist_id(artist_name):
    """Get artist ID by exact name match."""
    encoded = url_encode(artist_name)
    term = f'{{"name": {{"value": "{encoded}",' f' "match": "exact"}}}}'
    result = run_jankenoboe(
        [
            "search",
            "artist",
            "--fields",
            "id,name",
            "--term",
            term,
        ]
    )
    if result and result.get("results"):
        return result["results"][0]["id"]
    return None


def get_show_id(show_name, vintage):
    """Get show ID by name + vintage match."""
    encoded = url_encode(show_name)
    term = (
        f'{{"name": {{"value": "{encoded}",'
        f' "match": "exact-i"}},'
        f' "vintage": {{"value": "{vintage}"}}}}'
    )
    result = run_jankenoboe(
        [
            "search",
            "show",
            "--fields",
            "id,name,vintage",
            "--term",
            term,
        ]
    )
    if result and result.get("results"):
        return result["results"][0]["id"]
    return None


def get_song_id(song_name, artist_id):
    """Get song ID by name + artist_id match."""
    encoded = url_encode(song_name)
    term = (
        f'{{"name": {{"value": "{encoded}",'
        f' "match": "exact"}},'
        f' "artist_id": {{"value": "{artist_id}"}}}}'
    )
    result = run_jankenoboe(
        [
            "search",
            "song",
            "--fields",
            "id,name,artist_id",
            "--term",
            term,
        ]
    )
    if result and result.get("results"):
        return result["results"][0]["id"]
    return None


def check_rel_show_song(show_id, song_id):
    """Check if show-song link exists."""
    term = (
        f'{{"show_id": {{"value": "{show_id}"}},'
        f' "song_id": {{"value": "{song_id}"}}}}'
    )
    result = run_jankenoboe(
        [
            "search",
            "rel_show_song",
            "--fields",
            "show_id,song_id",
            "--term",
            term,
        ]
    )
    if result and result.get("results"):
        return True
    return False


def create_rel_show_song(show_id, song_id):
    """Create show-song link."""
    data = f'{{"show_id": "{show_id}",' f' "song_id": "{song_id}"}}'
    result = run_jankenoboe(
        [
            "create",
            "rel_show_song",
            "--data",
            data,
        ]
    )
    return result is not None


def create_play_history(show_id, song_id, media_url):
    """Create play history record."""
    encoded_url = url_encode(media_url)
    data = (
        f'{{"show_id": "{show_id}",'
        f' "song_id": "{song_id}",'
        f' "media_url": "{encoded_url}"}}'
    )
    result = run_jankenoboe(
        [
            "create",
            "play_history",
            "--data",
            data,
        ]
    )
    return result is not None


def resolve_entry(song_entry):
    """Resolve artist, show, and song IDs for an AMQ export entry.

    Returns a dict with:
      - entry info (names, vintage, url)
      - resolved IDs (or None for missing)
      - missing: list of what's missing (empty if complete)
    """
    info = song_entry.get("songInfo", {})
    artist_name = info.get("artist", "")
    song_name = info.get("songName", "")
    anime_names = info.get("animeNames", {})
    show_name = anime_names.get("english", "")
    vintage = info.get("vintage", "")
    video_url = song_entry.get("videoUrl", "")

    resolved = {
        "artist_name": artist_name,
        "song_name": song_name,
        "show_name": show_name,
        "vintage": vintage,
        "video_url": video_url,
        "artist_id": None,
        "show_id": None,
        "song_id": None,
        "missing": [],
    }

    # Resolve artist
    artist_id = get_artist_id(artist_name)
    if artist_id:
        resolved["artist_id"] = artist_id
    else:
        resolved["missing"].append(f"artist: {artist_name}")

    # Resolve show
    show_id = get_show_id(show_name, vintage)
    if show_id:
        resolved["show_id"] = show_id
    else:
        resolved["missing"].append(f"show: {show_name} ({vintage})")

    # Resolve song (requires artist_id)
    if artist_id:
        song_id = get_song_id(song_name, artist_id)
        if song_id:
            resolved["song_id"] = song_id
        else:
            resolved["missing"].append(f"song: {song_name} by {artist_name}")
    else:
        resolved["missing"].append(f"song: {song_name} (artist unresolved)")

    return resolved


def print_missing_report(missing_entries):
    """Print a grouped report of entries with missing entities."""
    print("\n=== Missing Entities Report ===")
    print(f"Total entries with missing data:" f" {len(missing_entries)}\n")

    # Group by missing entity type
    missing_artists = []
    missing_shows = []
    missing_songs = []

    for entry in missing_entries:
        for item in entry["missing"]:
            if item.startswith("artist:"):
                missing_artists.append(item)
            elif item.startswith("show:"):
                missing_shows.append(item)
            elif item.startswith("song:"):
                missing_songs.append(item)

    # Deduplicate
    missing_artists = sorted(set(missing_artists))
    missing_shows = sorted(set(missing_shows))
    missing_songs = sorted(set(missing_songs))

    if missing_artists:
        print(f"Missing artists ({len(missing_artists)}):")
        for a in missing_artists:
            print(f"  - {a}")
        print()

    if missing_shows:
        print(f"Missing shows ({len(missing_shows)}):")
        for s in missing_shows:
            print(f"  - {s}")
        print()

    if missing_songs:
        print(f"Missing songs ({len(missing_songs)}):")
        for s in missing_songs:
            print(f"  - {s}")
        print()

    # Detailed per-entry breakdown
    print("--- Per-entry details ---")
    for entry in missing_entries:
        label = (
            f"\"{entry['song_name']}\""
            f" by {entry['artist_name']}"
            f" from {entry['show_name']}"
            f" ({entry['vintage']})"
        )
        print(f"  {label}")
        for m in entry["missing"]:
            print(f"    \u2717 {m}")
    print()


def process_complete_entries(complete_entries, missing_only):
    """Link and create play_history for entries where all
    entities exist.

    When missing_only is True, skip entries where the show-song
    link already exists (they were processed in a previous run).
    """
    links_created = []
    play_histories_created = 0
    skipped = 0
    errors = []

    for entry in complete_entries:
        show_id = entry["show_id"]
        song_id = entry["song_id"]
        video_url = entry["video_url"]
        label = (
            f"\"{entry['song_name']}\""
            f" by {entry['artist_name']}"
            f" from {entry['show_name']}"
        )

        already_linked = check_rel_show_song(show_id, song_id)

        # In missing-only mode, skip already-processed entries
        if missing_only and already_linked:
            skipped += 1
            print(f"  \u2013 Skipped (already linked):" f" {label}")
            continue

        # Link show to song
        if not already_linked:
            if create_rel_show_song(show_id, song_id):
                links_created.append(label)
                print(f"  \u2713 Linked: {label}")
            else:
                errors.append(f"Failed to link: {label}")
                print(f"  \u274c Failed to link: {label}")
                continue

        # Create play history
        if create_play_history(show_id, song_id, video_url):
            play_histories_created += 1
            print(f"  \u2713 Play history: {label}")
        else:
            errors.append(f"Failed play_history: {label}")
            print(f"  \u274c Failed play_history: {label}")

    return (
        links_created,
        play_histories_created,
        skipped,
        errors,
    )


def import_amq_file(filepath, missing_only=False):
    """Import AMQ JSON export file.

    Args:
        filepath: Path to the AMQ JSON export file.
        missing_only: When True, skip entries where the
            show-song link already exists (use on re-runs
            after fixing missing entities to avoid duplicate
            play_history for previously processed entries).
    """
    with open(filepath, "r", encoding="utf-8") as f:
        data = json.load(f)

    songs = data.get("songs", [])
    total = len(songs)

    if missing_only:
        print("\n[--missing-only] Will skip" " already-linked entries.\n")

    print(f"=== Phase 1: Resolving {total} songs ===\n")

    complete = []
    missing = []

    for idx, song_entry in enumerate(songs, 1):
        info = song_entry.get("songInfo", {})
        artist_name = info.get("artist", "")
        song_name = info.get("songName", "")

        print(f"[{idx}/{total}] Resolving:" f' "{song_name}" by {artist_name}')

        resolved = resolve_entry(song_entry)

        if resolved["missing"]:
            missing.append(resolved)
            print(f"  \u2717 Missing: " + ", ".join(resolved["missing"]))
        else:
            complete.append(resolved)
            print("  \u2713 All entities found")

    print(f"\n--- Resolution Summary ---")
    print(f"Complete: {len(complete)}")
    print(f"Missing:  {len(missing)}")

    # Phase 2: Process complete entries
    if complete:
        print(f"\n=== Phase 2: Processing" f" {len(complete)} complete entries ===\n")
        links, plays, skipped, errors = process_complete_entries(complete, missing_only)

        print(f"\n--- Processing Summary ---")
        print(f"New show-song links: {len(links)}")
        print(f"Play histories created: {plays}")
        if missing_only:
            print(f"Skipped (already linked): {skipped}")
        if errors:
            print(f"Errors: {len(errors)}")
            for e in errors:
                print(f"  - {e}")
    else:
        print("\nNo complete entries to process.")

    # Report missing entries
    if missing:
        print_missing_report(missing)
    else:
        print("\nAll entries resolved successfully!")


if __name__ == "__main__":
    args = [a for a in sys.argv[1:] if not a.startswith("-")]
    flags = [a for a in sys.argv[1:] if a.startswith("-")]

    if not args:
        print(
            "Usage: python3"
            " .claude/skills/importing-amq-songs"
            "/scripts/import_amq.py"
            " [--missing-only] <amq_export.json>\n"
            "\nOptions:\n"
            "  --missing-only  Skip entries already"
            " linked (use on re-runs to avoid"
            " duplicate play_history)"
        )
        sys.exit(1)

    missing_only = "--missing-only" in flags
    import_amq_file(args[0], missing_only=missing_only)
