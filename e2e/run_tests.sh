#!/bin/sh
# Jankenoboe E2E Test Suite
# Runs against the installed jankenoboe binary with a real SQLite database.
# Each test resets the database to ensure isolation.
#
# Exit codes: 0 = all tests passed, 1 = one or more tests failed

# NOTE: Do not use set -e — many tests deliberately run commands that fail.

PASS=0
FAIL=0
TOTAL=0
DB_PATH="${JANKENOBOE_DB}"
INIT_SQL="${JANKENOBOE_INIT_SQL:-/app/init-db.sql}"
BINARY_PATH="${JANKENOBOE_BINARY_PATH:-/usr/local/bin/jankenoboe}"

# Colors (if terminal supports them)
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# --- Helpers ---

reset_db() {
  rm -f "$DB_PATH"
  sqlite3 "$DB_PATH" < "$INIT_SQL"
}

assert_exit_code() {
  local test_name="$1"
  local expected="$2"
  local actual="$3"
  TOTAL=$((TOTAL + 1))
  if [ "$actual" -eq "$expected" ]; then
    PASS=$((PASS + 1))
    printf "${GREEN}  ✓ %s${NC}\n" "$test_name"
  else
    FAIL=$((FAIL + 1))
    printf "${RED}  ✗ %s (expected exit %d, got %d)${NC}\n" "$test_name" "$expected" "$actual"
  fi
}

assert_json_field() {
  local test_name="$1"
  local json="$2"
  local jq_expr="$3"
  local expected="$4"
  TOTAL=$((TOTAL + 1))
  local actual
  actual=$(echo "$json" | jq -r "$jq_expr" 2>/dev/null || echo "__JQ_ERROR__")
  if [ "$actual" = "$expected" ]; then
    PASS=$((PASS + 1))
    printf "${GREEN}  ✓ %s${NC}\n" "$test_name"
  else
    FAIL=$((FAIL + 1))
    printf "${RED}  ✗ %s (expected '%s', got '%s')${NC}\n" "$test_name" "$expected" "$actual"
  fi
}

assert_output_contains() {
  local test_name="$1"
  local output="$2"
  local substring="$3"
  TOTAL=$((TOTAL + 1))
  if echo "$output" | grep -q "$substring"; then
    PASS=$((PASS + 1))
    printf "${GREEN}  ✓ %s${NC}\n" "$test_name"
  else
    FAIL=$((FAIL + 1))
    printf "${RED}  ✗ %s (output does not contain '%s')${NC}\n" "$test_name" "$substring"
  fi
}

# --- Test Suites ---

echo ""
printf "${YELLOW}=== Jankenoboe E2E Tests ===${NC}\n"
echo ""

# ---- 1. Binary basics ----
printf "${YELLOW}--- Binary Basics ---${NC}\n"

# --version
out=$(jankenoboe --version 2>&1)
ec=$?
assert_exit_code "--version exits 0" 0 "$ec"
assert_output_contains "--version contains 'jankenoboe'" "$out" "jankenoboe"

# No args shows usage on stderr
jankenoboe 2>/tmp/e2e_noargs_stderr 1>/dev/null; true
stderr=$(cat /tmp/e2e_noargs_stderr)
assert_output_contains "no args shows Usage" "$stderr" "Usage"

# Missing JANKENOBOE_DB
saved_db="$DB_PATH"
unset JANKENOBOE_DB
out=$(jankenoboe get artist some-id --fields id 2>&1 || true)
export JANKENOBOE_DB="$saved_db"
assert_output_contains "missing DB env shows JANKENOBOE_DB error" "$out" "JANKENOBOE_DB"

echo ""

# ---- 2. Invalid table errors ----
printf "${YELLOW}--- Invalid Table Errors ---${NC}\n"
reset_db

jankenoboe get bad_table some-id --fields id 2>/tmp/e2e_stderr 1>/dev/null; ec=$?
stderr=$(cat /tmp/e2e_stderr)
assert_output_contains "get invalid table" "$stderr" "Invalid table"

jankenoboe search bad_table --term '{"name":{"value":"x"}}' --fields id 2>/tmp/e2e_stderr 1>/dev/null; ec=$?
stderr=$(cat /tmp/e2e_stderr)
assert_output_contains "search invalid table" "$stderr" "Invalid table"

jankenoboe duplicates bad_table 2>/tmp/e2e_stderr 1>/dev/null; ec=$?
stderr=$(cat /tmp/e2e_stderr)
assert_output_contains "duplicates invalid table" "$stderr" "Invalid table"

jankenoboe create bad_table --data '{"name":"x"}' 2>/tmp/e2e_stderr 1>/dev/null; ec=$?
assert_exit_code "create invalid table exits non-zero" 1 "$ec"

echo ""

# ---- 3. CRUD lifecycle ----
printf "${YELLOW}--- CRUD Lifecycle ---${NC}\n"
reset_db

# Create artist
out=$(jankenoboe create artist --data '{"name":"TestArtist"}')
ec=$?
assert_exit_code "create artist exits 0" 0 "$ec"
ARTIST_ID=$(echo "$out" | jq -r '.id')
assert_json_field "create artist returns id" "$out" '.id' "$ARTIST_ID"

# Get artist
out=$(jankenoboe get artist "$ARTIST_ID" --fields id,name)
ec=$?
assert_exit_code "get artist exits 0" 0 "$ec"
assert_json_field "get artist name" "$out" '.results[0].name' "TestArtist"

# Update artist
out=$(jankenoboe update artist "$ARTIST_ID" --data '{"name":"UpdatedArtist"}')
ec=$?
assert_exit_code "update artist exits 0" 0 "$ec"
assert_output_contains "update response contains 'updated'" "$out" "updated"

# Verify update
out=$(jankenoboe get artist "$ARTIST_ID" --fields id,name)
assert_json_field "get artist after update" "$out" '.results[0].name' "UpdatedArtist"

# Delete artist
out=$(jankenoboe delete artist "$ARTIST_ID")
ec=$?
assert_exit_code "delete artist exits 0" 0 "$ec"
assert_output_contains "delete response contains 'deleted'" "$out" "deleted"

echo ""

# ---- 4. Search ----
printf "${YELLOW}--- Search ---${NC}\n"
reset_db

jankenoboe create artist --data '{"name":"FindMe"}' > /dev/null
jankenoboe create artist --data '{"name":"NotMe"}' > /dev/null

out=$(jankenoboe search artist --term '{"name":{"value":"FindMe"}}' --fields id,name)
ec=$?
assert_exit_code "search exits 0" 0 "$ec"
assert_json_field "search finds 1 result" "$out" '.results | length' "1"
assert_json_field "search result name" "$out" '.results[0].name' "FindMe"

echo ""

# ---- 5. Duplicates ----
printf "${YELLOW}--- Duplicates ---${NC}\n"
reset_db

