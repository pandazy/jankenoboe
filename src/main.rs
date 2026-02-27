use clap::{Parser, Subcommand};
use jankenoboe::commands;
use jankenoboe::db;
use jankenoboe::error::exit_with_error;

#[derive(Parser)]
#[command(name = "jankenoboe", version, about = "Anime song learning system")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get a record by ID
    Get {
        /// Table name
        table: String,
        /// Record UUID
        id: String,
        /// Comma-separated list of field names to return
        #[arg(long)]
        fields: String,
    },
    /// Search records with table-specific filters
    Search {
        /// Table name
        table: String,
        /// JSON object mapping column names to {value, match} pairs. String values are URL percent-decoded (e.g., %27 → ', %20 → space).
        #[arg(long)]
        term: String,
        /// Comma-separated list of field names to return
        #[arg(long)]
        fields: String,
    },
    /// Find duplicate records by name
    Duplicates {
        /// Table name
        table: String,
    },
    /// Create a new record
    Create {
        /// Table name
        table: String,
        /// JSON object with field values. String values are URL percent-decoded (e.g., %27 → ', %20 → space).
        #[arg(long)]
        data: String,
    },
    /// Update a record
    Update {
        /// Table name
        table: String,
        /// Record UUID
        id: String,
        /// JSON object with fields to update. String values are URL percent-decoded (e.g., %27 → ', %20 → space).
        #[arg(long)]
        data: String,
    },
    /// Delete a record
    Delete {
        /// Table name
        table: String,
        /// Record UUID
        id: String,
    },
    /// Get songs due for review
    LearningDue {
        /// Maximum number of results
        #[arg(long, default_value = "100")]
        limit: u32,
        /// Look-ahead offset in seconds (e.g., 7200 for 2 hours into the future). Default 0 = now only.
        #[arg(long, default_value = "0")]
        offset: u32,
    },
    /// Add songs to learning
    LearningBatch {
        /// Comma-separated song UUIDs
        #[arg(long)]
        song_ids: String,
        /// Comma-separated song UUIDs of graduated songs to re-learn
        #[arg(long)]
        relearn_song_ids: Option<String>,
        /// Starting level for re-learned songs (0-indexed, default: 7)
        #[arg(long, default_value = "7")]
        relearn_start_level: u32,
    },
    /// Generate HTML report of due songs with enriched data
    LearningSongReview {
        /// Output file path
        #[arg(long, default_value = "learning-song-review.html")]
        output: String,
        /// Maximum number of due songs to include
        #[arg(long, default_value = "500")]
        limit: u32,
        /// Look-ahead offset in seconds (e.g., 7200 for 2 hours into the future). Default 0 = now only.
        #[arg(long, default_value = "0")]
        offset: u32,
    },
    /// Level up specific learning records by their IDs
    LearningSongLevelupIds {
        /// Comma-separated learning UUIDs
        #[arg(long)]
        ids: String,
    },
    /// Get learning records by song IDs
    LearningBySongIds {
        /// Comma-separated song UUIDs
        #[arg(long)]
        song_ids: String,
    },
    /// Get learning stats per song (days spent learning)
    LearningSongStats {
        /// Comma-separated song UUIDs
        #[arg(long)]
        song_ids: String,
    },
    /// Get all shows where given artists have song performances
    ShowsByArtistIds {
        /// Comma-separated artist UUIDs
        #[arg(long)]
        artist_ids: String,
    },
    /// Get all songs by given artists
    SongsByArtistIds {
        /// Comma-separated artist UUIDs
        #[arg(long)]
        artist_ids: String,
    },
    /// Reassign multiple songs to a different artist
    BulkReassign {
        /// Comma-separated song UUIDs (mode 1)
        #[arg(long)]
        song_ids: Option<String>,
        /// Target artist UUID (mode 1)
        #[arg(long)]
        new_artist_id: Option<String>,
        /// Source artist UUID (mode 2)
        #[arg(long)]
        from_artist_id: Option<String>,
        /// Target artist UUID (mode 2)
        #[arg(long)]
        to_artist_id: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    let mut conn = match db::open_connection() {
        Ok(c) => c,
        Err(e) => exit_with_error(&e),
    };

    let result = match cli.command {
        Commands::Get { table, id, fields } => commands::cmd_get(&mut conn, &table, &id, &fields),
        Commands::Search {
            table,
            term,
            fields,
        } => commands::cmd_search(&mut conn, &table, &term, &fields),
        Commands::Duplicates { table } => commands::cmd_duplicates(&mut conn, &table),
        Commands::Create { table, data } => commands::cmd_create(&mut conn, &table, &data),
        Commands::Update { table, id, data } => commands::cmd_update(&mut conn, &table, &id, &data),
        Commands::Delete { table, id } => commands::cmd_delete(&mut conn, &table, &id),
        Commands::LearningDue { limit, offset } => {
            commands::cmd_learning_due(&mut conn, limit, offset)
        }
        Commands::LearningBatch {
            song_ids,
            relearn_song_ids,
            relearn_start_level,
        } => commands::cmd_learning_batch(
            &mut conn,
            &song_ids,
            relearn_song_ids.as_deref(),
            relearn_start_level,
        ),
        Commands::LearningSongReview {
            output,
            limit,
            offset,
        } => commands::cmd_learning_song_review(&mut conn, &output, limit, offset),
        Commands::LearningSongLevelupIds { ids } => {
            commands::cmd_learning_song_levelup_ids(&mut conn, &ids)
        }
        Commands::LearningBySongIds { song_ids } => {
            commands::cmd_learning_by_song_ids(&mut conn, &song_ids)
        }
        Commands::LearningSongStats { song_ids } => {
            commands::cmd_learning_song_stats(&mut conn, &song_ids)
        }
        Commands::ShowsByArtistIds { artist_ids } => {
            commands::cmd_shows_by_artist_ids(&mut conn, &artist_ids)
        }
        Commands::SongsByArtistIds { artist_ids } => {
            commands::cmd_songs_by_artist_ids(&mut conn, &artist_ids)
        }
        Commands::BulkReassign {
            song_ids,
            new_artist_id,
            from_artist_id,
            to_artist_id,
        } => commands::cmd_bulk_reassign(
            &mut conn,
            song_ids.as_deref(),
            new_artist_id.as_deref(),
            from_artist_id.as_deref(),
            to_artist_id.as_deref(),
        ),
    };

    match result {
        Ok(output) => {
            println!("{output}");
        }
        Err(e) => exit_with_error(&e),
    }
}
