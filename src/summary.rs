use crate::{err, utils::get_db_conn, HandlerState};

use axum::{
    extract::{Path, State},
    Json,
};
use jankenstore::sqlite::{
    basics::{CountConfig, FetchConfig},
    read::{all, count},
    shift::val::v_txt,
    sql::in_them,
};
use rusqlite::types;
use serde_json::{from_str, json, Value};

use std::{collections::HashMap, sync::Arc};

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
            where_config: Some((
                "graduated=1 and song_id not in (select song_id from learning where graduated=0)",
                &[],
            )),
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

pub async fn handle_media_urls(
    State(handler_state): State<Arc<crate::HandlerState>>,
    Path(song_ids): Path<String>,
) -> Result<Json<Value>, err::AppError> {
    let HandlerState {
        schema_family,
        db_path,
    } = handler_state.as_ref();

    let song_ids = from_str::<Vec<String>>(&song_ids)?;

    let conn = get_db_conn(db_path)?;

    let where_cond = in_them(
        "song_id",
        &song_ids.iter().map(|id| v_txt(id)).collect::<Vec<_>>(),
    );
    let (play_histories, _) = all(
        &conn,
        schema_family,
        "play_history",
        Some(FetchConfig {
            where_config: Some((
                format!(
                    "{} and (media_url is not null or media_url != '' or media_url != \"''\")",
                    where_cond.0
                )
                .as_str(),
                &where_cond.1,
            )),
            display_cols: Some(&["media_url", "song_id"]),
            is_distinct: true,
            ..Default::default()
        }),
        true,
    )?;

    let media_link_map = play_histories.iter().fold(
        HashMap::<String, Vec<String>>::new(),
        |mut acc, play_history| {
            let song_id = match &play_history["song_id"] {
                types::Value::Text(song_id) => song_id,
                _ => "",
            };
            let media_url = match &play_history["media_url"] {
                types::Value::Text(media_url) => media_url,
                _ => "",
            };
            let mut existing_urls: Vec<String> =
                acc.entry(song_id.to_string()).or_default().clone();
            if existing_urls.is_empty() {
                acc.insert(song_id.to_string(), vec![media_url.to_string()]);
            } else {
                existing_urls.push(media_url.to_string());
                acc.insert(song_id.to_string(), existing_urls);
            }
            acc
        },
    );
    Ok(Json(json!(media_link_map)))
}