jankenoboe create artist --data '{"name":"DupArtist"}' > /dev/null
jankenoboe create artist --data '{"name":"DupArtist"}' > /dev/null
jankenoboe create artist --data '{"name":"UniqueArtist"}' > /dev/null

out=$(jankenoboe duplicates artist)
ec=$?
assert_exit_code "duplicates exits 0" 0 "$ec"
assert_json_field "duplicates finds 1 group" "$out" '.duplicates | length' "1"

echo ""

# ---- 6. Song + Artist relationship ----
printf "${YELLOW}--- Song-Artist Relationship ---${NC}\n"
reset_db

a_out=$(jankenoboe create artist --data '{"name":"SongArtist"}')
A_ID=$(echo "$a_out" | jq -r '.id')

s_out=$(jankenoboe create song --data "{\"name\":\"MySong\",\"artist_id\":\"$A_ID\"}")
S_ID=$(echo "$s_out" | jq -r '.id')
assert_json_field "create song returns id" "$s_out" '.id' "$S_ID"

out=$(jankenoboe get song "$S_ID" --fields id,name,artist_id)
assert_json_field "song artist_id matches" "$out" '.results[0].artist_id' "$A_ID"

echo ""

# ---- 7. Bulk reassign ----
printf "${YELLOW}--- Bulk Reassign ---${NC}\n"
reset_db

a1_out=$(jankenoboe create artist --data '{"name":"OldArtist"}')
A1_ID=$(echo "$a1_out" | jq -r '.id')

a2_out=$(jankenoboe create artist --data '{"name":"NewArtist"}')
A2_ID=$(echo "$a2_out" | jq -r '.id')

s_out=$(jankenoboe create song --data "{\"name\":\"ReassignSong\",\"artist_id\":\"$A1_ID\"}")
S_ID=$(echo "$s_out" | jq -r '.id')

out=$(jankenoboe bulk-reassign --from-artist-id "$A1_ID" --to-artist-id "$A2_ID")
ec=$?
assert_exit_code "bulk-reassign exits 0" 0 "$ec"
assert_json_field "bulk-reassign count" "$out" '.reassigned_count' "1"

# Verify song now belongs to new artist
out=$(jankenoboe get song "$S_ID" --fields artist_id)
assert_json_field "song reassigned to new artist" "$out" '.results[0].artist_id' "$A2_ID"

echo ""

# ---- 8. Learning ----
printf "${YELLOW}--- Learning ---${NC}\n"
reset_db

# learning-due on empty DB
out=$(jankenoboe learning-due --limit 10)
ec=$?
assert_exit_code "learning-due exits 0" 0 "$ec"
assert_json_field "learning-due empty count" "$out" '.count' "0"

# Create artist + song, then add to learning
a_out=$(jankenoboe create artist --data '{"name":"LearnArtist"}')
A_ID=$(echo "$a_out" | jq -r '.id')

s_out=$(jankenoboe create song --data "{\"name\":\"LearnSong\",\"artist_id\":\"$A_ID\"}")
S_ID=$(echo "$s_out" | jq -r '.id')

out=$(jankenoboe learning-batch --song-ids "$S_ID")
ec=$?
assert_exit_code "learning-batch exits 0" 0 "$ec"
assert_json_field "learning-batch created 1 record" "$out" '.created_ids | length' "1"

echo ""

# ---- 9. Show CRUD ----
printf "${YELLOW}--- Show CRUD ---${NC}\n"
reset_db

# Create show
out=$(jankenoboe create show --data '{"name":"A Sign of Affection","name_romaji":"Yubisaki to Renren","vintage":"Winter 2024","s_type":"TV"}')
ec=$?
assert_exit_code "create show exits 0" 0 "$ec"
SHOW_ID=$(echo "$out" | jq -r '.id')
assert_json_field "create show returns id" "$out" '.id' "$SHOW_ID"

# Get show
out=$(jankenoboe get show "$SHOW_ID" --fields id,name,name_romaji,vintage,s_type)
ec=$?
assert_exit_code "get show exits 0" 0 "$ec"
assert_json_field "get show name" "$out" '.results[0].name' "A Sign of Affection"
assert_json_field "get show name_romaji" "$out" '.results[0].name_romaji' "Yubisaki to Renren"
assert_json_field "get show vintage" "$out" '.results[0].vintage' "Winter 2024"
assert_json_field "get show s_type" "$out" '.results[0].s_type' "TV"

# Update show
out=$(jankenoboe update show "$SHOW_ID" --data '{"name":"A Sign of Affection (Updated)"}')
ec=$?
assert_exit_code "update show exits 0" 0 "$ec"
assert_output_contains "update show response" "$out" "updated"

# Verify update
out=$(jankenoboe get show "$SHOW_ID" --fields name)
assert_json_field "get show after update" "$out" '.results[0].name' "A Sign of Affection (Updated)"

echo ""

# ---- 10. Play History & rel_show_song ----
printf "${YELLOW}--- Play History & rel_show_song ---${NC}\n"
reset_db

# Setup: create artist, song, show
a_out=$(jankenoboe create artist --data '{"name":"ChoQMay"}')
A_ID=$(echo "$a_out" | jq -r '.id')
s_out=$(jankenoboe create song --data "{\"name\":\"snowspring\",\"artist_id\":\"$A_ID\"}")
S_ID=$(echo "$s_out" | jq -r '.id')
sh_out=$(jankenoboe create show --data '{"name":"A Sign of Affection","vintage":"Winter 2024","s_type":"TV"}')
SH_ID=$(echo "$sh_out" | jq -r '.id')

# Create rel_show_song
out=$(jankenoboe create rel_show_song --data "{\"show_id\":\"$SH_ID\",\"song_id\":\"$S_ID\",\"media_url\":\"https://example.com/video1.mp4\"}")
ec=$?
assert_exit_code "create rel_show_song exits 0" 0 "$ec"

# Search rel_show_song
out=$(jankenoboe search rel_show_song --fields show_id,song_id,media_url --term "{\"show_id\":{\"value\":\"$SH_ID\"},\"song_id\":{\"value\":\"$S_ID\"}}")
ec=$?
assert_exit_code "search rel_show_song exits 0" 0 "$ec"
assert_json_field "search rel_show_song finds link" "$out" '.results | length' "1"
assert_json_field "search rel_show_song show_id" "$out" '.results[0].show_id' "$SH_ID"
assert_json_field "search rel_show_song song_id" "$out" '.results[0].song_id' "$S_ID"

# Create play_history
out=$(jankenoboe create play_history --data "{\"show_id\":\"$SH_ID\",\"song_id\":\"$S_ID\",\"media_url\":\"https://example.com/video1.mp4\"}")
ec=$?
assert_exit_code "create play_history exits 0" 0 "$ec"
PH_ID=$(echo "$out" | jq -r '.id')
assert_json_field "create play_history returns id" "$out" '.id' "$PH_ID"

