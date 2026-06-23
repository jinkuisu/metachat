pub mod migrate;
pub mod schema;
pub mod models;
pub mod account;
pub mod session;
pub mod message;
pub mod contact;
pub mod reply;
pub mod settings;
pub mod ops;

use rusqlite::{Connection, Result as SqlResult};
use std::path::Path;

/// Open or create the database at the given path, apply the current schema.
pub fn open(path: &Path) -> SqlResult<Connection> {
    let mut conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    let tx = conn.transaction()?;
    for sql in schema::CREATE_TABLES {
        tx.execute_batch(sql)?;
    }
    for sql in schema::CREATE_INDEXES {
        tx.execute_batch(sql)?;
    }
    tx.commit()?;
    Ok(conn)
}
