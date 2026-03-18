#!/usr/bin/env python3
"""
Import AMQ songs into Jankenoboe database.

Two-phase approach:
  Phase 1: Resolve all entities (artist, show, song) from the database.
           Separate into "complete" (all found) vs "missing" groups.
  Phase 2: For complete groups, link show-song and create play_history.
           For missing groups, output a report with actionable CLI
           commands for manual handling.
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


def search_artists_by_name(artist_name):
    """Search artists by exact name match. Returns all matches."""
    encoded = url_encode(artist_name)
    term = f'{{"name": {{"value": "{encoded}", "match": "exact"}}}}'
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
        return result["results"]
    return []


def get_songs_for_artist(artist_id):
    """Get all songs for an artist by ID."""
    term = f'{{"artist_id": {{"value": "{artist_id}"}}}}'
    result = run_jankenoboe(
        [
            "search",
            "song",
            "--fields",
            "id,name",
            "--term",
            term,
        ]
    )
    if result and result.get("results"):
        return result["results"]
    return []


def create_artist(artist_name):
    """Create a new artist and return its ID."""
    encoded = url_encode(artist_name)
    data = f'{{"name": "{encoded}"}}'
    result = run_jankenoboe(
        [
            "create",
            "artist",
            "--data",
            data,
        ]
    )
    if result and result.get("id"):
        return result["id"]
    return None


def prompt_artist_disambiguation(artist_name, artists, song_name, show_name):
    """Prompt user to select from multiple artists with the same name.

    Displays each artist's existing songs and lets the user choose
    the correct one, or create a new artist.

    Returns the selected artist ID, or None if user chooses to skip.
    """
    print(f'\n  ⚠ Multiple artists named "{artist_name}" found!')
    print(f'    (resolving: "{song_name}" from "{show_name}")')

    for idx, artist in enumerate(artists, 1):
        aid = artist["id"]
        songs = get_songs_for_artist(aid)
        song_names = [s["name"] for s in songs]
        print(f"\n    [{idx}] Artist ID: {aid}")
        if song_names:
            print("        Songs: " + ", ".join(song_names))
        else:
            print("        Songs: (none)")

    new_idx = len(artists) + 1
    skip_idx = new_idx + 1
    print(f'\n    [{new_idx}] Create a NEW artist named "{artist_name}"')
    print(f"    [{skip_idx}] Skip this entry")

    while True:
        try:
            choice = input(f"\n    Select [1-{skip_idx}]: ").strip()
            choice_num = int(choice)
        except (ValueError, EOFError):
            print("    Invalid input. Try again.")
            continue

        if 1 <= choice_num <= len(artists):
            selected = artists[choice_num - 1]
            print(f"    → Selected artist: {selected['id']}")
            return selected["id"]
        elif choice_num == new_idx:
            new_id = create_artist(artist_name)
            if new_id:
                print(f"    → Created new artist: {new_id}")
                return new_id
            else:
                print("    ✗ Failed to create artist. Try again.")
                continue
        elif choice_num == skip_idx:
            print("    → Skipping this entry.")
            return None
        else:
            print("    Invalid choice. Try again.")


def get_artist_id(artist_name, song_name="", show_name=""):
    """Get artist ID by exact name match.

    When multiple artists share the same name, prompts the
    user to disambiguate by showing each artist's song list.
    """
    artists = search_artists_by_name(artist_name)
    if not artists:
        return None
    if len(artists) == 1:
        return artists[0]["id"]

    # Multiple matches — prompt user to disambiguate
    return prompt_artist_disambiguation(artist_name, artists, song_name, show_name)


def get_show(show_name, vintage):
    """Get show record by name + vintage match.

    Returns the full show record dict (with id, name, vintage,
    name_romaji) or None if not found.
    """
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
            "id,name,vintage,name_romaji",
            "--term",
            term,
        ]
    )
    if result and result.get("results"):
        return result["results"][0]
    return None


def update_show_romaji(show_id, romaji_name):
    """Update a show's name_romaji field."""
    encoded = url_encode(romaji_name)
    data = f'{{"name_romaji": "{encoded}"}}'
    result = run_jankenoboe(
        [
            "update",
            "show",
            show_id,
            "--data",
            data,
        ]
    )
    return result is not None


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
        f'{{"show_id": {{"value": "{show_id}"}}, "song_id": {{"value": "{song_id}"}}}}'
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
    data = f'{{"show_id": "{show_id}", "song_id": "{song_id}"}}'
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
      - entry info (names, vintage, url, romaji, s_type)
      - resolved IDs (or None for missing)
      - missing: list of what's missing (empty if complete)
    """
    info = song_entry.get("songInfo", {})
    artist_name = info.get("artist", "")
    song_name = info.get("songName", "")
    anime_names = info.get("animeNames", {})
    show_name = anime_names.get("english", "")
    romaji_name = anime_names.get("romaji", "")
    vintage = info.get("vintage", "")
    s_type = info.get("animeType", "")
    video_url = song_entry.get("videoUrl", "")

    resolved = {
        "artist_name": artist_name,
        "song_name": song_name,
        "show_name": show_name,
        "romaji_name": romaji_name,
        "vintage": vintage,
        "s_type": s_type,
        "video_url": video_url,
        "artist_id": None,
        "show_id": None,
        "song_id": None,
        "missing": [],
    }

    # Resolve artist
    artist_id = get_artist_id(artist_name, song_name, show_name)
    if artist_id:
        resolved["artist_id"] = artist_id
    else:
        resolved["missing"].append(f"artist: {artist_name}")

    # Resolve show
    show_record = get_show(show_name, vintage)
    if show_record:
        resolved["show_id"] = show_record["id"]
        # Fill missing romaji name if the import has one
        existing_romaji = show_record.get("name_romaji") or ""
        if not existing_romaji and romaji_name:
            if update_show_romaji(show_record["id"], romaji_name):
                resolved["romaji_updated"] = True
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


def _missing_pattern(entry):
    """Return a tuple of missing entity types for an entry."""
    missing = []
    if entry["artist_id"] is None:
        missing.append("artist")
    if entry["show_id"] is None:
        missing.append("show")
    if entry["song_id"] is None:
        missing.append("song")
    return tuple(missing)


def _resolved_label(pattern):
    """Human-readable label for what's resolved."""
    all_entities = {"artist", "show", "song"}
    resolved = sorted(all_entities - set(pattern))
    if not resolved:
        return ""
    return " (" + " and ".join(resolved) + " resolved)"