echo ""

# ---- 11. Song operations (get, update, delete) ----
printf "${YELLOW}--- Song Operations ---${NC}\n"
reset_db

a_out=$(jankenoboe create artist --data '{"name":"Artist1"}')
A1_ID=$(echo "$a_out" | jq -r '.id')
a2_out=$(jankenoboe create artist --data '{"name":"Artist2"}')
A2_ID=$(echo "$a2_out" | jq -r '.id')

s_out=$(jankenoboe create song --data "{\"name\":\"TestSong\",\"artist_id\":\"$A1_ID\"}")
S_ID=$(echo "$s_out" | jq -r '.id')

# Get song with fields
out=$(jankenoboe get song "$S_ID" --fields id,name,artist_id)
ec=$?
assert_exit_code "get song exits 0" 0 "$ec"
assert_json_field "get song name" "$out" '.results[0].name' "TestSong"
assert_json_field "get song artist_id" "$out" '.results[0].artist_id' "$A1_ID"

# Update song (reassign artist_id)
out=$(jankenoboe update song "$S_ID" --data "{\"artist_id\":\"$A2_ID\"}")
ec=$?
assert_exit_code "update song exits 0" 0 "$ec"
assert_output_contains "update song response" "$out" "updated"

# Verify song reassignment
out=$(jankenoboe get song "$S_ID" --fields artist_id)
assert_json_field "song reassigned via update" "$out" '.results[0].artist_id' "$A2_ID"

# Delete song
out=$(jankenoboe delete song "$S_ID")
ec=$?
assert_exit_code "delete song exits 0" 0 "$ec"
assert_output_contains "delete song response" "$out" "deleted"

echo ""

# ---- 12. Search match modes ----
printf "${YELLOW}--- Search Match Modes ---${NC}\n"
reset_db

jankenoboe create artist --data '{"name":"Minami"}' > /dev/null
jankenoboe create artist --data '{"name":"MINAMI"}' > /dev/null
jankenoboe create artist --data '{"name":"Minimal Techno"}' > /dev/null

# exact-i (case-insensitive)
out=$(jankenoboe search artist --fields id,name --term '{"name":{"value":"minami","match":"exact-i"}}')
ec=$?
assert_exit_code "search exact-i exits 0" 0 "$ec"
assert_json_field "search exact-i finds 2 results" "$out" '.results | length' "2"

# starts-with
out=$(jankenoboe search artist --fields id,name --term '{"name":{"value":"Min","match":"starts-with"}}')
ec=$?
assert_exit_code "search starts-with exits 0" 0 "$ec"
assert_json_field "search starts-with finds 3 results" "$out" '.results | length' "3"

# contains
out=$(jankenoboe search artist --fields id,name --term '{"name":{"value":"nam","match":"contains"}}')
ec=$?
assert_exit_code "search contains exits 0" 0 "$ec"
assert_json_field "search contains finds 2 results" "$out" '.results | length' "2"

# ends-with
out=$(jankenoboe search artist --fields id,name --term '{"name":{"value":"ami","match":"ends-with"}}')
ec=$?
assert_exit_code "search ends-with exits 0" 0 "$ec"
assert_json_field "search ends-with finds 2 results" "$out" '.results | length' "2"

echo ""

# ---- 13. Multi-field search ----
printf "${YELLOW}--- Multi-field Search ---${NC}\n"
reset_db

# Create shows with different vintages
jankenoboe create show --data '{"name":"K-On!","vintage":"Spring 2009","s_type":"TV"}' > /dev/null
jankenoboe create show --data '{"name":"K-On!","vintage":"Spring 2010","s_type":"TV"}' > /dev/null
jankenoboe create show --data '{"name":"Other Show","vintage":"Spring 2009","s_type":"TV"}' > /dev/null

# Search show by name (exact-i) + vintage
out=$(jankenoboe search show --fields id,name,vintage --term '{"name":{"value":"k-on!","match":"exact-i"},"vintage":{"value":"Spring 2009"}}')
ec=$?
assert_exit_code "search show multi-field exits 0" 0 "$ec"
assert_json_field "search show multi-field finds 1 result" "$out" '.results | length' "1"
assert_json_field "search show multi-field vintage" "$out" '.results[0].vintage' "Spring 2009"

# Create artist + songs for multi-field song search
a_out=$(jankenoboe create artist --data '{"name":"SongArtistA"}')
AA_ID=$(echo "$a_out" | jq -r '.id')
a2_out=$(jankenoboe create artist --data '{"name":"SongArtistB"}')
AB_ID=$(echo "$a2_out" | jq -r '.id')

jankenoboe create song --data "{\"name\":\"SameSong\",\"artist_id\":\"$AA_ID\"}" > /dev/null
jankenoboe create song --data "{\"name\":\"SameSong\",\"artist_id\":\"$AB_ID\"}" > /dev/null

# Search song by name + artist_id
out=$(jankenoboe search song --fields id,name,artist_id --term "{\"name\":{\"value\":\"SameSong\"},\"artist_id\":{\"value\":\"$AA_ID\"}}")
ec=$?
assert_exit_code "search song multi-field exits 0" 0 "$ec"
assert_json_field "search song multi-field finds 1 result" "$out" '.results | length' "1"
assert_json_field "search song multi-field artist_id" "$out" '.results[0].artist_id' "$AA_ID"

echo ""

# ---- 14. Soft-delete (status update) ----
printf "${YELLOW}--- Soft Delete ---${NC}\n"
reset_db

a_out=$(jankenoboe create artist --data '{"name":"SoftDelArtist"}')
A_ID=$(echo "$a_out" | jq -r '.id')

# Soft-delete via status=1
out=$(jankenoboe update artist "$A_ID" --data '{"status": 1}')
ec=$?
assert_exit_code "soft-delete artist exits 0" 0 "$ec"
assert_output_contains "soft-delete response" "$out" "updated"

# Verify status updated
out=$(jankenoboe get artist "$A_ID" --fields id,name,status)
ec=$?
assert_exit_code "get soft-deleted artist exits 0" 0 "$ec"
assert_json_field "soft-deleted status is 1" "$out" '.results[0].status' "1"

echo ""

# ---- 15. Learning lifecycle (level up, level down, graduate) ----
printf "${YELLOW}--- Learning Lifecycle ---${NC}\n"
reset_db

a_out=$(jankenoboe create artist --data '{"name":"LifecycleArtist"}')
A_ID=$(echo "$a_out" | jq -r '.id')
s_out=$(jankenoboe create song --data "{\"name\":\"LifecycleSong\",\"artist_id\":\"$A_ID\"}")
S_ID=$(echo "$s_out" | jq -r '.id')

