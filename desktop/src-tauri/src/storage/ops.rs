use rusqlite::{Connection, Result as SqlResult, params};

pub fn outbox_insert(
    conn: &Connection, entity_type: &str, entity_id: &str,
    action: &str, payload: &str,
) -> SqlResult<usize> {
    let now = chrono::Utc::now().timestamp_millis();
    conn.execute(
        "INSERT INTO sync_outbox (entity_type, entity_id, action, payload_json, status, created_at)
         VALUES (?1, ?2, ?3, ?4, 'pending', ?5)",
        params![entity_type, entity_id, action, payload, now],
    )
}

pub fn outbox_list_pending(conn: &Connection, limit: i64) -> SqlResult<Vec<(i64, String, String, String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT id, entity_type, entity_id, action, payload_json
         FROM sync_outbox WHERE status = 'pending' ORDER BY created_at ASC LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
    })?;
    rows.collect()
}

pub fn outbox_mark_done(conn: &Connection, id: i64) -> SqlResult<usize> {
    conn.execute("UPDATE sync_outbox SET status = 'done' WHERE id = ?1", params![id])
}

pub fn outbox_mark_failed(conn: &Connection, id: i64) -> SqlResult<usize> {
    conn.execute(
        "UPDATE sync_outbox SET status = 'failed', retry_count = retry_count + 1 WHERE id = ?1",
        params![id],
    )
}

/// Get schema version from DB. Returns 0 if no version has been applied.
pub fn schema_version(conn: &Connection) -> SqlResult<i32> {
    conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_version",
        [],
        |row| row.get(0),
    )
}
