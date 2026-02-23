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
                        "english": "Kimi ni Todoke:" " From Me to You",
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
                        "english": "The Fragrant Flower" " Blooms With Dignity",
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
            "Kimi no Na wa.",
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


def run_cmd(cmd, env):
    """Run a command and return (returncode, stdout, stderr)."""
    result = subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        env=env,
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
        print(f"ERROR: jankenoboe binary not found at" f" {JANKENOBOE}")
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
            "4 artists seeded",
            count_table("artist", db_path) == 4,
        )
        check(
            "3 shows seeded",
            count_table("show", db_path) == 3,
        )
        check(
            "2 songs seeded",
            count_table("song", db_path) == 2,
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

        # Verify DB state: 2 play_history records
        ph_count = count_table("play_history", db_path)
        check(
            f"2 play_history records created" f" (got {ph_count})",
            ph_count == 2,
        )

        # Verify DB state: 2 rel_show_song records
        rss_count = count_table("rel_show_song", db_path)
        check(
            f"2 rel_show_song records created" f" (got {rss_count})",
            rss_count == 2,
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
            f"Still 2 play_history records" f" (got {ph2_count})",
            ph2_count == 2,
        )

        # === Test 3: Re-run WITHOUT --missing-only ===
        print("\n=== Test 3: Re-run WITHOUT" " --missing-only (duplicates) ===")
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
            f"4 play_history records after" f" non-flagged re-run (got {ph3_count})",
            ph3_count == 4,
        )

        # === Test 4: Seed missing, run --missing-only ===
        print("\n=== Test 4: Fix missing entities," " re-run --missing-only ===")
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
            f"5 total play_history records" f" (got {ph4_count})",
            ph4_count == 5,
        )

    # Summary
    total = passed + failed
    print(f"\n{'='*40}")
    print(f"Results: {passed}/{total} passed")
    if failed:
        print(f"  {failed} FAILED")
        sys.exit(1)
    else:
        print("  All tests passed!")
        sys.exit(0)


if __name__ == "__main__":
    main()