# Add to learning
batch_out=$(jankenoboe learning-batch --song-ids "$S_ID")
L_ID=$(echo "$batch_out" | jq -r '.created_ids[0]')

# Level up (level 0 -> 3)
out=$(jankenoboe update learning "$L_ID" --data '{"level": 3}')
ec=$?
assert_exit_code "level up exits 0" 0 "$ec"
assert_output_contains "level up response" "$out" "updated"

# Verify level
out=$(jankenoboe get learning "$L_ID" --fields id,level,graduated)
assert_json_field "level is 3" "$out" '.results[0].level' "3"

# Level down (3 -> 1)
out=$(jankenoboe update learning "$L_ID" --data '{"level": 1}')
ec=$?
assert_exit_code "level down exits 0" 0 "$ec"

# Verify level
out=$(jankenoboe get learning "$L_ID" --fields id,level)
assert_json_field "level is 1 after level down" "$out" '.results[0].level' "1"

# Graduate
out=$(jankenoboe update learning "$L_ID" --data '{"graduated": 1}')
ec=$?
assert_exit_code "graduate exits 0" 0 "$ec"
assert_output_contains "graduate response" "$out" "updated"

# Verify graduated
out=$(jankenoboe get learning "$L_ID" --fields id,graduated)
assert_json_field "graduated is 1" "$out" '.results[0].graduated' "1"

echo ""

# ---- 16. Learning re-learn workflow ----
printf "${YELLOW}--- Learning Re-learn ---${NC}\n"
reset_db

a_out=$(jankenoboe create artist --data '{"name":"RelearnArtist"}')
A_ID=$(echo "$a_out" | jq -r '.id')
s_out=$(jankenoboe create song --data "{\"name\":\"RelearnSong\",\"artist_id\":\"$A_ID\"}")
S_ID=$(echo "$s_out" | jq -r '.id')

# Add to learning and graduate
batch_out=$(jankenoboe learning-batch --song-ids "$S_ID")
L_ID=$(echo "$batch_out" | jq -r '.created_ids[0]')
jankenoboe update learning "$L_ID" --data '{"graduated": 1}' > /dev/null

# Attempt to add graduated song without relearn flag
out=$(jankenoboe learning-batch --song-ids "$S_ID")
ec=$?
assert_exit_code "learning-batch graduated song exits 0" 0 "$ec"
assert_json_field "already_graduated detected" "$out" '.already_graduated_song_ids | length' "1"
assert_json_field "no new created" "$out" '.created_ids | length' "0"

# Re-learn with default start level (7)
out=$(jankenoboe learning-batch --song-ids "$S_ID" --relearn-song-ids "$S_ID")
ec=$?
assert_exit_code "relearn exits 0" 0 "$ec"
assert_json_field "relearn created 1 record" "$out" '.created_ids | length' "1"
NEW_L_ID=$(echo "$out" | jq -r '.created_ids[0]')

# Verify new learning record starts at level 7
out=$(jankenoboe get learning "$NEW_L_ID" --fields id,level,graduated)
assert_json_field "relearn level is 7" "$out" '.results[0].level' "7"
assert_json_field "relearn not graduated" "$out" '.results[0].graduated' "0"

echo ""

# ---- 17. Learning re-learn with custom start level ----
printf "${YELLOW}--- Learning Re-learn Custom Level ---${NC}\n"
reset_db

a_out=$(jankenoboe create artist --data '{"name":"CustomLevelArtist"}')
A_ID=$(echo "$a_out" | jq -r '.id')
s_out=$(jankenoboe create song --data "{\"name\":\"CustomLevelSong\",\"artist_id\":\"$A_ID\"}")
S_ID=$(echo "$s_out" | jq -r '.id')

# Add to learning, graduate
batch_out=$(jankenoboe learning-batch --song-ids "$S_ID")
L_ID=$(echo "$batch_out" | jq -r '.created_ids[0]')
jankenoboe update learning "$L_ID" --data '{"graduated": 1}' > /dev/null

# Re-learn with custom start level 5
out=$(jankenoboe learning-batch --song-ids "$S_ID" --relearn-song-ids "$S_ID" --relearn-start-level 5)
ec=$?
assert_exit_code "relearn custom level exits 0" 0 "$ec"
NEW_L_ID=$(echo "$out" | jq -r '.created_ids[0]')

# Verify starts at level 5
out=$(jankenoboe get learning "$NEW_L_ID" --fields id,level)
assert_json_field "relearn level is 5" "$out" '.results[0].level' "5"

echo ""

# ---- 18. Learning song review (HTML report) ----
printf "${YELLOW}--- Learning Song Review ---${NC}\n"
reset_db

# learning-song-review on empty DB
out=$(jankenoboe learning-song-review --output /tmp/e2e_review.html)
ec=$?
assert_exit_code "learning-song-review empty exits 0" 0 "$ec"
assert_json_field "learning-song-review empty count" "$out" '.count' "0"

# Create data: artist -> song -> show -> rel_show_song -> learning
a_out=$(jankenoboe create artist --data '{"name":"ReviewArtist"}')
A_ID=$(echo "$a_out" | jq -r '.id')
s_out=$(jankenoboe create song --data "{\"name\":\"ReviewSong\",\"artist_id\":\"$A_ID\"}")
S_ID=$(echo "$s_out" | jq -r '.id')
sh_out=$(jankenoboe create show --data '{"name":"ReviewShow","vintage":"Winter 2024","s_type":"TV"}')
SH_ID=$(echo "$sh_out" | jq -r '.id')
jankenoboe create rel_show_song --data "{\"show_id\":\"$SH_ID\",\"song_id\":\"$S_ID\",\"media_url\":\"https://example.com/review.mp4\"}" > /dev/null

# Add to learning (level 0 is immediately due with 5-min warm-up after creation)
jankenoboe learning-batch --song-ids "$S_ID" > /dev/null

# Test --offset: song just created is NOT due without offset, but IS due with offset
out=$(jankenoboe learning-due --limit 10)
ec=$?
assert_exit_code "learning-due no offset exits 0" 0 "$ec"
assert_json_field "learning-due no offset count 0 (warm-up)" "$out" '.count' "0"

out=$(jankenoboe learning-due --limit 10 --offset 400)
ec=$?
assert_exit_code "learning-due with offset exits 0" 0 "$ec"
assert_json_field "learning-due with offset count 1" "$out" '.count' "1"

# Wait briefly, then generate review
# Note: Level 0 songs have a 300-second warm-up, so we manually set last_level_up_at in the past
# by using learning-due first, and then generating the review
# For e2e, we can use the --limit flag to generate the report
out=$(jankenoboe learning-song-review --output /tmp/e2e_review_with_data.html --limit 10)
ec=$?
assert_exit_code "learning-song-review with data exits 0" 0 "$ec"
assert_json_field "learning-song-review file path" "$out" '.file' "/tmp/e2e_review_with_data.html"

