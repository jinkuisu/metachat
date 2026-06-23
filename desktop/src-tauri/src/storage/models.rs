use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ===== Account types =====
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    WhatsApp, Facebook, Instagram, Telegram, Line,
    Messenger, TikTok, X, Zalo, Tgkcn,
    Custom(String),
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::WhatsApp => write!(f, "whatsapp"),
            Self::Facebook => write!(f, "facebook"),
            Self::Instagram => write!(f, "instagram"),
            Self::Telegram => write!(f, "telegram"),
            Self::Line => write!(f, "line"),
            Self::Messenger => write!(f, "messenger"),
            Self::TikTok => write!(f, "tiktok"),
            Self::X => write!(f, "x"),
            Self::Zalo => write!(f, "zalo"),
            Self::Tgkcn => write!(f, "tgkcn"),
            Self::Custom(s) => write!(f, "custom_{}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub proxy_type: String,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub platform: Platform,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub status: String,
    pub proxy: Option<ProxyConfig>,
    pub fingerprint: String,
    pub user_data_dir: String,
    pub cookie_encrypted: Option<Vec<u8>>,
    pub sort_order: i32,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub platform: String,
    pub account_id: String,
    pub contact_id: Option<String>,
    pub group_id: Option<String>,
    pub name: String,
    pub avatar_url: Option<String>,
    pub last_message_preview: Option<String>,
    pub unread_count: u32,
    pub last_active_at: i64,
    pub created_at: i64,
    pub is_pinned: bool,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub platform: String,
    pub platform_message_id: String,
    pub account_id: String,
    pub session_id: String,
    pub sender_id: String,
    pub sender_name: String,
    pub content_json: String,
    pub timestamp: i64,
    pub direction: String,
    pub status: String,
    pub translated_json: Option<String>,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: String,
    pub platform: String,
    pub platform_contact_id: String,
    pub account_id: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub language: Option<String>,
    pub remarks: HashMap<String, String>,
    pub labels: Vec<String>,
    pub added_at: i64,
    pub last_active_at: i64,
    pub is_deleted: bool,
    pub sync_version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub id: String, pub name: String, pub color: String,
    pub account_id: String, pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyTemplate {
    pub id: String, pub group_id: String, pub title: String,
    pub content: String, pub template_type: String,
    pub files: Vec<serde_json::Value>,
    pub sort_order: i32, pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: serde_json::Value,
    pub updated_at: i64,
    pub sync_to_cloud: bool,
}

use std::str::FromStr;

impl FromStr for Platform {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "whatsapp" => Platform::WhatsApp,
            "facebook" => Platform::Facebook,
            "instagram" => Platform::Instagram,
            "telegram" => Platform::Telegram,
            "line" => Platform::Line,
            "messenger" => Platform::Messenger,
            "tiktok" => Platform::TikTok,
            "x" => Platform::X,
            "zalo" => Platform::Zalo,
            "tgkcn" => Platform::Tgkcn,
            other => Platform::Custom(other.to_string()),
        })
    }
}
