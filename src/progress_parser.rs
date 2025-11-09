use regex::Regex;
use std::sync::LazyLock;

use crate::error::{Result, YtdlError};

/// yt-dlpの進捗情報
#[derive(Debug, Clone)]
pub struct ProgressInfo {
    /// 進捗率（0.0 ~ 100.0）
    pub percent: f64,
    /// ダウンロード済みサイズ（バイト）
    pub downloaded_bytes: Option<u64>,
    /// 総サイズ（バイト）
    pub total_bytes: Option<u64>,
    /// ダウンロード速度（バイト/秒）
    pub speed: Option<f64>,
    /// 残り時間（秒）
    pub eta: Option<u64>,
}

impl ProgressInfo {
    /// ダウンロード済みサイズを人間が読める形式で取得
    pub fn downloaded_size_str(&self) -> String {
        self.downloaded_bytes
            .map(format_bytes)
            .unwrap_or_else(|| "不明".to_string())
    }

    /// 総サイズを人間が読める形式で取得
    pub fn total_size_str(&self) -> String {
        self.total_bytes
            .map(format_bytes)
            .unwrap_or_else(|| "不明".to_string())
    }

    /// ダウンロード速度を人間が読める形式で取得
    pub fn speed_str(&self) -> String {
        self.speed
            .map(|s| format!("{}/s", format_bytes(s as u64)))
            .unwrap_or_else(|| "不明".to_string())
    }

    /// 残り時間を人間が読める形式で取得
    pub fn eta_str(&self) -> String {
        self.eta
            .map(format_duration)
            .unwrap_or_else(|| "不明".to_string())
    }
}

/// yt-dlpの出力から進捗情報をパース
pub struct ProgressParser {
    // yt-dlpの進捗出力パターン
    // 例: [download]  45.2% of 123.45MiB at 1.23MiB/s ETA 00:42
    download_regex: LazyLock<Regex>,
}

impl Default for ProgressParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressParser {
    pub fn new() -> Self {
        Self {
            download_regex: LazyLock::new(|| {
                Regex::new(
                    r"\[download\]\s+(?P<percent>[\d.]+)%\s+of\s+(?P<total>[\d.]+)(?P<total_unit>[KMG]iB)(?:\s+at\s+(?P<speed>[\d.]+)(?P<speed_unit>[KMG]iB)/s)?(?:\s+ETA\s+(?P<eta>\d+:\d+))?"
                ).expect("正規表現のコンパイルに失敗")
            }),
        }
    }

    /// yt-dlpの出力行をパースして進捗情報を抽出
    pub fn parse(&self, line: &str) -> Result<Option<ProgressInfo>> {
        // [download]で始まる行のみ処理
        if !line.contains("[download]") {
            return Ok(None);
        }

        // 進捗率のみの行（簡易版）をチェック
        // 例: [download] 45.2% of ~123.45MiB at 1.23MiB/s ETA 00:42
        if let Some(caps) = self.download_regex.captures(line) {
            let percent = caps
                .name("percent")
                .and_then(|m| m.as_str().parse::<f64>().ok())
                .ok_or_else(|| {
                    YtdlError::ProgressParseError("進捗率のパースに失敗".to_string())
                })?;

            let total_bytes = caps
                .name("total")
                .and_then(|m| m.as_str().parse::<f64>().ok())
                .and_then(|val| {
                    caps.name("total_unit")
                        .map(|unit| parse_size(val, unit.as_str()))
                });

            let downloaded_bytes = total_bytes.map(|total| ((total as f64) * percent / 100.0) as u64);

            let speed = caps
                .name("speed")
                .and_then(|m| m.as_str().parse::<f64>().ok())
                .and_then(|val| {
                    caps.name("speed_unit")
                        .map(|unit| parse_size(val, unit.as_str()) as f64)
                });

            let eta = caps
                .name("eta")
                .and_then(|m| parse_time_str(m.as_str()));

            return Ok(Some(ProgressInfo {
                percent,
                downloaded_bytes,
                total_bytes,
                speed,
                eta,
            }));
        }

        Ok(None)
    }
}

/// サイズ文字列をバイト数にパース（例: "123.45", "MiB" -> バイト数）
fn parse_size(value: f64, unit: &str) -> u64 {
    let multiplier = match unit {
        "KiB" => 1024.0,
        "MiB" => 1024.0 * 1024.0,
        "GiB" => 1024.0 * 1024.0 * 1024.0,
        _ => 1.0,
    };
    (value * multiplier) as u64
}

/// 時間文字列をパース（例: "01:23" -> 83秒）
fn parse_time_str(time_str: &str) -> Option<u64> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() == 2 {
        let minutes = parts[0].parse::<u64>().ok()?;
        let seconds = parts[1].parse::<u64>().ok()?;
        Some(minutes * 60 + seconds)
    } else {
        None
    }
}

/// バイト数を人間が読める形式にフォーマット
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

/// 秒数を人間が読める形式にフォーマット
fn format_duration(seconds: u64) -> String {
    let minutes = seconds / 60;
    let secs = seconds % 60;

    if minutes > 0 {
        format!("{:02}:{:02}", minutes, secs)
    } else {
        format!("00:{:02}", secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_progress() {
        let parser = ProgressParser::new();

        let line = "[download]  45.2% of 123.45MiB at 1.23MiB/s ETA 00:42";
        let result = parser.parse(line).unwrap();
        assert!(result.is_some());

        let info = result.unwrap();
        assert_eq!(info.percent, 45.2);
        assert!(info.total_bytes.is_some());
        assert!(info.speed.is_some());
        assert!(info.eta.is_some());
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512.00 B");
        assert_eq!(format_bytes(1024), "1.00 KiB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MiB");
        assert_eq!(format_bytes(1536 * 1024 * 1024), "1.50 GiB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "00:30");
        assert_eq!(format_duration(90), "01:30");
        assert_eq!(format_duration(3661), "61:01");
    }

    #[test]
    fn test_parse_time_str() {
        assert_eq!(parse_time_str("01:30"), Some(90));
        assert_eq!(parse_time_str("00:42"), Some(42));
        assert_eq!(parse_time_str("invalid"), None);
    }
}
