#!/usr/bin/env python3
"""
Import AMQ songs into Jankenoboe database.

Processes songs sequentially: resolve artist/show/song, link them,
create play history.
"""
import json
import subprocess
import sys
from urllib.parse import quote


def url_encode(text):
    """URL percent-encode text for CLI arguments."""
    return quote(text, safe='')


def run_jankenoboe(args):
    """Run jankenoboe CLI command and return parsed JSON result."""
    cmd = ['jankenoboe'] + args
    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode != 0:
        print(f"ERROR running: {' '.join(cmd)}")
        print(f"stderr: {result.stderr}")
        return None
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError:
        print(f"ERROR parsing JSON from: {' '.join(cmd)}")
        print(f"stdout: {result.stdout}")
        return None


def get_artist_id(artist_name):
    """Get artist ID by exact name match."""
    encoded = url_encode(artist_name)
    term = (
        f'{{"name": {{"value": "{encoded}",'
        f' "match": "exact"}}}}'
    )
    result = run_jankenoboe([
        'search', 'artist',
        '--fields', 'id,name',
        '--term', term,
    ])
    if result and result.get('results'):
        return result['results'][0]['id']
    return None


def get_show_id(show_name, vintage):
    """Get show ID by name + vintage match."""
    encoded = url_encode(show_name)
    term = (
        f'{{"name": {{"value": "{encoded}",'
        f' "match": "exact-i"}},'
        f' "vintage": {{"value": "{vintage}"}}}}'
    )
    result = run_jankenoboe([
        'search', 'show',
        '--fields', 'id,name,vintage',
        '--term', term,
    ])
    if result and result.get('results'):
        return result['results'][0]['id']
    return None


def get_song_id(song_name, artist_id):
    """Get song ID by name + artist_id match."""
    encoded = url_encode(song_name)
    term = (
        f'{{"name": {{"value": "{encoded}",'
        f' "match": "exact"}},'
        f' "artist_id": {{"value": "{artist_id}"}}}}'
    )
    result = run_jankenoboe([
        'search', 'song',
        '--fields', 'id,name,artist_id',
        '--term', term,
    ])
    if result and result.get('results'):
        return result['results'][0]['id']
    return None


def check_rel_show_song(show_id, song_id):
    """Check if show-song link exists."""
    term = (
        f'{{"show_id": {{"value": "{show_id}"}},'
        f' "song_id": {{"value": "{song_id}"}}}}'
    )
    result = run_jankenoboe([
        'search', 'rel_show_song',
        '--fields', 'show_id,song_id',
        '--term', term,
    ])
    if result and result.get('results'):
        return True
    return False


def create_song(song_name, artist_id):
    """Create a new song."""
    encoded_name = url_encode(song_name)
    data = (
        f'{{"name": "{encoded_name}",'
        f' "artist_id": "{artist_id}"}}'
    )
    result = run_jankenoboe([
        'create', 'song',
        '--data', data,
    ])
    if result and result.get('id'):
        return result['id']
    return None


def create_rel_show_song(show_id, song_id):
    """Create show-song link."""
    data = (
        f'{{"show_id": "{show_id}",'
        f' "song_id": "{song_id}"}}'
    )
    result = run_jankenoboe([
        'create', 'rel_show_song',
        '--data', data,
    ])
    return result is not None


def create_play_history(show_id, song_id, media_url):
    """Create play history record."""
    encoded_url = url_encode(media_url)
    data = (
        f'{{"show_id": "{show_id}",'
        f' "song_id": "{song_id}",'
        f' "media_url": "{encoded_url}"}}'
    )
    result = run_jankenoboe([
        'create', 'play_history',
        '--data', data,
    ])
    return result is not None


def import_amq_file(filepath):
    """Import AMQ JSON export file."""
    with open(filepath, 'r', encoding='utf-8') as f:
        data = json.load(f)

    songs = data.get('songs', [])
    total = len(songs)

    print(f"\n=== Starting Import: {total} songs ===\n")

    songs_created = []
    links_created = []

    for idx, song_entry in enumerate(songs, 1):
        info = song_entry.get('songInfo', {})

        artist_name = info.get('artist', '')
        song_name = info.get('songName', '')
        anime_names = info.get('animeNames', {})
        show_name = anime_names.get('english', '')
        vintage = info.get('vintage', '')
        video_url = song_entry.get('videoUrl', '')

        print(
            f"[{idx}/{total}] Processing:"
            f" \"{song_name}\" by {artist_name}"
        )
        print(f"         from {show_name} ({vintage})")

        # Step 1: Get artist ID
        artist_id = get_artist_id(artist_name)
        if not artist_id:
            print(
                "  \u274c ERROR: Artist not found:"
                f" {artist_name}"
            )
            continue
        print(f"  \u2713 Artist ID: {artist_id}")

        # Step 2: Get show ID
        show_id = get_show_id(show_name, vintage)
        if not show_id:
            print(
                "  \u274c ERROR: Show not found:"
                f" {show_name} ({vintage})"
            )
            continue
        print(f"  \u2713 Show ID: {show_id}")

        # Step 3: Get or create song
        song_id = get_song_id(song_name, artist_id)
        if song_id:
            print(f"  \u2713 Song exists: {song_id}")
        else:
            print(f"  \u2192 Creating song: {song_name}")
            song_id = create_song(song_name, artist_id)
            if song_id:
                print(f"  \u2713 Song created: {song_id}")
                songs_created.append(song_name)
            else:
                print("  \u274c ERROR: Failed to create song")
                continue

        # Step 4: Link show to song
        if check_rel_show_song(show_id, song_id):
            print("  \u2713 Show-song link exists")
        else:
            print("  \u2192 Creating show-song link")
            if create_rel_show_song(show_id, song_id):
                print("  \u2713 Show-song link created")
                links_created.append(
                    f"{show_name} \u2192 {song_name}"
                )
            else:
                print(
                    "  \u274c ERROR:"
                    " Failed to create show-song link"
                )
                continue

        # Step 5: Create play history
        print("  \u2192 Creating play history")
        if create_play_history(show_id, song_id, video_url):
            print("  \u2713 Play history created")
        else:
            print(
                "  \u274c ERROR:"
                " Failed to create play history"
            )

        print()

    # Summary
    print("\n=== Import Complete ===")
    print(f"Total songs processed: {total}")
    print(f"New songs created: {len(songs_created)}")
    if songs_created:
        for song in songs_created:
            print(f"  - {song}")
    print(f"New show-song links: {len(links_created)}")
    if links_created:
        for link in links_created:
            print(f"  - {link}")


if __name__ == '__main__':
    if len(sys.argv) < 2:
        print(
            "Usage: python3"
            " .claude/skills/importing-amq-songs/scripts/import_amq.py"
            " <amq_export.json>"
        )
        sys.exit(1)

    import_amq_file(sys.argv[1])