# Verify HTML file was created
TOTAL=$((TOTAL + 1))
if [ -f /tmp/e2e_review_with_data.html ]; then
  PASS=$((PASS + 1))
  printf "${GREEN}  ✓ HTML report file exists${NC}\n"
else
  FAIL=$((FAIL + 1))
  printf "${RED}  ✗ HTML report file not found${NC}\n"
fi

echo ""

# ---- 19. Learning song levelup-ids ----
printf "${YELLOW}--- Learning Song Levelup IDs ---${NC}\n"
reset_db

# levelup-ids with empty ids
jankenoboe learning-song-levelup-ids --ids "" 2>/tmp/e2e_stderr 1>/dev/null; ec=$?
assert_exit_code "levelup-ids empty exits 1" 1 "$ec"

# levelup-ids with nonexistent id
jankenoboe learning-song-levelup-ids --ids "nonexistent-id" 2>/tmp/e2e_stderr 1>/dev/null; ec=$?
assert_exit_code "levelup-ids not found exits 1" 1 "$ec"

# Create artist + songs + add to learning
a_out=$(jankenoboe create artist --data '{"name":"LevelupIdsArtist"}')
A_ID=$(echo "$a_out" | jq -r '.id')

s1_out=$(jankenoboe create song --data "{\"name\":\"LevelupIdsSong1\",\"artist_id\":\"$A_ID\"}")
S1_ID=$(echo "$s1_out" | jq -r '.id')
s2_out=$(jankenoboe create song --data "{\"name\":\"LevelupIdsSong2\",\"artist_id\":\"$A_ID\"}")
S2_ID=$(echo "$s2_out" | jq -r '.id')

batch_out=$(jankenoboe learning-batch --song-ids "$S1_ID,$S2_ID")
L1_ID=$(echo "$batch_out" | jq -r '.created_ids[0]')
L2_ID=$(echo "$batch_out" | jq -r '.created_ids[1]')

# Level up specific IDs (no need to be due)
out=$(jankenoboe learning-song-levelup-ids --ids "$L1_ID,$L2_ID")
ec=$?
assert_exit_code "levelup-ids exits 0" 0 "$ec"
assert_json_field "levelup-ids total processed" "$out" '.total_processed' "2"
assert_json_field "levelup-ids leveled_up_count" "$out" '.leveled_up_count' "2"
assert_json_field "levelup-ids graduated_count" "$out" '.graduated_count' "0"

# Verify levels incremented
out=$(jankenoboe get learning "$L1_ID" --fields level)
assert_json_field "song1 level after levelup-ids" "$out" '.results[0].level' "1"
out=$(jankenoboe get learning "$L2_ID" --fields level)
assert_json_field "song2 level after levelup-ids" "$out" '.results[0].level' "1"

# Level up only one of them
out=$(jankenoboe learning-song-levelup-ids --ids "$L1_ID")
ec=$?
assert_exit_code "levelup-ids single exits 0" 0 "$ec"
assert_json_field "levelup-ids single total" "$out" '.total_processed' "1"

out=$(jankenoboe get learning "$L1_ID" --fields level)
assert_json_field "song1 level after second levelup" "$out" '.results[0].level' "2"
out=$(jankenoboe get learning "$L2_ID" --fields level)
assert_json_field "song2 level unchanged" "$out" '.results[0].level' "1"

echo ""

# ---- 20c. Learning song levelup-ids with graduation ----
printf "${YELLOW}--- Levelup IDs with Graduation ---${NC}\n"
reset_db

a_out=$(jankenoboe create artist --data '{"name":"GradIdsArtist"}')
A_ID=$(echo "$a_out" | jq -r '.id')
s_out=$(jankenoboe create song --data "{\"name\":\"GradIdsSong\",\"artist_id\":\"$A_ID\"}")
S_ID=$(echo "$s_out" | jq -r '.id')

batch_out=$(jankenoboe learning-batch --song-ids "$S_ID")
L_ID=$(echo "$batch_out" | jq -r '.created_ids[0]')

# Set to max level (19)
jankenoboe update learning "$L_ID" --data '{"level": 19}' > /dev/null

# Level up should graduate
out=$(jankenoboe learning-song-levelup-ids --ids "$L_ID")
ec=$?
assert_exit_code "levelup-ids graduation exits 0" 0 "$ec"
assert_json_field "levelup-ids graduated 1" "$out" '.graduated_count' "1"
assert_json_field "levelup-ids total 1" "$out" '.total_processed' "1"

# Verify graduated
out=$(jankenoboe get learning "$L_ID" --fields graduated)
assert_json_field "song graduated after levelup-ids" "$out" '.results[0].graduated' "1"

# Trying to levelup-ids a graduated record should fail
jankenoboe learning-song-levelup-ids --ids "$L_ID" 2>/tmp/e2e_stderr 1>/dev/null; ec=$?
stderr=$(cat /tmp/e2e_stderr)
assert_exit_code "levelup-ids graduated exits 1" 1 "$ec"
assert_output_contains "levelup-ids graduated shows error" "$stderr" "graduated"

echo ""

# ---- 20d. Learning song review returns learning_ids ----
printf "${YELLOW}--- Learning Song Review learning_ids ---${NC}\n"
reset_db

# Empty review should return empty learning_ids
out=$(jankenoboe learning-song-review --output /tmp/e2e_review_ids.html)
ec=$?
assert_exit_code "review empty learning_ids exits 0" 0 "$ec"
assert_json_field "review empty learning_ids count" "$out" '.learning_ids | length' "0"

# Create data and make learning due
a_out=$(jankenoboe create artist --data '{"name":"ReviewIdsArtist"}')
A_ID=$(echo "$a_out" | jq -r '.id')
s_out=$(jankenoboe create song --data "{\"name\":\"ReviewIdsSong\",\"artist_id\":\"$A_ID\"}")
S_ID=$(echo "$s_out" | jq -r '.id')

batch_out=$(jankenoboe learning-batch --song-ids "$S_ID")
L_ID=$(echo "$batch_out" | jq -r '.created_ids[0]')

# Make it due by backdating
sqlite3 "$DB_PATH" "UPDATE learning SET last_level_up_at = 0, updated_at = 0;"

out=$(jankenoboe learning-song-review --output /tmp/e2e_review_ids2.html)
ec=$?
assert_exit_code "review with learning_ids exits 0" 0 "$ec"
assert_json_field "review learning_ids count" "$out" '.learning_ids | length' "1"
assert_json_field "review learning_ids matches" "$out" '.learning_ids[0]' "$L_ID"

# Use the returned learning_ids to levelup
REVIEW_IDS=$(echo "$out" | jq -r '.learning_ids | join(",")')
out=$(jankenoboe learning-song-levelup-ids --ids "$REVIEW_IDS")
ec=$?
assert_exit_code "levelup from review ids exits 0" 0 "$ec"
assert_json_field "levelup from review ids total" "$out" '.total_processed' "1"

