#!/usr/bin/env python3
"""
Parse AMQ export JSON and extract unique artists, shows, and songs
for import.
"""
import json
import sys
from collections import OrderedDict


def parse_amq_export(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        data = json.load(f)

    songs = data.get('songs', [])

    # Track unique entities
    artists = OrderedDict()
    shows = OrderedDict()
    song_data = []

    for idx, song_entry in enumerate(songs, 1):
        info = song_entry.get('songInfo', {})

        artist_name = info.get('artist', '')
        song_name = info.get('songName', '')

        anime_names = info.get('animeNames', {})
        english_name = anime_names.get('english', '')
        romaji_name = anime_names.get('romaji', '')

        vintage = info.get('vintage', '')
        anime_type = info.get('animeType', '')
        video_url = info.get('videoUrl', '')

        # Track artist
        if artist_name and artist_name not in artists:
            artists[artist_name] = True

        # Track show (unique by english name + vintage)
        show_key = f"{english_name}|||{vintage}"
        if show_key not in shows:
            shows[show_key] = {
                'english': english_name,
                'romaji': romaji_name,
                'vintage': vintage,
                'type': anime_type
            }

        # Track song
        song_data.append({
            'number': idx,
            'song_name': song_name,
            'artist': artist_name,
            'show_english': english_name,
            'show_romaji': romaji_name,
            'vintage': vintage,
            'type': anime_type,
            'video_url': video_url
        })

    return {
        'artists': list(artists.keys()),
        'shows': list(shows.values()),
        'songs': song_data
    }


if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: python3 parse_amq_import.py <amq_export.json>")
        sys.exit(1)

    result = parse_amq_export(sys.argv[1])

    print("\n=== SUMMARY ===")
    print(f"Total songs: {len(result['songs'])}")
    print(f"Unique artists: {len(result['artists'])}")
    print(f"Unique shows: {len(result['shows'])}")

    print(f"\n=== ARTISTS ({len(result['artists'])}) ===")
    for artist in result['artists']:
        print(f"  - {artist}")

    print(f"\n=== SHOWS ({len(result['shows'])}) ===")
    for show in result['shows']:
        print(f"  - {show['english']} ({show['vintage']})")

    print(f"\n=== SONGS ({len(result['songs'])}) ===")
    for song in result['songs']:
        print(
            f"  {song['number']}. \"{song['song_name']}\""
            f" by {song['artist']}"
        )
        print(
            f"     from {song['show_english']}"
            f" ({song['vintage']})"
        )
