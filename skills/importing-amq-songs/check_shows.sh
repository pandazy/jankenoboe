#!/bin/bash
# Check which shows from an AMQ export already exist in the database
# Usage: bash skills/importing-amq-songs/check_shows.sh <amq_export.json>

if [ -z "$1" ]; then
  echo "Usage: bash skills/importing-amq-songs/check_shows.sh <amq_export.json>"
  exit 1
fi

if [ ! -f "$1" ]; then
  echo "Error: File not found: $1"
  exit 1
fi

# Extract unique show name|vintage pairs from the AMQ export JSON
mapfile -t shows < <(python3 -c "
import json, sys
with open(sys.argv[1], 'r') as f:
    data = json.load(f)
seen = set()
for song in data.get('songs', []):
    info = song.get('songInfo', {})
    name = info.get('animeNames', {}).get('english', '')
    vintage = info.get('vintage', '')
    key = f'{name}|{vintage}'
    if name and key not in seen:
        seen.add(key)
        print(key)
" "$1")

echo "=== Checking Shows (${#shows[@]} unique) ==="
found=0
not_found=0
for entry in "${shows[@]}"; do
  IFS='|' read -r show_name vintage <<< "$entry"

  encoded=$(python3 tools/url_encode.py "$show_name")
  result=$(jankenoboe search show --fields id,name,vintage --term "{\"name\": {\"value\": \"$encoded\", \"match\": \"exact-i\"}, \"vintage\": {\"value\": \"$vintage\"}}" 2>&1)

  if echo "$result" | grep -q '"results":\[\]'; then
    echo "❌ NOT FOUND: $show_name ($vintage)"
    ((not_found++))
  elif echo "$result" | grep -q '"results":\['; then
    echo "✓ EXISTS: $show_name ($vintage)"
    ((found++))
  else
    echo "? ERROR checking: $show_name ($vintage)"
    echo "  Response: $result"
  fi
done

echo ""
echo "=== Summary ==="
echo "Found: $found"
echo "Not found: $not_found"
echo "Total: ${#shows[@]}"