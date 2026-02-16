-- Jankenoboe Database Initialization
-- Run: sqlite3 datasource.db < docs/init-db.sql

CREATE TABLE IF NOT EXISTS "artist" (
	"id"	TEXT,
	"name"	TEXT NOT NULL,
	"name_context"	TEXT DEFAULT '',
	"created_at"	INTEGER,
	"updated_at"	INTEGER,
	"status"	INTEGER NOT NULL DEFAULT 0, -- 0: normal, 1: deleted
	PRIMARY KEY("id")
);

CREATE TABLE IF NOT EXISTS "show" (
	"id" TEXT PRIMARY KEY,
	"name" TEXT NOT NULL,
	"name_romaji" TEXT DEFAULT '',
	"vintage" TEXT DEFAULT '',
	"s_type" TEXT DEFAULT '',
	"created_at" INTEGER,
	"updated_at" INTEGER,
	"status" INTEGER NOT NULL DEFAULT 0 -- 0: normal, 1: deleted
);

CREATE TABLE IF NOT EXISTS "song" (
	"id" TEXT PRIMARY KEY,
	"name" TEXT NOT NULL,
	"name_context" TEXT DEFAULT '',
	"artist_id" TEXT NOT NULL,
	"created_at" INTEGER,
	"updated_at" INTEGER,
	"status" INTEGER NOT NULL DEFAULT 0 -- 0: normal, 1: deleted
);

CREATE TABLE IF NOT EXISTS "play_history" (
	"id" TEXT PRIMARY KEY,
	"show_id" TEXT NOT NULL,
	"song_id" TEXT NOT NULL,
	"created_at" INTEGER NOT NULL,
	"media_url" TEXT DEFAULT ''
);

CREATE TABLE IF NOT EXISTS "learning" (
	"id" TEXT PRIMARY KEY,
	"song_id" TEXT NOT NULL,
	"level" INTEGER NOT NULL DEFAULT 0,
	"created_at" INTEGER NOT NULL,
	"updated_at" INTEGER NOT NULL,
	"last_level_up_at" INTEGER NOT NULL,
	"level_up_path" TEXT NOT NULL,
	"graduated" INTEGER NOT NULL DEFAULT 0, -- 0: in progress, 1: graduated
	FOREIGN KEY("song_id") REFERENCES "song"("id")
);

CREATE TABLE IF NOT EXISTS "rel_show_song" (
	"show_id" TEXT NOT NULL,
	"song_id" TEXT NOT NULL,
	"media_url" TEXT,
	"created_at" INTEGER,
	CONSTRAINT "unique_song_show_rel_show_song" UNIQUE("show_id", "song_id"),
	FOREIGN KEY("song_id") REFERENCES "song"("id") ON DELETE CASCADE,
	FOREIGN KEY("show_id") REFERENCES "show"("id") ON DELETE CASCADE
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_learning_song_id ON learning(song_id);
CREATE INDEX IF NOT EXISTS idx_rel_show_song_song_id ON rel_show_song(song_id);
CREATE INDEX IF NOT EXISTS idx_rel_show_song_show_id ON rel_show_song(show_id);