out=$(jankenoboe get learning "$L_ID" --fields level)
assert_json_field "level after review+levelup flow" "$out" '.results[0].level' "1"

echo ""

# ---- 21. Learning by song IDs ----
printf "${YELLOW}--- Learning by Song IDs ---${NC}\n"
reset_db

a_out=$(jankenoboe create artist --data '{"name":"BySongArtist"}')
A_ID=$(echo "$a_out" | jq -r '.id')
s1_out=$(jankenoboe create song --data "{\"name\":\"BySongSong1\",\"artist_id\":\"$A_ID\"}")
S1_ID=$(echo "$s1_out" | jq -r '.id')
s2_out=$(jankenoboe create song --data "{\"name\":\"BySongSong2\",\"artist_id\":\"$A_ID\"}")
S2_ID=$(echo "$s2_out" | jq -r '.id')

# Add both songs to learning
batch_out=$(jankenoboe learning-batch --song-ids "$S1_ID,$S2_ID")
L1_ID=$(echo "$batch_out" | jq -r '.created_ids[0]')
L2_ID=$(echo "$batch_out" | jq -r '.created_ids[1]')

# Level up song1 to level 3
jankenoboe update learning "$L1_ID" --data '{"level": 3}' > /dev/null

# Query learning records by song IDs
out=$(jankenoboe learning-by-song-ids --song-ids "$S1_ID,$S2_ID")
ec=$?
assert_exit_code "learning-by-song-ids exits 0" 0 "$ec"
assert_json_field "learning-by-song-ids count" "$out" '.count' "2"
# Ordered by level DESC: song1 (level 3) first, song2 (level 0) second
assert_json_field "learning-by-song-ids first level" "$out" '.results[0].level' "3"
assert_json_field "learning-by-song-ids first song_name" "$out" '.results[0].song_name' "BySongSong1"
assert_json_field "learning-by-song-ids second level" "$out" '.results[1].level' "0"
assert_json_field "learning-by-song-ids second song_name" "$out" '.results[1].song_name' "BySongSong2"

# Empty song-ids should fail
jankenoboe learning-by-song-ids --song-ids "" 2>/tmp/e2e_stderr 1>/dev/null; ec=$?
assert_exit_code "learning-by-song-ids empty exits 1" 1 "$ec"

echo ""

# ---- 22. Bulk reassign by song-ids ----
printf "${YELLOW}--- Bulk Reassign by Song IDs ---${NC}\n"
reset_db

a1_out=$(jankenoboe create artist --data '{"name":"WrongArtist"}')
A1_ID=$(echo "$a1_out" | jq -r '.id')
a2_out=$(jankenoboe create artist --data '{"name":"CorrectArtist"}')
A2_ID=$(echo "$a2_out" | jq -r '.id')

s1_out=$(jankenoboe create song --data "{\"name\":\"Song1\",\"artist_id\":\"$A1_ID\"}")
S1_ID=$(echo "$s1_out" | jq -r '.id')
s2_out=$(jankenoboe create song --data "{\"name\":\"Song2\",\"artist_id\":\"$A1_ID\"}")
S2_ID=$(echo "$s2_out" | jq -r '.id')

# Reassign specific songs by IDs
out=$(jankenoboe bulk-reassign --song-ids "$S1_ID,$S2_ID" --new-artist-id "$A2_ID")
ec=$?
assert_exit_code "bulk-reassign by song-ids exits 0" 0 "$ec"
assert_json_field "bulk-reassign by song-ids count" "$out" '.reassigned_count' "2"

# Verify both songs now belong to CorrectArtist
out=$(jankenoboe get song "$S1_ID" --fields artist_id)
assert_json_field "song1 reassigned" "$out" '.results[0].artist_id' "$A2_ID"
out=$(jankenoboe get song "$S2_ID" --fields artist_id)
assert_json_field "song2 reassigned" "$out" '.results[0].artist_id' "$A2_ID"

echo ""

# ---- 23. Create learning directly ----
printf "${YELLOW}--- Create Learning Directly ---${NC}\n"
reset_db

a_out=$(jankenoboe create artist --data '{"name":"DirectLearnArtist"}')
A_ID=$(echo "$a_out" | jq -r '.id')
s_out=$(jankenoboe create song --data "{\"name\":\"DirectLearnSong\",\"artist_id\":\"$A_ID\"}")
S_ID=$(echo "$s_out" | jq -r '.id')

# Create learning record directly
out=$(jankenoboe create learning --data "{\"song_id\":\"$S_ID\",\"level_up_path\":\"[1,1,1,1,1,1,1,2,3,5,7,13,19,32,52,84,135,220,355,574]\"}")
ec=$?
assert_exit_code "create learning exits 0" 0 "$ec"
L_ID=$(echo "$out" | jq -r '.id')
assert_json_field "create learning returns id" "$out" '.id' "$L_ID"

# Verify learning record
out=$(jankenoboe get learning "$L_ID" --fields id,song_id,level,graduated)
assert_json_field "created learning song_id" "$out" '.results[0].song_id' "$S_ID"
assert_json_field "created learning level" "$out" '.results[0].level' "0"
assert_json_field "created learning graduated" "$out" '.results[0].graduated' "0"

echo ""

# ---- 24. Error handling ----
printf "${YELLOW}--- Error Handling ---${NC}\n"
reset_db

# Update nonexistent record
jankenoboe update artist nonexistent-id --data '{"name":"x"}' 2>/tmp/e2e_stderr 1>/dev/null; ec=$?
stderr=$(cat /tmp/e2e_stderr)
assert_exit_code "update nonexistent exits 1" 1 "$ec"
assert_output_contains "update nonexistent shows error" "$stderr" "error"

# Delete nonexistent record
jankenoboe delete artist nonexistent-id 2>/tmp/e2e_stderr 1>/dev/null; ec=$?
stderr=$(cat /tmp/e2e_stderr)
assert_exit_code "delete nonexistent exits 1" 1 "$ec"
assert_output_contains "delete nonexistent shows error" "$stderr" "error"

# Empty learning-batch
jankenoboe learning-batch --song-ids "" 2>/tmp/e2e_stderr 1>/dev/null; ec=$?
stderr=$(cat /tmp/e2e_stderr)
assert_exit_code "learning-batch empty exits 1" 1 "$ec"
assert_output_contains "learning-batch empty shows error" "$stderr" "error"

# bulk-reassign with no args
jankenoboe bulk-reassign 2>/tmp/e2e_stderr 1>/dev/null; ec=$?
assert_exit_code "bulk-reassign no args exits non-zero" 1 "$ec"

echo ""

