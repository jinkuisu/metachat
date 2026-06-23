use rusqlite::{Connection, Result as SqlResult, params};
use crate::storage::models::ReplyTemplate;

pub fn template_insert(conn: &Connection, t: &ReplyTemplate) -> SqlResult<usize> {
    conn.execute(
        "INSERT INTO reply_templates (id, group_id, title, content, template_type, files_json, sort_order, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            t.id, t.group_id, t.title, t.content, t.template_type,
            serde_json::to_string(&t.files).unwrap_or_default(),
            t.sort_order, t.created_at,
        ],
    )
}

pub fn template_list(conn: &Connection, group_id: &str) -> SqlResult<Vec<ReplyTemplate>> {
    let mut stmt = conn.prepare(
        "SELECT * FROM reply_templates WHERE group_id = ?1 ORDER BY sort_order ASC",
    )?;
    let rows = stmt.query_map(params![group_id], |row| decode_template(row))?;
    rows.collect()
}

pub fn template_delete(conn: &Connection, id: &str) -> SqlResult<usize> {
    conn.execute("DELETE FROM reply_templates WHERE id = ?1", params![id])
}

fn decode_template(row: &rusqlite::Row) -> rusqlite::Result<ReplyTemplate> {
    Ok(ReplyTemplate {
        id: row.get(0)?, group_id: row.get(1)?, title: row.get(2)?,
        content: row.get(3)?, template_type: row.get(4)?,
        files: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
        sort_order: row.get(6)?, created_at: row.get(7)?,
    })
}
