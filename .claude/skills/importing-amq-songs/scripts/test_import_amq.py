#!/usr/bin/env python3
"""
Test import_amq.py with a temporary database.

Seeds a temp DB with some entities from the sample AMQ export,
then runs the import to verify:
  1. Complete entries get linked + play_history
  2. Missing entries are reported
  3. --missing-only skips already-processed entries

Usage:
  cargo build --release
  python3 .claude/skills/importing-amq-songs/scripts/test_import_amq.py
"""

import json
import os
import subprocess
import sys
import tempfile
import uuid

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(SCRIPT_DIR, "..", "..", "..", ".."))
IMPORT_SCRIPT = os.path.join(SCRIPT_DIR, "import_amq.py")
INIT_SQL = os.path.join(PROJECT_ROOT, "docs", "init-db.sql")

# Use release binary if available, otherwise debug
JANKENOBOE = os.path.join(PROJECT_ROOT, "target", "release", "jankenoboe")
if not os.path.exists(JANKENOBOE):
    JANKENOBOE = os.path.join(PROJECT_ROOT, "target", "debug", "jankenoboe")


def new_id():
    return str(uuid.uuid4())[:8] + str(uuid.uuid4())[:8]


# --- Test data ---
# We'll seed: 2 complete songs, leave 3 missing

# Complete song 1: "Zen Zen Zense movie ver." by RADWIMPS
#   from "Your Name." (Summer 2016)
RADWIMPS_ID = new_id()
YOUR_NAME_ID = new_id()
ZEN_ZEN_SONG_ID = new_id()

# Complete song 2: "Kimi ni Todoke" by Tomofumi Tanizawa
#   from "Kimi ni Todoke: From Me to You" (Fall 2009)
TANIZAWA_ID = new_id()
KNT_SHOW_ID = new_id()
KNT_SONG_ID = new_id()

# Missing song 1: "snowspring" by ChoQMay
#   -> artist ChoQMay NOT in DB
# Missing song 2: "Koi Suru Kokoro" by eufonius
#   -> show "Kashimashi: Girl Meets Girl" NOT in DB,
#      but artist eufonius IS in DB
EUFONIUS_ID = new_id()
# Missing song 3: "Hitohira" by Hitomi Miyahara
#   -> artist exists, show exists, but song NOT in DB
MIYAHARA_ID = new_id()
FRAGRANT_SHOW_ID = new_id()

# Namesake disambiguation: two artists named "Minami"
MINAMI_A_ID = new_id()
MINAMI_B_ID = new_id()
MINAMI_SONG_A_ID = new_id()  # "Crying for Rain" by Minami A
MINAMI_SONG_B_ID = new_id()  # "Kawaki wo Ameku" by Minami B
DOMESTIC_GF_SHOW_ID = new_id()

NOW = 1740000000


def build_test_json():
    """Build a small AMQ export with 5 songs."""
    return {
        "roomName": "Test Room",
        "startTime": "Sat Feb 14 2026",
        "songs": [
            # Song 1: COMPLETE
            {
                "songNumber": 1,
                "songInfo": {
                    "animeNames": {
                        "english": "Your Name.",
                        "romaji": "Kimi no Na wa.",
                    },
                    "artist": "RADWIMPS",
                    "songName": "Zen Zen Zense movie ver.",
                    "vintage": "Summer 2016",
                    "animeType": "movie",
                },
                "videoUrl": "https://example.com/zenzenzense.webm",
            },
            # Song 2: COMPLETE
            {
                "songNumber": 2,
                "songInfo": {
                    "animeNames": {
                        "english": "Kimi ni Todoke: From Me to You",
                        "romaji": "Kimi ni Todoke",
                    },
                    "artist": "Tomofumi Tanizawa",
                    "songName": "Kimi ni Todoke",
                    "vintage": "Fall 2009",
                    "animeType": "TV",
                },
                "videoUrl": "https://example.com/knt.webm",
            },
            # Song 3: MISSING artist (ChoQMay not in DB)
            {
                "songNumber": 3,
                "songInfo": {
                    "animeNames": {
                        "english": "A Sign of Affection",
                        "romaji": "Yubisaki to Renren",
                    },
                    "artist": "ChoQMay",
                    "songName": "snowspring",
                    "vintage": "Winter 2024",
                    "animeType": "TV",
                },
                "videoUrl": "https://example.com/snowspring.webm",
            },
            # Song 4: MISSING show
            #   (Kashimashi not in DB, eufonius IS)
            {
                "songNumber": 4,
                "songInfo": {
                    "animeNames": {
                        "english": "Kashimashi: Girl Meets Girl",
                        "romaji": "Kashimashi: Girl Meets Girl",
                    },
                    "artist": "eufonius",
                    "songName": "Koi Suru Kokoro",
                    "vintage": "Winter 2006",
                    "animeType": "TV",
                },
                "videoUrl": "https://example.com/koisuru.webm",
            },
            # Song 5: MISSING song
            #   (artist+show exist, song doesn't)
            {
                "songNumber": 5,
                "songInfo": {
                    "animeNames": {
                        "english": "The Fragrant Flower Blooms With Dignity",
                        "romaji": "Kaoru Hana wa Rin to Saku",
                    },
                    "artist": "Hitomi Miyahara",
                    "songName": "Hitohira",
                    "vintage": "Summer 2025",
                    "animeType": "TV",
                },
                "videoUrl": "https://example.com/hitohira.webm",
            },
        ],
    }


