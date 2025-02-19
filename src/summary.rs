use crate::{err, utils::get_db_conn, HandlerState};

use axum::{extract::State, Json};
use jankenstore::sqlite::{basics::CountConfig, read::count};
use serde_json::{json, Value};

use std::sync::Arc;

pub async fn handle_summary(
    State(handler_state): State<Arc<crate::HandlerState>>,
) -> Result<Json<Value>, err::AppError> {
    let HandlerState {
        schema_family,
        db_path,
    } = handler_state.as_ref();
    let common_config = CountConfig {
        where_config: Some(("status=0", &[])),
        ..Default::default()
    };
    let conn = get_db_conn(db_path)?;

    let total_shows = count(&conn, schema_family, "show", Some(common_config))?;

    let total_songs = count(&conn, schema_family, "song", Some(common_config))?;

    let total_artists = count(&conn, schema_family, "artist", Some(common_config))?;

    let total_learning_songs = count(
        &conn,
        schema_family,
        "learning",
        Some(CountConfig {
            distinct_field: Some("song_id"),
            where_config: Some(("graduated=0", &[])),
        }),
    )?;

    let total_graduated_songs = count(
        &conn,
        schema_family,
        "learning",
        Some(CountConfig {
            distinct_field: Some("song_id"),
            where_config: Some(("graduated=1", &[])),
        }),
    )?;

    Ok(Json(json!({
      "totalShows": total_shows,
      "totalSongs": total_songs,
      "totalArtists": total_artists,
      "totalLearningSongs": total_learning_songs,
      "totalGraduatedSongs": total_graduated_songs,
    })))
}
