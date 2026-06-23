/// Desktop SQLite schema - 15 tables\n
pub const CURRENT_SCHEMA_VERSION: i32 = 1;

pub const CREATE_TABLES: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS accounts (id TEXT PRIMARY KEY,platform TEXT NOT NULL,nickname TEXT NOT NULL,avatar_url TEXT,status TEXT DEFAULT 'offline',proxy_json TEXT,fingerprint_json TEXT NOT NULL,user_data_dir TEXT NOT NULL,created_at INTEGER NOT NULL,updated_at INTEGER NOT NULL);",
    "CREATE TABLE IF NOT EXISTS sessions (id TEXT PRIMARY KEY,platform TEXT NOT NULL,account_id TEXT NOT NULL,contact_id TEXT,group_id TEXT,name TEXT NOT NULL,avatar_url TEXT,last_message_preview TEXT,unread_count INTEGER DEFAULT 0,last_active_at INTEGER NOT NULL,created_at INTEGER NOT NULL,is_pinned INTEGER DEFAULT 0,sort_order INTEGER DEFAULT 0);",
    "CREATE TABLE IF NOT EXISTS messages (id TEXT PRIMARY KEY,platform TEXT NOT NULL,platform_message_id TEXT NOT NULL,account_id TEXT NOT NULL,session_id TEXT NOT NULL,sender_id TEXT NOT NULL,sender_name TEXT NOT NULL,content_json TEXT NOT NULL,timestamp INTEGER NOT NULL,direction TEXT NOT NULL,status TEXT DEFAULT 'sent',translated_json TEXT,is_deleted INTEGER DEFAULT 0);",
     "CREATE TABLE IF NOT EXISTS contacts (id TEXT PRIMARY KEY,platform TEXT NOT NULL,platform_contact_id TEXT NOT NULL,account_id TEXT NOT NULL,name TEXT NOT NULL,avatar_url TEXT,language TEXT,remark_json TEXT,labels_json TEXT,added_at INTEGER NOT NULL,last_active_at INTEGER NOT NULL,is_deleted INTEGER DEFAULT 0,sync_version INTEGER DEFAULT 0);",
    "CREATE TABLE IF NOT EXISTS labels (id TEXT PRIMARY KEY,name TEXT NOT NULL,color TEXT NOT NULL,account_id TEXT NOT NULL,created_at INTEGER NOT NULL);",
    "CREATE TABLE IF NOT EXISTS reply_groups (id TEXT PRIMARY KEY,name TEXT NOT NULL,account_id TEXT NOT NULL,sort_order INTEGER DEFAULT 0,created_at INTEGER NOT NULL);",
    "CREATE TABLE IF NOT EXISTS reply_templates (id TEXT PRIMARY KEY,group_id TEXT NOT NULL,title TEXT NOT NULL,content TEXT NOT NULL,template_type TEXT DEFAULT text,files_json TEXT,sort_order INTEGER DEFAULT 0,created_at INTEGER NOT NULL);",
    "CREATE TABLE IF NOT EXISTS reply_rules (id TEXT PRIMARY KEY,name TEXT NOT NULL,platform TEXT NOT NULL,account_id TEXT NOT NULL,trigger_type TEXT NOT NULL,trigger_keywords TEXT,reply_template_id TEXT NOT NULL,is_active INTEGER DEFAULT 1,time_range_json TEXT,frequency_limit INTEGER,created_at INTEGER NOT NULL);",
    "CREATE TABLE IF NOT EXISTS ai_conversations (id TEXT PRIMARY KEY,role TEXT NOT NULL,session_id TEXT NOT NULL,timestamp INTEGER NOT NULL);",
    "CREATE TABLE IF NOT EXISTS translation_cache (cache_key TEXT PRIMARY KEY,result_json TEXT NOT NULL,expires_at INTEGER NOT NULL);",
    "CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY,value_json TEXT NOT NULL,updated_at INTEGER NOT NULL);",
    "CREATE TABLE IF NOT EXISTS schema_version (version INTEGER PRIMARY KEY,applied_at INTEGER NOT NULL,description TEXT);",
];

pub const CREATE_INDEXES: &[&str] = &[
    "CREATE INDEX IF NOT EXISTS idx_accounts_platform ON accounts(platform);",
    "CREATE INDEX IF NOT EXISTS idx_sessions_account ON sessions(account_id, last_active_at);",
    "CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id, timestamp);",
    "CREATE INDEX IF NOT EXISTS idx_contacts_account ON contacts(account_id);",
];

pub fn get_create_tables_sql() -> &'static [&'static str] {
    &CREATE_TABLES
}

pub fn get_create_indexes_sql() -> &'static [&'static str] {
    &CREATE_INDEXES
}
