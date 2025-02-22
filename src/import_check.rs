use axum::{extract::State, Json};
use hyper::StatusCode;
use jankenstore::sqlite::{
    add::create,
    basics::FetchConfig,
    peer::link,
    read::all,
    schema::SchemaFamily,
    shift::{
        val::{v_int, v_txt},
        val_to_json,
    },
};
use rusqlite::{types, Connection};
use serde_json::{json, Value};
use uuid::Uuid;

use std::{collections::HashMap, sync::Arc};

use crate::{
    err,
    utils::{get_db_conn, get_timestamp},
    HandlerState,
};

fn get_show_exist_query(
    conn: &Connection,
    schema_family: &SchemaFamily,
    name: &str,
    vintage: &str,
) -> Result<(Vec<HashMap<String, types::Value>>, u64), err::AppError> {
    let (shows, count) = all(
        conn,
        schema_family,
        "show",
        Some(FetchConfig {
            where_config: Some(("name=? and vintage=?", &[v_txt(name), v_txt(vintage)])),
            ..Default::default()
        }),
        false,
    )?;
    Ok((shows, count))
}

fn get_artist_exist_query(
    conn: &Connection,
    schema_family: &SchemaFamily,
    name: &str,
) -> Result<(Vec<HashMap<String, types::Value>>, u64), err::AppError> {
    let (artists, count) = all(
        conn,
        schema_family,
        "artist",
        Some(FetchConfig {
            where_config: Some(("name=?", &[v_txt(name)])),
            ..Default::default()
        }),
        false,
    )?;
    Ok((artists, count))
}

fn get_song_exist_query(
    conn: &Connection,
    schema_family: &SchemaFamily,
    name: &str,
    artist_id: &str,
) -> Result<(Vec<HashMap<String, types::Value>>, u64), err::AppError> {
    let (songs, count) = all(
        conn,
        schema_family,
        "song",
        Some(FetchConfig {
            where_config: Some(("name=? and artist_id=?", &[v_txt(name), v_txt(artist_id)])),
            ..Default::default()
        }),
        false,
    )?;
    Ok((songs, count))
}

