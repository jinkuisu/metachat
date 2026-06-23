use rusqlite::{Connection, Result as SqlResult, params};
use crate::storage::models::Contact;

/// Insert a contact. remark_json and labels_json are stored as JSON strings.
pub fn insert(conn: &Connection, c: &Contact) -> SqlResult<usize> {
    conn.execute(
        "INSERT INTO contacts (id, platform, platform_contact_id, account_id, name,
         avatar_url, language, remark_json, labels_json, added_at, last_active_at, sync_version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            c.id, c.platform, c.platform_contact_id, c.account_id, c.name,
            c.avatar_url, c.language,
            serde_json::to_string(&c.remarks).unwrap_or_default(),
            serde_json::to_string(&c.labels).unwrap_or_default(),
            c.added_at, c.last_active_at, c.sync_version,
        ],
    )
}

/// List all non-deleted contacts for an account, ordered by last_active_at.
pub fn list(conn: &Connection, account_id: &str) -> SqlResult<Vec<Contact>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM contacts WHERE account_id = ?1 AND is_deleted = 0
         ORDER BY last_active_at DESC",
    )?;
    let rows = stmt.query_map(params![account_id], |row| decode(row))?;
    rows.collect()
}

pub fn update(conn: &Connection, c: &Contact) -> SqlResult<usize> {
    conn.execute(
        "UPDATE contacts SET name=?1, avatar_url=?2, language=?3, remark_json=?4,
         labels_json=?5, last_active_at=?6, sync_version=?7 WHERE id=?8",
        params![
            c.name, c.avatar_url, c.language,
            serde_json::to_string(&c.remarks).unwrap_or_default(),
            serde_json::to_string(&c.labels).unwrap_or_default(),
            c.last_active_at, c.sync_version, c.id,
        ],
    )
}

/// Soft-delete a contact.
pub fn delete(conn: &Connection, id: &str) -> SqlResult<usize> {
    conn.execute("UPDATE contacts SET is_deleted = 1 WHERE id = ?1", params![id])
}

/// Search contacts by name or remark content.
pub fn search(conn: &Connection, account_id: &str, query: &str) -> SqlResult<Vec<Contact>> {
    let p = format!("%{}%", query);
    let mut stmt = conn.prepare(
        "SELECT * FROM contacts WHERE account_id = ?1 AND is_deleted = 0
         AND (name LIKE ?2 OR remark_json LIKE ?2) ORDER BY last_active_at DESC",
    )?;
    let rows = stmt.query_map(params![account_id, p], |row| decode(row))?;
    rows.collect()
}

/// List contacts updated after a given sync_version (for cloud sync).
pub fn list_sync(conn: &Connection, account_id: &str, since: i64) -> SqlResult<Vec<Contact>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM contacts WHERE account_id = ?1 AND sync_version > ?2
         ORDER BY sync_version ASC",
    )?;
    let rows = stmt.query_map(params![account_id, since], |row| decode(row))?;
    rows.collect()
}

fn decode(row: &rusqlite::Row) -> rusqlite::Result<Contact> {
    Ok(Contact {
        id: row.get(0)?, platform: row.get(1)?, platform_contact_id: row.get(2)?,
        account_id: row.get(3)?, name: row.get(4)?, avatar_url: row.get(5)?,
        language: row.get(6)?,
        remarks: serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_default(),
        labels: serde_json::from_str(&row.get::<_, String>(8)?).unwrap_or_default(),
        added_at: row.get(9)?, last_active_at: row.get(10)?,
        is_deleted: row.get::<_, i32>(11)? != 0, sync_version: row.get(12)?,
    })
}
