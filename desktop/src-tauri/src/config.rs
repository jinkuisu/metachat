pub struct AppConfig {
    pub db_path: std::path::PathBuf,
    pub cloak_dir: std::path::PathBuf,
    pub log_level: String,
    pub translation_cache_ttl_secs: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        let app_dir = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("metachat");

        Self {
            db_path: app_dir.join("metachat.db"),
            cloak_dir: app_dir.join("cloak"),
            log_level: "info".to_string(),
            translation_cache_ttl_secs: 86400,
        }
    }
}

impl AppConfig {
    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        if let Some(parent) = self.db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::create_dir_all(&self.cloak_dir)?;
        Ok(())
    }
}
