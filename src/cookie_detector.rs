use std::env;
use std::path::PathBuf;

use crate::error::{Result, YtdlError};

/// サポートされているブラウザ
#[derive(Debug, Clone)]
pub enum Browser {
    Chrome,
    Firefox,
    Edge,
    Brave,
    Opera,
}

impl Browser {
    /// 文字列からブラウザを解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "chrome" => Some(Browser::Chrome),
            "firefox" => Some(Browser::Firefox),
            "edge" => Some(Browser::Edge),
            "brave" => Some(Browser::Brave),
            "opera" => Some(Browser::Opera),
            _ => None,
        }
    }

    /// ブラウザ名を取得
    pub fn name(&self) -> &str {
        match self {
            Browser::Chrome => "chrome",
            Browser::Firefox => "firefox",
            Browser::Edge => "edge",
            Browser::Brave => "brave",
            Browser::Opera => "opera",
        }
    }
}

/// Cookie検出器
pub struct CookieDetector {
    browser: Browser,
}

impl CookieDetector {
    /// 新しいCookie検出器を作成
    pub fn new(browser: Browser) -> Self {
        Self { browser }
    }

    /// 文字列からCookie検出器を作成
    pub fn from_str(browser_name: &str) -> Result<Self> {
        let browser = Browser::from_str(browser_name).ok_or_else(|| {
            YtdlError::CookieDetection(format!(
                "サポートされていないブラウザ: {}",
                browser_name
            ))
        })?;
        Ok(Self::new(browser))
    }

    /// Cookieファイルのパスを検出
    ///
    /// 注意: このパスは実際にはyt-dlpが内部で処理するため、
    /// ここでは存在確認のみを行います。
    pub fn detect_cookie_path(&self) -> Result<Option<PathBuf>> {
        let path = self.get_browser_cookie_path()?;

        if path.exists() {
            Ok(Some(path))
        } else {
            // Cookieファイルが見つからない場合は警告
            eprintln!(
                "警告: {}のCookieファイルが見つかりません: {}",
                self.browser.name(),
                path.display()
            );
            eprintln!("公開動画のみダウンロード可能です。");
            Ok(None)
        }
    }

    /// ブラウザのCookieファイルパスを取得
    fn get_browser_cookie_path(&self) -> Result<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            self.get_windows_cookie_path()
        }

        #[cfg(target_os = "macos")]
        {
            self.get_macos_cookie_path()
        }

        #[cfg(target_os = "linux")]
        {
            self.get_linux_cookie_path()
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            Err(YtdlError::CookieDetection(
                "サポートされていないOSです".to_string(),
            ))
        }
    }

    #[cfg(target_os = "windows")]
    fn get_windows_cookie_path(&self) -> Result<PathBuf> {
        let local_appdata = env::var("LOCALAPPDATA").map_err(|_| {
            YtdlError::CookieDetection("LOCALAPPDATA環境変数が設定されていません".to_string())
        })?;

        let path = match self.browser {
            Browser::Chrome => {
                PathBuf::from(local_appdata).join(r"Google\Chrome\User Data\Default\Network\Cookies")
            }
            Browser::Firefox => {
                // FirefoxはプロファイルがランダムなのでAppData\Roamingから探す必要がある
                let appdata = env::var("APPDATA").map_err(|_| {
                    YtdlError::CookieDetection("APPDATA環境変数が設定されていません".to_string())
                })?;
                PathBuf::from(appdata).join(r"Mozilla\Firefox\Profiles")
            }
            Browser::Edge => {
                PathBuf::from(local_appdata).join(r"Microsoft\Edge\User Data\Default\Network\Cookies")
            }
            Browser::Brave => {
                PathBuf::from(local_appdata).join(r"BraveSoftware\Brave-Browser\User Data\Default\Network\Cookies")
            }
            Browser::Opera => {
                PathBuf::from(local_appdata.replace("Local", "Roaming"))
                    .join(r"Opera Software\Opera Stable\Network\Cookies")
            }
        };

        Ok(path)
    }

    #[cfg(target_os = "macos")]
    fn get_macos_cookie_path(&self) -> Result<PathBuf> {
        let home = env::var("HOME")
            .map_err(|_| YtdlError::CookieDetection("HOME環境変数が設定されていません".to_string()))?;

        let path = match self.browser {
            Browser::Chrome => PathBuf::from(home)
                .join("Library/Application Support/Google/Chrome/Default/Cookies"),
            Browser::Firefox => {
                PathBuf::from(home).join("Library/Application Support/Firefox/Profiles")
            }
            Browser::Edge => PathBuf::from(home)
                .join("Library/Application Support/Microsoft Edge/Default/Cookies"),
            Browser::Brave => PathBuf::from(home)
                .join("Library/Application Support/BraveSoftware/Brave-Browser/Default/Cookies"),
            Browser::Opera => PathBuf::from(home)
                .join("Library/Application Support/com.operasoftware.Opera/Cookies"),
        };

        Ok(path)
    }

    #[cfg(target_os = "linux")]
    fn get_linux_cookie_path(&self) -> Result<PathBuf> {
        let home = env::var("HOME")
            .map_err(|_| YtdlError::CookieDetection("HOME環境変数が設定されていません".to_string()))?;

        let path = match self.browser {
            Browser::Chrome => PathBuf::from(home).join(".config/google-chrome/Default/Cookies"),
            Browser::Firefox => PathBuf::from(home).join(".mozilla/firefox"),
            Browser::Edge => PathBuf::from(home).join(".config/microsoft-edge/Default/Cookies"),
            Browser::Brave => {
                PathBuf::from(home).join(".config/BraveSoftware/Brave-Browser/Default/Cookies")
            }
            Browser::Opera => PathBuf::from(home).join(".config/opera/Cookies"),
        };

        Ok(path)
    }

    /// yt-dlp用のブラウザ指定文字列を取得
    ///
    /// yt-dlpは `--cookies-from-browser chrome` のような形式でブラウザを指定します。
    /// これにより、yt-dlpが自動的にCookieの暗号化を解除してくれます。
    pub fn get_ytdlp_browser_arg(&self) -> String {
        self.browser.name().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_from_str() {
        assert!(matches!(
            Browser::from_str("chrome"),
            Some(Browser::Chrome)
        ));
        assert!(matches!(
            Browser::from_str("FIREFOX"),
            Some(Browser::Firefox)
        ));
        assert!(Browser::from_str("unknown").is_none());
    }

    #[test]
    fn test_cookie_detector_creation() {
        let detector = CookieDetector::from_str("chrome");
        assert!(detector.is_ok());

        let detector = CookieDetector::from_str("invalid");
        assert!(detector.is_err());
    }
}
