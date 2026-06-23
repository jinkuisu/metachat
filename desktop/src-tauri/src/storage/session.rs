use rusqlite::{Connection, Result as SqlResult, params};
use crate::storage::models::Session;

/// Insert or replace a session record.
pub fn insert(conn: &Connection, s: &Session) -> SqlResult<usize> {
    conn.execute(
        "INSERT INTO sessions (id, platform, account_id, contact_id, group_id, name,
         avatar_url, last_message_preview, unread_count, last_active_at, created_at,
         is_pinned, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            s.id, s.platform, s.account_id, s.contact_id, s.group_id,
            s.name, s.avatar_url, s.last_message_preview, s.unread_count,
            s.last_active_at, s.created_at, s.is_pinned as i32, s.sort_order,
        ],
    )
}

pub fn get(conn: &Connection, id: &str) -> SqlResult<Option<Session>> {
    let mut stmt = conn.prepare(
        "SELECT id, platform, account_id, contact_id, group_id, name, avatar_url,
         last_message_preview, unread_count, last_active_at, created_at,
         is_pinned, sort_order FROM sessions WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map(params![id], |row| decode(row))?;
    Ok(rows.next().transpose()?)
}

/// List sessions for an account, pinned first, then by last_active_at descending.
pub fn list(conn: &Connection, account_id: &str, limit: i64, offset: i64) -> SqlResult<Vec<Session>> {
    let mut stmt = conn.prepare(
        "SELECT id, platform, account_id, contact_id, group_id, name, avatar_url,
         last_message_preview, unread_count, last_active_at, created_at,
         is_pinned, sort_order FROM sessions WHERE account_id = ?1
         ORDER BY is_pinned DESC, last_active_at DESC LIMIT ?2 OFFSET ?3",
    )?;
    let rows = stmt.query_map(params![account_id, limit, offset], |row| decode(row))?;
    rows.collect()
}

pub fn update(conn: &Connection, s: &Session) -> SqlResult<usize> {
    conn.execute(
        "UPDATE sessions SET name=?1, avatar_url=?2, last_message_preview=?3,
         unread_count=?4, is_pinned=?5, last_active_at=?6 WHERE id=?7",
        params![s.name, s.avatar_url, s.last_message_preview, s.unread_count,
                s.is_pinned as i32, s.last_active_at, s.id],
    )
}

pub fn delete(conn: &Connection, id: &str) -> SqlResult<usize> {
    conn.execute("DELETE FROM sessions WHERE id = ?1", params![id])
}

fn decode(row: &rusqlite::Row) -> rusqlite::Result<Session> {
    Ok(Session {
        id: row.get(0)?, platform: row.get(1)?, account_id: row.get(2)?,
        contact_id: row.get(3)?, group_id: row.get(4)?, name: row.get(5)?,
        avatar_url: row.get(6)?, last_message_preview: row.get(7)?,
        unread_count: row.get(8)?, last_active_at: row.get(9)?, created_at: row.get(10)?,
        is_pinned: row.get::<_, i32>(11)? != 0, sort_order: row.get(12)?,
    })
}