def _build_create_cmd(table, data_dict):
    """Build a jankenoboe create command string."""
    encoded_parts = []
    for key, val in data_dict.items():
        if val:
            encoded_val = url_encode(val)
            encoded_parts.append(f'"{key}": "{encoded_val}"')
    data_json = "{" + ", ".join(encoded_parts) + "}"
    return f"jankenoboe create {table} --data '{data_json}'"


def print_missing_report(missing_entries):
    """Print a grouped report of entries with missing entities.

    Groups entries by their missing pattern (which combination of
    artist/show/song is missing). Each group shows resolved IDs and
    actionable CLI commands for creating missing entities.
    """
    print("\n=== Missing Entities Report ===")
    print(f"Total entries with missing data: {len(missing_entries)}\n")

    # Collect unique missing entities (deduplicated)
    missing_artists = {}
    missing_shows = {}
    missing_songs = {}

    for entry in missing_entries:
        if entry["artist_id"] is None:
            name = entry["artist_name"]
            if name not in missing_artists:
                missing_artists[name] = entry
        if entry["show_id"] is None:
            key = f"{entry['show_name']}|||{entry['vintage']}"
            if key not in missing_shows:
                missing_shows[key] = entry
        if entry["song_id"] is None:
            key = f"{entry['song_name']}|||{entry['artist_name']}"
            if key not in missing_songs:
                missing_songs[key] = entry

    if missing_artists:
        print(f"Missing artists ({len(missing_artists)}):")
        for name, entry in sorted(missing_artists.items()):
            print(f"  - artist: {name}")
            cmd = _build_create_cmd("artist", {"name": name})
            print(f"    → {cmd}")
        print()

    if missing_shows:
        print(f"Missing shows ({len(missing_shows)}):")
        for key, entry in sorted(missing_shows.items()):
            show_name = entry["show_name"]
            vintage = entry["vintage"]
            romaji = entry["romaji_name"]
            s_type = entry["s_type"]
            print(f"  - show: {show_name} ({vintage})")
            if romaji:
                print(f"    romaji: {romaji}")
            show_data = {
                "name": show_name,
                "name_romaji": romaji,
                "vintage": vintage,
                "s_type": s_type,
            }
            cmd = _build_create_cmd("show", show_data)
            print(f"    → {cmd}")
        print()

    if missing_songs:
        print(f"Missing songs ({len(missing_songs)}):")
        for key, entry in sorted(missing_songs.items()):
            song_name = entry["song_name"]
            artist_name = entry["artist_name"]
            artist_id = entry["artist_id"]
            if artist_id:
                print(f"  - song: {song_name} by {artist_name}")
                cmd = _build_create_cmd(
                    "song",
                    {
                        "name": song_name,
                        "artist_id": artist_id,
                    },
                )
                print(f"    → {cmd}")
            else:
                print(f"  - song: {song_name} by {artist_name} (create artist first)")
        print()

    # Group entries by missing pattern
    from collections import OrderedDict

    groups = OrderedDict()
    for entry in missing_entries:
        pattern = _missing_pattern(entry)
        groups.setdefault(pattern, []).append(entry)

    # Print each group with resolved IDs
    for pattern, entries in groups.items():
        label = "missing " + ", ".join(pattern)
        label += _resolved_label(pattern)
        print(f"--- {label} ---")
        for entry in entries:
            desc = (
                f'"{entry["song_name"]}"'
                f" by {entry['artist_name']}"
                f" from {entry['show_name']}"
                f" ({entry['vintage']})"
            )
            print(f"  {desc}")

            # Print resolved IDs
            if entry["artist_id"]:
                print(f"    \u2713 artist_id: {entry['artist_id']}")
            if entry["show_id"]:
                print(f"    \u2713 show_id: {entry['show_id']}")
            if entry["song_id"]:
                print(f"    \u2713 song_id: {entry['song_id']}")

            # Print what's missing
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
            f'"{entry["song_name"]}"'
            f" by {entry['artist_name']}"
            f" from {entry['show_name']}"
        )

        already_linked = check_rel_show_song(show_id, song_id)

        # In missing-only mode, skip already-processed entries
        if missing_only and already_linked:
            skipped += 1
            print(f"  \u2013 Skipped (already linked): {label}")
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
        print("\n[--missing-only] Will skip already-linked entries.\n")

    print(f"=== Phase 1: Resolving {total} songs ===\n")

    complete = []
    missing = []

    for idx, song_entry in enumerate(songs, 1):
        info = song_entry.get("songInfo", {})
        artist_name = info.get("artist", "")
        song_name = info.get("songName", "")

        print(f'[{idx}/{total}] Resolving: "{song_name}" by {artist_name}')

        resolved = resolve_entry(song_entry)

        if resolved["missing"]:
            missing.append(resolved)
            print("  \u2717 Missing: " + ", ".join(resolved["missing"]))
        else:
            complete.append(resolved)
            romaji_note = ""
            if resolved.get("romaji_updated"):
                romaji_note = " (filled romaji name)"
            print(f"  \u2713 All entities found{romaji_note}")

    print("\n--- Resolution Summary ---")
    print(f"Complete: {len(complete)}")
    print(f"Missing:  {len(missing)}")

    # Phase 2: Process complete entries
    if complete:
        print(f"\n=== Phase 2: Processing {len(complete)} complete entries ===\n")
        links, plays, skipped, errors = process_complete_entries(complete, missing_only)

        print("\n--- Processing Summary ---")
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