# ---- 25. URL Percent-Encoding ----
printf "${YELLOW}--- URL Percent-Encoding ---${NC}\n"
reset_db

# Create artist with URL-encoded single quote: Ado%27s%20Music -> Ado's Music
out=$(jankenoboe create artist --data '{"name":"Ado%27s%20Music"}')
ec=$?
assert_exit_code "create URL-encoded artist exits 0" 0 "$ec"
ENCODED_ID=$(echo "$out" | jq -r '.id')

# Verify the stored name is decoded
out=$(jankenoboe get artist "$ENCODED_ID" --fields name)
assert_json_field "URL-encoded name stored correctly" "$out" '.results[0].name' "Ado's Music"

# Search using URL-encoded value
out=$(jankenoboe search artist --fields id,name --term '{"name":{"value":"Ado%27s%20Music"}}')
ec=$?
assert_exit_code "search URL-encoded exits 0" 0 "$ec"
assert_json_field "search URL-encoded finds 1 result" "$out" '.results | length' "1"
assert_json_field "search URL-encoded name matches" "$out" '.results[0].name' "Ado's Music"

# Update with URL-encoded double quote: The%20%22Best%22 -> The "Best"
out=$(jankenoboe update artist "$ENCODED_ID" --data '{"name":"The%20%22Best%22"}')
ec=$?
assert_exit_code "update URL-encoded exits 0" 0 "$ec"
out=$(jankenoboe get artist "$ENCODED_ID" --fields name)
assert_json_field "URL-encoded update stored correctly" "$out" '.results[0].name' 'The "Best"'

# Create song with URL-encoded parentheses: Fuwa%20Fuwa%20%28Ver.%29 -> Fuwa Fuwa (Ver.)
a_out=$(jankenoboe create artist --data '{"name":"TestEnc"}')
ENC_AID=$(echo "$a_out" | jq -r '.id')
s_out=$(jankenoboe create song --data "{\"name\":\"Fuwa%20Fuwa%20%28Ver.%29\",\"artist_id\":\"$ENC_AID\"}")
ec=$?
assert_exit_code "create URL-encoded song exits 0" 0 "$ec"
ENC_SID=$(echo "$s_out" | jq -r '.id')
out=$(jankenoboe get song "$ENC_SID" --fields name)
assert_json_field "URL-encoded song name stored" "$out" '.results[0].name' "Fuwa Fuwa (Ver.)"

# Search with URL-encoded contains mode
out=$(jankenoboe search song --fields id,name --term '{"name":{"value":"Fuwa%20%28Ver","match":"contains"}}')
ec=$?
assert_exit_code "search URL-encoded contains exits 0" 0 "$ec"
assert_json_field "search URL-encoded contains finds 1" "$out" '.results | length' "1"

# Plain text (no encoding) still works
jankenoboe create artist --data '{"name":"PlainText"}' > /dev/null
out=$(jankenoboe search artist --fields id,name --term '{"name":{"value":"PlainText"}}')
assert_json_field "plain text search still works" "$out" '.results | length' "1"

echo ""

# ---- 26. Shows by artist IDs ----
printf "${YELLOW}--- Shows by Artist IDs ---${NC}\n"
reset_db

# Setup: create 2 artists, 3 songs, 2 shows, link them via rel_show_song
a1_out=$(jankenoboe create artist --data '{"name":"ShowArtist1"}')
A1_ID=$(echo "$a1_out" | jq -r '.id')
a2_out=$(jankenoboe create artist --data '{"name":"ShowArtist2"}')
A2_ID=$(echo "$a2_out" | jq -r '.id')

s1_out=$(jankenoboe create song --data "{\"name\":\"Song1ByA1\",\"artist_id\":\"$A1_ID\"}")
S1_ID=$(echo "$s1_out" | jq -r '.id')
s2_out=$(jankenoboe create song --data "{\"name\":\"Song2ByA1\",\"artist_id\":\"$A1_ID\"}")
S2_ID=$(echo "$s2_out" | jq -r '.id')
s3_out=$(jankenoboe create song --data "{\"name\":\"Song3ByA2\",\"artist_id\":\"$A2_ID\"}")
S3_ID=$(echo "$s3_out" | jq -r '.id')

sh1_out=$(jankenoboe create show --data '{"name":"Show Alpha","vintage":"Spring 2024"}')
SH1_ID=$(echo "$sh1_out" | jq -r '.id')
sh2_out=$(jankenoboe create show --data '{"name":"Show Beta","vintage":"Fall 2024"}')
SH2_ID=$(echo "$sh2_out" | jq -r '.id')

# Link: Song1ByA1 -> Show Alpha, Song2ByA1 -> Show Beta, Song3ByA2 -> Show Alpha
jankenoboe create rel_show_song --data "{\"show_id\":\"$SH1_ID\",\"song_id\":\"$S1_ID\"}" > /dev/null
jankenoboe create rel_show_song --data "{\"show_id\":\"$SH2_ID\",\"song_id\":\"$S2_ID\"}" > /dev/null
jankenoboe create rel_show_song --data "{\"show_id\":\"$SH1_ID\",\"song_id\":\"$S3_ID\"}" > /dev/null

# Query shows for single artist
out=$(jankenoboe shows-by-artist-ids --artist-ids "$A1_ID")
ec=$?
assert_exit_code "shows-by-artist-ids single artist exits 0" 0 "$ec"
assert_json_field "shows-by-artist-ids single artist count" "$out" '.count' "2"

# Query shows for multiple artists
out=$(jankenoboe shows-by-artist-ids --artist-ids "$A1_ID,$A2_ID")
ec=$?
assert_exit_code "shows-by-artist-ids multiple artists exits 0" 0 "$ec"
assert_json_field "shows-by-artist-ids multiple artists count" "$out" '.count' "3"

# Verify returned fields
assert_json_field "shows-by-artist-ids has show_name" "$out" '.results[0].show_name' "Show Alpha"
assert_json_field "shows-by-artist-ids has artist_name" "$out" '.results[0].artist_name' "ShowArtist1"
assert_json_field "shows-by-artist-ids has vintage" "$out" '.results[0].vintage' "Spring 2024"

# Nonexistent artist returns empty
out=$(jankenoboe shows-by-artist-ids --artist-ids "nonexistent-uuid")
ec=$?
assert_exit_code "shows-by-artist-ids nonexistent exits 0" 0 "$ec"
assert_json_field "shows-by-artist-ids nonexistent count" "$out" '.count' "0"

# Empty artist-ids should fail
jankenoboe shows-by-artist-ids --artist-ids "" 2>/tmp/e2e_stderr 1>/dev/null; ec=$?
assert_exit_code "shows-by-artist-ids empty exits 1" 1 "$ec"

echo ""

# ---- 27. Songs by artist IDs ----
printf "${YELLOW}--- Songs by Artist IDs ---${NC}\n"
reset_db

