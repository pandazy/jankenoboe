//! Centralized per-table field configuration.
//!
//! Single source of truth for which fields each table supports
//! across all operations (get, search, create, update).
//! Eliminates scattered match-statement duplication in models.rs.

use serde_json::{Value, json};

/// Per-table field metadata.
pub struct TableConfig {
    /// Fields selectable via --fields (used by get, search)
    pub selectable: &'static [&'static str],
    /// Columns searchable via --term keys (used by search)
    pub searchable: &'static [&'static str],
    /// Fields writable via --data for create
    pub creatable: &'static [&'static str],
    /// Fields writable via --data for update
    pub updatable: &'static [&'static str],
}

static ARTIST: TableConfig = TableConfig {
    selectable: &[
        "id",
        "name",
        "name_context",
        "created_at",
        "updated_at",
        "status",
    ],
    searchable: &["name", "name_context"],
    creatable: &["name", "name_context"],
    updatable: &["name", "name_context", "status"],
};

static SHOW: TableConfig = TableConfig {
    selectable: &[
        "id",
        "name",
        "name_romaji",
        "vintage",
        "s_type",
        "created_at",
        "updated_at",
        "status",
    ],
    searchable: &["name", "name_romaji", "vintage"],
    creatable: &["name", "name_romaji", "vintage", "s_type"],
    updatable: &["name", "name_romaji", "vintage", "s_type", "status"],
};

static SONG: TableConfig = TableConfig {
    selectable: &[
        "id",
        "name",
        "name_context",
        "artist_id",
        "created_at",
        "updated_at",
        "status",
    ],
    searchable: &["name", "name_context", "artist_id"],
    creatable: &["name", "name_context", "artist_id"],
    updatable: &["name", "name_context", "artist_id", "status"],
};

static PLAY_HISTORY: TableConfig = TableConfig {
    selectable: &[
        "id",
        "show_id",
        "song_id",
        "created_at",
        "media_url",
        "status",
    ],
    searchable: &["show_id", "song_id"],
    creatable: &["show_id", "song_id", "media_url"],
    updatable: &["show_id", "song_id", "media_url", "status"],
};

static LEARNING: TableConfig = TableConfig {
    selectable: &[
        "id",
        "song_id",
        "level",
        "created_at",
        "updated_at",
        "last_level_up_at",
        "level_up_path",
        "graduated",
    ],
    searchable: &[
        "song_id",
        "level",
        "graduated",
        "last_level_up_at",
        "level_up_path",
    ],
    creatable: &["song_id", "level_up_path"],
    updatable: &["level", "graduated"],
};

static REL_SHOW_SONG: TableConfig = TableConfig {
    selectable: &["show_id", "song_id", "media_url", "created_at"],
    searchable: &["show_id", "song_id"],
    creatable: &["show_id", "song_id", "media_url"],
    updatable: &[],
};

/// All known table configurations.
const ALL_TABLES: &[(&str, &TableConfig)] = &[
    ("artist", &ARTIST),
    ("show", &SHOW),
    ("song", &SONG),
    ("play_history", &PLAY_HISTORY),
    ("learning", &LEARNING),
    ("rel_show_song", &REL_SHOW_SONG),
];

/// Look up the config for a table by name.
pub fn get(table: &str) -> Option<&'static TableConfig> {
    ALL_TABLES
        .iter()
        .find(|(name, _)| *name == table)
        .map(|(_, config)| *config)
}

/// Build an `enumif` JSON object mapping table names to their selectable fields.
/// Used for JankenSQLHub `~[fields]` with `enumif` constraints.
///
/// Only includes tables in the given `tables` slice.
pub fn build_selectable_enumif(tables: &[&str]) -> Value {
    let mut map = serde_json::Map::new();
    for (name, config) in ALL_TABLES {
        if tables.contains(name) {
            map.insert((*name).to_string(), json!(config.selectable));
        }
    }
    json!({ "table": map })
}

/// Build a JSON array of table names from the given slice.
/// Used for JankenSQLHub `#[table]` with `enum` constraints.
pub fn build_table_enum(tables: &[&str]) -> Value {
    json!(tables)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_known_table() {
        let config = get("artist").unwrap();
        assert!(config.selectable.contains(&"name"));
        assert!(config.searchable.contains(&"name"));
        assert!(config.creatable.contains(&"name"));
        assert!(config.updatable.contains(&"status"));
    }

    #[test]
    fn test_get_unknown_table() {
        assert!(get("nonexistent").is_none());
    }

    #[test]
    fn test_all_tables_have_selectable() {
        for (name, config) in ALL_TABLES {
            assert!(
                !config.selectable.is_empty(),
                "Table {name} has no selectable fields"
            );
        }
    }

    #[test]
    fn test_build_selectable_enumif() {
        let enumif = build_selectable_enumif(&["artist", "song"]);
        let table_map = enumif["table"].as_object().unwrap();
        assert_eq!(table_map.len(), 2);
        assert!(table_map.contains_key("artist"));
        assert!(table_map.contains_key("song"));
        let artist_fields = table_map["artist"].as_array().unwrap();
        assert!(artist_fields.contains(&json!("name")));
    }

    #[test]
    fn test_build_table_enum() {
        let enum_val = build_table_enum(&["artist", "show"]);
        let arr = enum_val.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0], "artist");
        assert_eq!(arr[1], "show");
    }

    #[test]
    fn test_rel_show_song_has_no_updatable() {
        let config = get("rel_show_song").unwrap();
        assert!(config.updatable.is_empty());
    }

    #[test]
    fn test_learning_searchable_fields() {
        let config = get("learning").unwrap();
        assert!(config.searchable.contains(&"song_id"));
        assert!(config.searchable.contains(&"level"));
        assert!(config.searchable.contains(&"graduated"));
    }
}
