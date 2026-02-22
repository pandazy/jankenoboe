#!/bin/bash
# Check which artists from an AMQ export already exist in the database
# Usage: bash .claude/skills/importing-amq-songs/scripts/check_artists.sh <amq_export.json>

if [ -z "$1" ]; then
  echo "Usage: bash .claude/skills/importing-amq-songs/scripts/check_artists.sh <amq_export.json>"
  exit 1
fi

if [ ! -f "$1" ]; then
  echo "Error: File not found: $1"
  exit 1
fi

# Extract unique artist names from the AMQ export JSON
mapfile -t artists < <(python3 -c "
import json, sys
with open(sys.argv[1], 'r') as f:
    data = json.load(f)
seen = set()
for song in data.get('songs', []):
    artist = song.get('songInfo', {}).get('artist', '')
    if artist and artist not in seen:
        seen.add(artist)
        print(artist)
" "$1")

echo "=== Checking Artists (${#artists[@]} unique) ==="
found=0
not_found=0
for artist in "${artists[@]}"; do
  encoded=$(python3 tools/url_encode.py "$artist")
  result=$(jankenoboe search artist --fields id,name --term "{\"name\": {\"value\": \"$encoded\", \"match\": \"exact\"}}" 2>&1)

  if echo "$result" | grep -q '"results":\[\]'; then
    echo "❌ NOT FOUND: $artist"
    ((not_found++))
  elif echo "$result" | grep -q '"results":\['; then
    echo "✓ EXISTS: $artist"
    ((found++))
  else
    echo "? ERROR checking: $artist"
    echo "  Response: $result"
  fi
done

echo ""
echo "=== Summary ==="
echo "Found: $found"
echo "Not found: $not_found"
echo "Total: ${#artists[@]}"