# Setup: create 2 artists with songs
a1_out=$(jankenoboe create artist --data '{"name":"SongArtistAlpha"}')
SA1_ID=$(echo "$a1_out" | jq -r '.id')
a2_out=$(jankenoboe create artist --data '{"name":"SongArtistBeta"}')
SA2_ID=$(echo "$a2_out" | jq -r '.id')

jankenoboe create song --data "{\"name\":\"AlphaSong1\",\"artist_id\":\"$SA1_ID\"}" > /dev/null
jankenoboe create song --data "{\"name\":\"AlphaSong2\",\"artist_id\":\"$SA1_ID\"}" > /dev/null
jankenoboe create song --data "{\"name\":\"BetaSong1\",\"artist_id\":\"$SA2_ID\"}" > /dev/null

# Single artist query
out=$(jankenoboe songs-by-artist-ids --artist-ids "$SA1_ID")
ec=$?
assert_exit_code "songs-by-artist-ids single artist exits 0" 0 "$ec"
assert_json_field "songs-by-artist-ids single artist count" "$out" '.count' "2"
assert_json_field "songs-by-artist-ids has artist_name" "$out" '.results[0].artist_name' "SongArtistAlpha"

# Multiple artists query
out=$(jankenoboe songs-by-artist-ids --artist-ids "$SA1_ID,$SA2_ID")
ec=$?
assert_exit_code "songs-by-artist-ids multiple artists exits 0" 0 "$ec"
assert_json_field "songs-by-artist-ids multiple artists count" "$out" '.count' "3"

# Verify returned fields
assert_json_field "songs-by-artist-ids has song_name" "$out" '.results[0].song_name' "AlphaSong1"
assert_json_field "songs-by-artist-ids has song_id" "$out" '.results[0].song_id | length > 0' "true"

# Nonexistent artist returns empty
out=$(jankenoboe songs-by-artist-ids --artist-ids "nonexistent-uuid")
ec=$?
assert_exit_code "songs-by-artist-ids nonexistent exits 0" 0 "$ec"
assert_json_field "songs-by-artist-ids nonexistent count" "$out" '.count' "0"

# Empty artist-ids should fail
jankenoboe songs-by-artist-ids --artist-ids "" 2>/tmp/e2e_stderr 1>/dev/null; ec=$?
assert_exit_code "songs-by-artist-ids empty exits 1" 1 "$ec"

echo ""

# ---- 28. Learning song stats ----
printf "${YELLOW}--- Learning Song Stats ---${NC}\n"
reset_db

a_out=$(jankenoboe create artist --data '{"name":"StatsArtist"}')
A_ID=$(echo "$a_out" | jq -r '.id')
s1_out=$(jankenoboe create song --data "{\"name\":\"StatsSong1\",\"artist_id\":\"$A_ID\"}")
S1_ID=$(echo "$s1_out" | jq -r '.id')
s2_out=$(jankenoboe create song --data "{\"name\":\"StatsSong2\",\"artist_id\":\"$A_ID\"}")
S2_ID=$(echo "$s2_out" | jq -r '.id')

# Add both to learning, then backdate created_at and set last_level_up_at
jankenoboe learning-batch --song-ids "$S1_ID,$S2_ID" > /dev/null
# Backdate: song1 created 10 days ago, last_level_up_at = now (864000s = 10 days)
sqlite3 "$DB_PATH" "UPDATE learning SET created_at = created_at - 864000 WHERE song_id = '$S1_ID';"
# Backdate: song2 created 5 days ago
sqlite3 "$DB_PATH" "UPDATE learning SET created_at = created_at - 432000 WHERE song_id = '$S2_ID';"
# Set last_level_up_at to current time for both
sqlite3 "$DB_PATH" "UPDATE learning SET last_level_up_at = CAST(strftime('%s','now') AS INTEGER);"

out=$(jankenoboe learning-song-stats --song-ids "$S1_ID,$S2_ID")
ec=$?
assert_exit_code "learning-song-stats exits 0" 0 "$ec"
assert_json_field "learning-song-stats count" "$out" '.count' "2"
# Ordered by days_spent DESC: song1 (10 days) first, song2 (5 days) second
assert_json_field "learning-song-stats first song" "$out" '.results[0].song_name' "StatsSong1"
assert_json_field "learning-song-stats first days_spent" "$out" '.results[0].days_spent' "10"
assert_json_field "learning-song-stats second song" "$out" '.results[1].song_name' "StatsSong2"
assert_json_field "learning-song-stats second days_spent" "$out" '.results[1].days_spent' "5"

# Empty song-ids should fail
jankenoboe learning-song-stats --song-ids "" 2>/tmp/e2e_stderr 1>/dev/null; ec=$?
assert_exit_code "learning-song-stats empty exits 1" 1 "$ec"

echo ""

# ---- 29. Uninstall verification ----
printf "${YELLOW}--- Uninstall Verification ---${NC}\n"

# Verify binary is where we expect
TOTAL=$((TOTAL + 1))
if [ -f "$BINARY_PATH" ]; then
  PASS=$((PASS + 1))
  printf "${GREEN}  ✓ binary exists at %s${NC}\n" "$BINARY_PATH"
else
  FAIL=$((FAIL + 1))
  printf "${RED}  ✗ binary not found at %s${NC}\n" "$BINARY_PATH"
fi

# Verify which resolves correctly
which_path=$(which jankenoboe)
TOTAL=$((TOTAL + 1))
if [ "$which_path" = "$BINARY_PATH" ]; then
  PASS=$((PASS + 1))
  printf "${GREEN}  ✓ which jankenoboe resolves to %s${NC}\n" "$BINARY_PATH"
else
  FAIL=$((FAIL + 1))
  printf "${RED}  ✗ which jankenoboe resolves to '%s' (expected '%s')${NC}\n" "$which_path" "$BINARY_PATH"
fi

# Simulate removal and verify it's gone
rm "$BINARY_PATH"
TOTAL=$((TOTAL + 1))
if [ ! -f "$BINARY_PATH" ]; then
  PASS=$((PASS + 1))
  printf "${GREEN}  ✓ binary cleanly removed (file no longer exists)${NC}\n"
else
  FAIL=$((FAIL + 1))
  printf "${RED}  ✗ binary still exists after removal${NC}\n"
fi

echo ""

# --- Summary ---
printf "${YELLOW}=== Results ===${NC}\n"
printf "  Total: %d | ${GREEN}Passed: %d${NC} | ${RED}Failed: %d${NC}\n" "$TOTAL" "$PASS" "$FAIL"
echo ""

if [ "$FAIL" -gt 0 ]; then
  printf "${RED}E2E TESTS FAILED${NC}\n"
  exit 1
fi

printf "${GREEN}ALL E2E TESTS PASSED${NC}\n"
exit 0