def build_namesake_json():
    """Build an AMQ export with a namesake artist entry."""
    return {
        "roomName": "Test Room",
        "startTime": "Sat Feb 14 2026",
        "songs": [
            {
                "songNumber": 1,
                "songInfo": {
                    "animeNames": {
                        "english": "Domestic Girlfriend",
                        "romaji": "Domestic na Kanojo",
                    },
                    "artist": "Minami",
                    "songName": "Kawaki wo Ameku",
                    "vintage": "Winter 2019",
                    "animeType": "TV",
                },
                "videoUrl": "https://example.com/kawaki.webm",
            },
        ],
    }


def seed_db(db_path):
    """Seed the DB with test data."""
    import sqlite3

    conn = sqlite3.connect(db_path)
    c = conn.cursor()

    # Artists
    artists = [
        (RADWIMPS_ID, "RADWIMPS", "", NOW, NOW, 0),
        (TANIZAWA_ID, "Tomofumi Tanizawa", "", NOW, NOW, 0),
        (EUFONIUS_ID, "eufonius", "", NOW, NOW, 0),
        (MIYAHARA_ID, "Hitomi Miyahara", "", NOW, NOW, 0),
        (MINAMI_A_ID, "Minami", "", NOW, NOW, 0),
        (MINAMI_B_ID, "Minami", "", NOW, NOW, 0),
    ]
    c.executemany(
        "INSERT INTO artist"
        " (id, name, name_context,"
        " created_at, updated_at, status)"
        " VALUES (?, ?, ?, ?, ?, ?)",
        artists,
    )

    # Shows
    shows = [
        (
            YOUR_NAME_ID,
            "Your Name.",
            "",  # Empty romaji — should be backfilled by import
            "Summer 2016",
            "movie",
            NOW,
            NOW,
            0,
        ),
        (
            KNT_SHOW_ID,
            "Kimi ni Todoke: From Me to You",
            "Kimi ni Todoke",
            "Fall 2009",
            "TV",
            NOW,
            NOW,
            0,
        ),
        (
            FRAGRANT_SHOW_ID,
            "The Fragrant Flower Blooms With Dignity",
            "Kaoru Hana wa Rin to Saku",
            "Summer 2025",
            "TV",
            NOW,
            NOW,
            0,
        ),
        (
            DOMESTIC_GF_SHOW_ID,
            "Domestic Girlfriend",
            "Domestic na Kanojo",
            "Winter 2019",
            "TV",
            NOW,
            NOW,
            0,
        ),
        # NOTE: no Kashimashi show
    ]
    c.executemany(
        "INSERT INTO show"
        " (id, name, name_romaji, vintage,"
        " s_type, created_at, updated_at, status)"
        " VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        shows,
    )

    # Songs
    songs = [
        (
            ZEN_ZEN_SONG_ID,
            "Zen Zen Zense movie ver.",
            "",
            RADWIMPS_ID,
            NOW,
            NOW,
            0,
        ),
        (
            KNT_SONG_ID,
            "Kimi ni Todoke",
            "",
            TANIZAWA_ID,
            NOW,
            NOW,
            0,
        ),
        (
            MINAMI_SONG_A_ID,
            "Crying for Rain",
            "",
            MINAMI_A_ID,
            NOW,
            NOW,
            0,
        ),
        (
            MINAMI_SONG_B_ID,
            "Kawaki wo Ameku",
            "",
            MINAMI_B_ID,
            NOW,
            NOW,
            0,
        ),
        # NOTE: no snowspring, no Koi Suru Kokoro,
        #       no Hitohira
    ]
    c.executemany(
        "INSERT INTO song"
        " (id, name, name_context,"
        " artist_id, created_at, updated_at, status)"
        " VALUES (?, ?, ?, ?, ?, ?, ?)",
        songs,
    )

    conn.commit()
    conn.close()


