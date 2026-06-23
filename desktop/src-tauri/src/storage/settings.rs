use rusqlite::{Connection, Result as SqlResult, params};
use crate::storage::models::Setting;

pub fn setting_get(conn: &Connection, key: &str) -> SqlResult<Option<serde_json::Value>> {
    let mut stmt = conn.prepare("SELECT value_json FROM settings WHERE key = ?1")?;
    let mut rows = stmt.query(params![key])?;
    match rows.next()? {
        Some(row) => {
            let val: String = row.get(0)?;
            Ok(serde_json::from_str(&val).ok())
        }
        None => Ok(None),
    }
}

pub fn setting_set(conn: &Connection, key: &str, value: &serde_json::Value, sync: bool) -> SqlResult<usize> {
    let now = chrono::Utc::now().timestamp_millis();
    conn.execute(
        "INSERT INTO settings (key, value_json, updated_at, sync_to_cloud)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(key) DO UPDATE SET value_json=?2, updated_at=?3, sync_to_cloud=?4",
        params![key, value.to_string(), now, sync as i32],
    )
}

pub fn setting_delete(conn: &Connection, key: &str) -> SqlResult<usize> {
    conn.execute("DELETE FROM settings WHERE key = ?1", params![key])
}

/// Translation cache: retrieve a non-expired entry.
pub fn cache_get(conn: &Connection, key: &str) -> SqlResult<Option<String>> {
    let now = chrono::Utc::now().timestamp_millis();
    conn.query_row(
        "SELECT result_json FROM translation_cache WHERE cache_key = ?1 AND expires_at > ?2",
        params![key, now], |row| row.get::<_, String>(0),
    ).map(Some).or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn cache_set(conn: &Connection, key: &str, result: &str, ttl_secs: u64) -> SqlResult<usize> {
    let expires = chrono::Utc::now().timestamp_millis() + (ttl_secs as i64 * 1000);
    conn.execute(
        "INSERT OR REPLACE INTO translation_cache (cache_key, result_json, expires_at)
         VALUES (?1, ?2, ?3)",
        params![key, result, expires],
    )
}

pub fn cache_clean(conn: &Connection) -> SqlResult<usize> {
    let now = chrono::Utc::now().timestamp_millis();
    conn.execute("DELETE FROM translation_cache WHERE expires_at <= ?1", params![now])
}
