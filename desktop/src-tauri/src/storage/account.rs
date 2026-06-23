/// Account database operations.
///
/// All functions take a usqlite::Connection reference as the first argument.
/// Returns usqlite::Result for integration with Tauri command handlers.

use rusqlite::{Connection, Result as SqlResult, params};
use crate::storage::models::Account;

/// Insert a new social media account into the database.
/// All config fields (proxy, fingerprint) are stored as JSON strings.
pub fn insert(conn: &Connection, a: &Account) -> SqlResult<usize> {
    conn.execute(
        "INSERT INTO accounts (id, platform, nickname, avatar_url, status, proxy_json,
         fingerprint_json, user_data_dir, cookie_encrypted, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            a.id, a.platform.to_string(), a.nickname, a.avatar_url, a.status,
            serde_json::to_string(&a.proxy).ok(),
            a.fingerprint, a.user_data_dir, a.cookie_encrypted,
            a.sort_order, a.created_at, a.updated_at,
        ],
    )
}

/// Fetch a single account by its UUID primary key.
/// Returns one if no account matches the given id.
pub fn get(conn: &Connection, id: &str) -> SqlResult<Option<Account>> {
    let mut stmt = conn.prepare(
        "SELECT id, platform, nickname, avatar_url, status, proxy_json,
         fingerprint_json, user_data_dir, cookie_encrypted, sort_order, created_at, updated_at
         FROM accounts WHERE id = ?1",
    )?;
    let mut rows = stmt.query_map(params![id], |row| read_account(row))?;
    Ok(rows.next().transpose()?)
}

/// List all accounts ordered by sort_order ascending, then created_at descending.
pub fn list(conn: &Connection) -> SqlResult<Vec<Account>> {
    let mut stmt = conn.prepare(
        "SELECT id, platform, nickname, avatar_url, status, proxy_json,
         fingerprint_json, user_data_dir, cookie_encrypted, sort_order, created_at, updated_at
         FROM accounts ORDER BY sort_order ASC, created_at DESC",
    )?;
    let rows = stmt.query_map([], |row| read_account(row))?;
    rows.collect()
}

/// Update an account's mutable fields: nickname, avatar, proxy, fingerprint, sort_order.
/// Timestamp is explicitly set by the caller.
pub fn update(conn: &Connection, a: &Account) -> SqlResult<usize> {
    conn.execute(
        "UPDATE accounts SET nickname=?1, avatar_url=?2, status=?3, proxy_json=?4,
         fingerprint_json=?5, sort_order=?6, updated_at=?7 WHERE id=?8",
        params![
            a.nickname, a.avatar_url, a.status,
            serde_json::to_string(&a.proxy).ok(),
            a.fingerprint, a.sort_order, a.updated_at, a.id,
        ],
    )
}

/// Permanently delete an account and its associated sessions and contacts.
pub fn delete(conn: &Connection, id: &str) -> SqlResult<usize> {
    conn.execute("DELETE FROM accounts WHERE id = ?1", params![id])
}

/// Update just the status field of an account (used for online/offline tracking).
pub fn update_status(conn: &Connection, id: &str, status: &str) -> SqlResult<usize> {
    let now = chrono::Utc::now().timestamp_millis();
    conn.execute(
        "UPDATE accounts SET status=?1, updated_at=?2 WHERE id=?3",
        params![status, now, id],
    )
}

// Private helper: convert a DB row to an Account struct.
fn read_account(row: &rusqlite::Row) -> rusqlite::Result<Account> {
    Ok(Account {
        id: row.get(0)?,
        platform: row.get::<_, String>(1)?
            .parse().unwrap_or(crate::storage::models::Platform::Custom("unknown".into())),
        nickname: row.get(2)?,
        avatar_url: row.get(3)?,
        status: row.get(4)?,
        proxy: row.get::<_, Option<String>>(5)?
            .and_then(|s| serde_json::from_str(&s).ok()),
        fingerprint: row.get(6)?,
        user_data_dir: row.get(7)?,
        cookie_encrypted: row.get(8)?,
        sort_order: row.get(9)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}
