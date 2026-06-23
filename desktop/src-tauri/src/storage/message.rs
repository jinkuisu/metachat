use rusqlite::{Connection, Result as SqlResult, params};
use crate::storage::models::Message;

/// Insert a message. content_json is serialized by the caller.
pub fn insert(conn: &Connection, m: &Message) -> SqlResult<usize> {
    conn.execute(
        "INSERT INTO messages (id, platform, platform_message_id, account_id, session_id,
         sender_id, sender_name, content_json, timestamp, direction, status, translated_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            m.id, m.platform, m.platform_message_id, m.account_id, m.session_id,
            m.sender_id, m.sender_name, m.content_json, m.timestamp,
            m.direction, m.status, m.translated_json,
        ],
    )
}

/// List messages in a session, newest first, with pagination.
pub fn list(conn: &Connection, session_id: &str, limit: i64, offset: i64) -> SqlResult<Vec<Message>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM messages WHERE session_id = ?1 AND is_deleted = 0
         ORDER BY timestamp DESC LIMIT ?2 OFFSET ?3",
    )?;
    let rows = stmt.query_map(params![session_id, limit, offset], |row| decode(row))?;
    rows.collect()
}

/// Full-text search across all messages for an account.
pub fn search(conn: &Connection, account_id: &str, query: &str, limit: i64) -> SqlResult<Vec<Message>> {
    let pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
        "SELECT * FROM messages WHERE account_id = ?1 AND content_json LIKE ?2
         AND is_deleted = 0 ORDER BY timestamp DESC LIMIT ?3",
    )?;
    let rows = stmt.query_map(params![account_id, pattern, limit], |row| decode(row))?;
    rows.collect()
}

/// Soft-delete a message (marks is_deleted = 1).
pub fn delete(conn: &Connection, id: &str) -> SqlResult<usize> {
    conn.execute("UPDATE messages SET is_deleted = 1 WHERE id = ?1", params![id])
}

/// Get message count for a session (for unread badge calculation).
pub fn count(conn: &Connection, session_id: &str) -> SqlResult<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM messages WHERE session_id = ?1 AND is_deleted = 0",
        params![session_id], |row| row.get(0),
    )
}

fn decode(row: &rusqlite::Row) -> rusqlite::Result<Message> {
    Ok(Message {
        id: row.get(0)?, platform: row.get(1)?, platform_message_id: row.get(2)?,
        account_id: row.get(3)?, session_id: row.get(4)?, sender_id: row.get(5)?,
        sender_name: row.get(6)?, content_json: row.get(7)?, timestamp: row.get(8)?,
        direction: row.get(9)?, status: row.get(10)?,
        translated_json: row.get(11)?,
        is_deleted: row.get::<_, i32>(12)? != 0,
    })
}