pub async fn handle_import_check(
    State(handler_state): State<Arc<crate::HandlerState>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, err::AppError> {
    let HandlerState {
        schema_family,
        db_path,
    } = handler_state.as_ref();
    let conn = get_db_conn(db_path)?;

    let import_songs = body["songs"]
        .as_array()
        .ok_or(anyhow::anyhow!(err::http_err_msg(
            "import_data must be an array",
            StatusCode::BAD_REQUEST,
        )))?;
    let dup_artist_map =
        body["dupArtistMap"]
            .as_object()
            .ok_or(anyhow::anyhow!(err::http_err_msg(
                "dupArtistMap must be an object",
                StatusCode::BAD_REQUEST,
            )))?;

    let mut straight_records = vec![];
    let mut to_be_decided_records = vec![];

    for record in import_songs {
        let song_info = record["songInfo"].clone();
        let new_video_url = record["videoUrl"].as_str().unwrap_or("");
        let show_name = song_info["animeNames"]["english"]
            .as_str()
            .ok_or(anyhow::anyhow!(err::http_err_msg(
                format!(
                    "failed to get show's English name: {:?}",
                    record["songInfo"]
                )
                .as_str(),
                StatusCode::BAD_REQUEST,
            )))?;
        let vintage = song_info["vintage"]
            .as_str()
            .ok_or(anyhow::anyhow!(err::http_err_msg(
                format!("failed to get show's vintage: {:?}", record["songInfo"]).as_str(),
                StatusCode::BAD_REQUEST,
            )))?;
        let artist_name =
            song_info["artist"]
                .as_str()
                .ok_or(anyhow::anyhow!(err::http_err_msg(
                    format!("failed to get artist's name: {:?}", record["songInfo"]).as_str(),
                    StatusCode::BAD_REQUEST,
                )))?;
        let song_name =
            song_info["songName"]
                .as_str()
                .ok_or(anyhow::anyhow!(err::http_err_msg(
                    format!("failed to get song's name: {:?}", record["songInfo"]).as_str(),
                    StatusCode::BAD_REQUEST,
                )))?;

        let mut is_straight = true;

        let (shows, count) = get_show_exist_query(&conn, schema_family, show_name, vintage)?;
        let show_json = if count != 1 {
            is_straight = false;
            let mut existing_shows = vec![];
            for show in shows {
                existing_shows.push(val_to_json(&show)?);
            }
            let show_js = json!({
                "name": show_name,
                "vintage": vintage,
                "$tbd_options": existing_shows,
                "$tbd": true,
            });
            show_js
        } else {
            val_to_json(&shows[0])?
        };

        let (artists, count) = get_artist_exist_query(&conn, schema_family, artist_name)?;
        let artist_json = if count != 1 {
            if dup_artist_map.contains_key(artist_name) {
                is_straight = true;
                let dup_artist_id = dup_artist_map[artist_name].as_str().unwrap_or("");
                let artist = artists.iter().find(|artist| match &artist["id"] {
                    types::Value::Text(id) => id == dup_artist_id,
                    _ => false,
                });
                val_to_json(artist.ok_or(anyhow::anyhow!(err::http_err_msg(
                        format!(
                            "failed to get artist: {:?} with dup_artist_id: {}",
                            record["songInfo"], dup_artist_id
                        )
                        .as_str(),
                        StatusCode::BAD_REQUEST,
                    )))?)?
            } else {
                is_straight = false;
                let mut existing_artists = vec![];
                for artist in artists {
                    existing_artists.push(val_to_json(&artist)?);
                }
                json!({
                    "name": artist_name,
                    "$tbd_options": existing_artists,
                    "$tbd": true,
                })
            }
        } else {
            val_to_json(&artists[0])?
        };

        let default_artist_id = json!("");
        let artist_id = artist_json.get("id").unwrap_or(&default_artist_id);
        let artist_id = artist_id.as_str().unwrap_or("");
        let (songs, count) = if artist_id.trim().is_empty() {
            (vec![], 0)
        } else {
            get_song_exist_query(&conn, schema_family, song_name, artist_id)?
        };
        let song_json = if count != 1 {
            is_straight = false;
            let mut existing_songs = vec![];
            for song in songs {
                existing_songs.push(val_to_json(&song)?);
            }
            let mut song_js = json!({
                "name": song_name,
                "$tbd_options": existing_songs,
                "$tbd": true,
            });
            if !artist_id.trim().is_empty() {
                song_js["artist_id"] = json!(artist_id);
            }
            song_js
        } else {
            val_to_json(&songs[0])?
        };

        if is_straight {
            straight_records.push(json!({
                "show": show_json,
                "artist": artist_json,
                "song": song_json,
                "videoUrl": new_video_url,
            }));
        } else {
            to_be_decided_records.push(json!({
                "show": show_json,
                "artist": artist_json,
                "song": song_json,
                "videoUrl": new_video_url,
            }));
        }
    }

    Ok(Json(json!({
        "straightRecords": straight_records,
        "toBeDecidedRecords": to_be_decided_records,
    })))
}

pub async fn handle_play_history(
    State(handler_state): State<Arc<crate::HandlerState>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, err::AppError> {
    let HandlerState {
        schema_family,
        db_path,
    } = handler_state.as_ref();
    let conn = get_db_conn(db_path)?;

    let schema = schema_family.try_get_schema("play_history")?;

    let play_history = body.as_array().ok_or(anyhow::anyhow!(err::http_err_msg(
        "play_history must be an array",
        StatusCode::BAD_REQUEST,
    )))?;

    for record in play_history {
        let show_id = record["showId"].as_str().unwrap_or("");
        let song_id = record["songId"].as_str().unwrap_or("");
        let new_video_url = record["videoUrl"].as_str().unwrap_or("");
        let new_video_url = if new_video_url == "''" {
            ""
        } else {
            new_video_url
        };

        let mut input = HashMap::from([
            (
                schema.pk.clone(),
                v_txt(Uuid::new_v4().to_string().as_str()),
            ),
            ("show_id".to_string(), v_txt(show_id)),
            ("song_id".to_string(), v_txt(song_id)),
            ("created_at".to_string(), v_int(get_timestamp() as i64)),
        ]);
        if schema.types.contains_key("media_url") {
            input.insert("media_url".to_string(), v_txt(new_video_url));
        }

        create(&conn, schema_family, "play_history", &input, false)?;
        link(
            &conn,
            schema_family,
            &HashMap::from([
                ("show".to_string(), vec![v_txt(show_id)]),
                ("song".to_string(), vec![v_txt(song_id)]),
            ]),
        )?;
    }

    Ok(Json(json!({
        "success": true,
    })))
}