def run_cmd(cmd, env, stdin_input=None):
    """Run a command and return (returncode, stdout, stderr)."""
    result = subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        env=env,
        input=stdin_input,
    )
    return result.returncode, result.stdout, result.stderr


def run_jk(args, env):
    """Run jankenoboe and return parsed JSON."""
    cmd = [JANKENOBOE] + args
    rc, stdout, stderr = run_cmd(cmd, env)
    if rc != 0:
        return None
    try:
        return json.loads(stdout)
    except json.JSONDecodeError:
        return None


def count_table(table, db_path):
    """Count records in a table via direct sqlite3 query."""
    import sqlite3

    conn = sqlite3.connect(db_path)
    c = conn.cursor()
    c.execute(f'SELECT COUNT(*) FROM "{table}"')
    count = c.fetchone()[0]
    conn.close()
    return count


def main():
    if not os.path.exists(JANKENOBOE):
        print(f"ERROR: jankenoboe binary not found at {JANKENOBOE}")
        print("Run: cargo build --release")
        sys.exit(1)

    passed = 0
    failed = 0

    def check(desc, condition):
        nonlocal passed, failed
        if condition:
            print(f"  \u2713 PASS: {desc}")
            passed += 1
        else:
            print(f"  \u274c FAIL: {desc}")
            failed += 1

    with tempfile.TemporaryDirectory() as tmp:
        db_path = os.path.join(tmp, "test.db")
        json_path = os.path.join(tmp, "test_export.json")

        # Init DB schema
        with open(INIT_SQL, "r") as f:
            init_sql = f.read()
        import sqlite3

        conn = sqlite3.connect(db_path)
        conn.executescript(init_sql)
        conn.close()

        # Seed test data
        seed_db(db_path)

        # Write test JSON
        test_data = build_test_json()
        with open(json_path, "w") as f:
            json.dump(test_data, f)

        env = os.environ.copy()
        env["JANKENOBOE_DB"] = db_path
        # Add binary dir to PATH so import_amq.py
        # can find jankenoboe
        bin_dir = os.path.dirname(JANKENOBOE)
        env["PATH"] = bin_dir + ":" + env.get("PATH", "")

        # Verify seed data
        print("\n=== Test Setup Verification ===")
        check(
            "6 artists seeded",
            count_table("artist", db_path) == 6,
        )
        check(
            "4 shows seeded",
            count_table("show", db_path) == 4,
        )
        check(
            "4 songs seeded",
            count_table("song", db_path) == 4,
        )

        # === Test 1: First import run ===
        print("\n=== Test 1: First import run ===")
        rc, stdout, stderr = run_cmd(
            [
                sys.executable,
                IMPORT_SCRIPT,
                json_path,
            ],
            env,
        )
        print(stdout)
        if stderr:
            print(f"STDERR: {stderr}")

        check("Import exits successfully", rc == 0)
        check(
            "Output mentions 'Complete: 2'",
            "Complete: 2" in stdout,
        )
        check(
            "Output mentions 'Missing:  3'",
            "Missing:  3" in stdout,
        )
        check(
            "Missing report shows ChoQMay",
            "artist: ChoQMay" in stdout,
        )
        check(
            "Missing report shows Kashimashi",
            "Kashimashi" in stdout,
        )
        check(
            "Missing report shows Hitohira",
            "song: Hitohira" in stdout,
        )

        # Verify actionable CLI commands in report
        check(
            "CLI command for creating ChoQMay",
            "jankenoboe create artist --data" in stdout and "ChoQMay" in stdout,
        )
        check(
            "CLI command for creating Kashimashi includes name_romaji",
            "name_romaji" in stdout and "Kashimashi" in stdout,
        )
        check(
            "CLI command for creating show includes vintage and s_type",
            "Winter%202006" in stdout and "TV" in stdout,
        )

        # Verify grouped output with resolved IDs
        check(
            "Group: missing artist, show, song",
            "missing artist, show, song" in stdout,
        )
        check(
            "Group: missing show, song (artist resolved)",
            "missing show, song (artist resolved)" in stdout,
        )
        check(
            "Group: missing song (artist and show resolved)",
            "missing song (artist and show resolved)" in stdout,
        )
        # eufonius entry has resolved artist_id
        check(
            "Resolved artist_id shown for eufonius entry",
            f"artist_id: {EUFONIUS_ID}" in stdout,
        )
        # Hitohira entry has resolved artist_id
        # and show_id
        check(
            "Resolved artist_id shown for Miyahara entry",
            f"artist_id: {MIYAHARA_ID}" in stdout,
        )
        check(
            "Resolved show_id shown for Hitohira entry",
            f"show_id: {FRAGRANT_SHOW_ID}" in stdout,
        )

        # Verify DB state: 2 play_history records
        ph_count = count_table("play_history", db_path)
        check(
            f"2 play_history records created (got {ph_count})",
            ph_count == 2,
        )

        # Verify DB state: 2 rel_show_song records
        rss_count = count_table("rel_show_song", db_path)
        check(
            f"2 rel_show_song records created (got {rss_count})",
            rss_count == 2,
        )

        # Verify romaji backfill: "Your Name." had
        # empty romaji, import should have filled it
        check(
            "Output mentions romaji backfill",
            "filled romaji name" in stdout,
        )
        # Verify DB: "Your Name." now has name_romaji
        conn = sqlite3.connect(db_path)
        c = conn.cursor()
        c.execute(
            "SELECT name_romaji FROM show WHERE id=?",
            (YOUR_NAME_ID,),
        )
        romaji_val = c.fetchone()[0]
        conn.close()
        check(
            f"Your Name. romaji backfilled (got '{romaji_val}')",
            romaji_val == "Kimi no Na wa.",
        )
        # Verify: KnT show romaji NOT overwritten
        # (it already had a value)
        conn = sqlite3.connect(db_path)
        c = conn.cursor()
        c.execute(
            "SELECT name_romaji FROM show WHERE id=?",
            (KNT_SHOW_ID,),
        )
        knt_romaji = c.fetchone()[0]
        conn.close()
        check(
            f"KnT show romaji unchanged (got '{knt_romaji}')",
            knt_romaji == "Kimi ni Todoke",
        )

        # === Test 2: Re-run with --missing-only ===
        print("\n=== Test 2: Re-run with --missing-only ===")
        rc2, stdout2, stderr2 = run_cmd(
            [
                sys.executable,
                IMPORT_SCRIPT,
                "--missing-only",
                json_path,
            ],
            env,
        )
        print(stdout2)
        if stderr2:
            print(f"STDERR: {stderr2}")

        check(
            "--missing-only exits successfully",
            rc2 == 0,
        )
        check(
            "Output mentions skipping",
            "Skipped (already linked): 2" in stdout2,
        )
        check(
            "Play histories created: 0",
            "Play histories created: 0" in stdout2,
        )

        # Verify no duplicate play_history
        ph2_count = count_table("play_history", db_path)
        check(
            f"Still 2 play_history records (got {ph2_count})",
            ph2_count == 2,
        )

        # === Test 3: Re-run WITHOUT --missing-only ===
        print("\n=== Test 3: Re-run WITHOUT --missing-only (duplicates) ===")
        rc3, stdout3, stderr3 = run_cmd(
            [
                sys.executable,
                IMPORT_SCRIPT,
                json_path,
            ],
            env,
        )

        ph3_count = count_table("play_history", db_path)
        check(
            f"4 play_history records after non-flagged re-run (got {ph3_count})",
            ph3_count == 4,
        )

        # === Test 4: Seed missing, run --missing-only ===
        print("\n=== Test 4: Fix missing entities, re-run --missing-only ===")
        # Add ChoQMay artist, A Sign of Affection show,
        # snowspring song
        choqmay_id = new_id()
        sign_show_id = new_id()
        snowspring_id = new_id()

        conn = sqlite3.connect(db_path)
        c = conn.cursor()
        c.execute(
            "INSERT INTO artist"
            " (id, name, name_context,"
            " created_at, updated_at, status)"
            " VALUES (?, ?, ?, ?, ?, ?)",
            (choqmay_id, "ChoQMay", "", NOW, NOW, 0),
        )
        c.execute(
            "INSERT INTO show"
            " (id, name, name_romaji, vintage,"
            " s_type, created_at, updated_at, status)"
            " VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            (
                sign_show_id,
                "A Sign of Affection",
                "Yubisaki to Renren",
                "Winter 2024",
                "TV",
                NOW,
                NOW,
                0,
            ),
        )
        c.execute(
            "INSERT INTO song"
            " (id, name, name_context,"
            " artist_id, created_at, updated_at, status)"
            " VALUES (?, ?, ?, ?, ?, ?, ?)",
            (
                snowspring_id,
                "snowspring",
                "",
                choqmay_id,
                NOW,
                NOW,
                0,
            ),
        )
        conn.commit()
        conn.close()

        rc4, stdout4, stderr4 = run_cmd(
            [
                sys.executable,
                IMPORT_SCRIPT,
                "--missing-only",
                json_path,
            ],
            env,
        )
        print(stdout4)

        check(
            "Now Complete: 3 (snowspring resolved)",
            "Complete: 3" in stdout4,
        )
        check(
            "Skipped 2 already-linked entries",
            "Skipped (already linked): 2" in stdout4,
        )
        check(
            "Play histories created: 1 (snowspring)",
            "Play histories created: 1" in stdout4,
        )

        # Verify: 5 total play_history
        #   (4 from test 3 + 1 new)
        ph4_count = count_table("play_history", db_path)
        check(
            f"5 total play_history records (got {ph4_count})",
            ph4_count == 5,
        )

        # === Test 5: Namesake artist disambiguation ===
        print("\n=== Test 5: Namesake artist disambiguation ===")

        # Write namesake test JSON
        namesake_path = os.path.join(tmp, "namesake_export.json")
        with open(namesake_path, "w") as f:
            json.dump(build_namesake_json(), f)

        # 5a: Select an existing artist (option "1")
        print("\n--- Test 5a: Select existing artist ---")
        rc5a, stdout5a, stderr5a = run_cmd(
            [
                sys.executable,
                IMPORT_SCRIPT,
                namesake_path,
            ],
            env,
            stdin_input="1\n",
        )
        print(stdout5a)
        if stderr5a:
            print(f"STDERR: {stderr5a}")

        check(
            "5a: Exits successfully",
            rc5a == 0,
        )
        check(
            "5a: Prompt shows Multiple artists",
            "Multiple artists named" in stdout5a,
        )
        check(
            "5a: Shows both Minami artist IDs",
            MINAMI_A_ID in stdout5a and MINAMI_B_ID in stdout5a,
        )
        check(
            "5a: Shows song lists for artists",
            "Crying for Rain" in stdout5a or "Kawaki wo Ameku" in stdout5a,
        )
        check(
            "5a: Shows context (song + show)",
            "Kawaki wo Ameku" in stdout5a and "Domestic Girlfriend" in stdout5a,
        )
        check(
            "5a: Selected artist confirmed",
            "Selected artist:" in stdout5a,
        )

        # 5b: Skip the entry (option "4" = skip)
        print("\n--- Test 5b: Skip namesake entry ---")
        rc5b, stdout5b, stderr5b = run_cmd(
            [
                sys.executable,
                IMPORT_SCRIPT,
                "--missing-only",
                namesake_path,
            ],
            env,
            stdin_input="4\n",
        )
        print(stdout5b)

        check(
            "5b: Exits successfully",
            rc5b == 0,
        )
        check(
            "5b: Skipping confirmed",
            "Skipping this entry" in stdout5b,
        )
        check(
            "5b: Entry reported as missing",
            "Missing:" in stdout5b and "artist: Minami" in stdout5b,
        )

        # 5c: Create a NEW artist (option "3")
        print("\n--- Test 5c: Create new namesake artist ---")
        artist_count_before = count_table("artist", db_path)
        rc5c, stdout5c, stderr5c = run_cmd(
            [
                sys.executable,
                IMPORT_SCRIPT,
                "--missing-only",
                namesake_path,
            ],
            env,
            stdin_input="3\n",
        )
        print(stdout5c)

        check(
            "5c: Exits successfully",
            rc5c == 0,
        )
        check(
            "5c: Created new artist confirmed",
            "Created new artist:" in stdout5c,
        )
        artist_count_after = count_table("artist", db_path)
        check(
            "5c: New artist record created"
            f" ({artist_count_before}"
            f" -> {artist_count_after})",
            artist_count_after == artist_count_before + 1,
        )

    # Summary
    total = passed + failed
    print(f"\n{'=' * 40}")
    print(f"Results: {passed}/{total} passed")
    if failed:
        print(f"  {failed} FAILED")
        sys.exit(1)
    else:
        print("  All tests passed!")
        sys.exit(0)


if __name__ == "__main__":
    main()
