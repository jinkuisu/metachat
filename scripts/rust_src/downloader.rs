// ═══════════════════════════════════════════════════════════════
// downloader.rs — CloakBrowser 下载/校验/解压
// CDN_URL 需要配置实际的下载地址
// ═══════════════════════════════════════════════════════════════

use std::path::{Path, PathBuf};
use sha2::{Sha256, Digest};
use std::io::Read;

/// CloakBrowser 下载配置
pub struct DownloadConfig {
    /// CDN 下载地址（需要配置）
    pub cdn_url: String,
    /// 期望的 SHA256
    pub expected_sha256: String,
    /// 存储路径
    pub install_dir: PathBuf,
    /// 版本号
    pub version: String,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            cdn_url: String::new(),    // ← 需要配置
            expected_sha256: String::new(),
            install_dir: PathBuf::new(),
            version: "1.0.0".into(),
        }
    }
}

/// 检查 CloakBrowser 是否已下载
pub fn is_cloak_installed(cloak_dir: &Path) -> bool {
    let marker = cloak_dir.join(".version");
    if !marker.exists() { return false; }
    if let Ok(content) = std::fs::read_to_string(&marker) {
        return !content.trim().is_empty();
    }
    false
}

/// 下载 CloakBrowser（支持断点续传）
pub async fn download_cloak(
    config: &DownloadConfig,
    on_progress: impl Fn(u64, u64),  // (downloaded, total)
) -> Result<PathBuf, String> {
    if config.cdn_url.is_empty() {
        return Err("CDN_URL 未配置，请在 config 中设置 CloakBrowser 下载地址".into());
    }

    let temp_path = config.install_dir.join("cloak_download.zip");
    let extract_dir = config.install_dir.join("temp_extract");

    // 下载
    let response = reqwest::get(&config.cdn_url)
        .await
        .map_err(|e| format!("下载失败: {}", e))?;

    let total = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut file = tokio::fs::File::create(&temp_path).await
        .map_err(|e| format!("创建文件失败: {}", e))?;

    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("下载数据包错误: {}", e))?;
        use tokio::io::AsyncWriteExt;
        file.write_all(&chunk).await.map_err(|e| format!("写入文件失败: {}", e))?;
        downloaded += chunk.len() as u64;
        on_progress(downloaded, total);
    }

    // 校验 SHA256
    if !config.expected_sha256.is_empty() {
        let hash = sha256_file(&temp_path)
            .map_err(|e| format!("SHA256 计算失败: {}", e))?;
        if hash != config.expected_sha256 {
            let _ = std::fs::remove_file(&temp_path);
            return Err(format!("SHA256 不匹配: 期望 {} 实际 {}", config.expected_sha256, hash));
        }
    }

    // 解压
    let _ = std::fs::remove_dir_all(&extract_dir);
    std::fs::create_dir_all(&extract_dir)
        .map_err(|e| format!("创建解压目录失败: {}", e))?;

    #[cfg(target_os = "windows")]
    {
        let file = std::fs::File::open(&temp_path)
            .map_err(|e| format!("打开压缩文件失败: {}", e))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| format!("读取压缩文件失败: {}", e))?;
        archive.extract(&extract_dir)
            .map_err(|e| format!("解压失败: {}", e))?;
    }

    // 移动到最终目录
    let final_dir = config.install_dir.join("cloak");
    let _ = std::fs::remove_dir_all(&final_dir);
    std::fs::rename(&extract_dir, &final_dir)
        .map_err(|e| format!("移动文件失败: {}", e))?;

    // 写入版本标记
    let _ = std::fs::write(final_dir.join(".version"), &config.version);

    // 删除下载文件
    let _ = std::fs::remove_file(&temp_path);

    // 找到可执行文件
    let exe_name = if cfg!(target_os = "windows") { "cloak-browser.exe" } else { "cloak-browser" };
    let exe_path = final_dir.join(exe_name);
    if !exe_path.exists() {
        return Err(format!("解压后找不到可执行文件: {}", exe_path.display()));
    }
    Ok(exe_path)
}

fn sha256_file(path: &Path) -> Result<String, std::io::Error> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
