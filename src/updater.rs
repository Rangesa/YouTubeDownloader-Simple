use std::process::Command;

use crate::error::{Result, YtdlError};

/// yt-dlp更新機能
pub struct Updater;

impl Updater {
    /// yt-dlpを最新版に更新
    pub fn update_ytdlp() -> Result<()> {
        // pip経由でインストールされている場合はpip upgradeを試す
        let pip_update = Command::new("pip")
            .args(&["install", "--upgrade", "yt-dlp"])
            .output();

        if let Ok(output) = pip_update {
            if output.status.success() {
                println!("✅ yt-dlpを最新版に更新しました（pip経由）");
                return Ok(());
            }
        }

        // pip更新が失敗した場合は--updateを試す
        let ytdlp_update = Command::new("yt-dlp")
            .arg("--update")
            .output();

        if let Ok(output) = ytdlp_update {
            if output.status.success() {
                println!("✅ yt-dlpを最新版に更新しました");
                return Ok(());
            }
        }

        // どちらも失敗した場合は警告のみ
        eprintln!("⚠️ yt-dlpの自動更新をスキップしました（手動更新が必要な場合があります）");
        Ok(())
    }

    /// yt-dlpのバージョンを表示
    #[allow(dead_code)]
    pub fn show_version() -> Result<String> {
        let output = Command::new("yt-dlp")
            .arg("--version")
            .output()
            .map_err(|_| YtdlError::YtdlpNotFound)?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            Ok(version.trim().to_string())
        } else {
            Err(YtdlError::YtdlpNotFound)
        }
    }
}
