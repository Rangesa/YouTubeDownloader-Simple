use thiserror::Error;

/// アプリケーション全体で使用するエラー型
#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum YtdlError {
    #[error("yt-dlpが見つかりません。Dockerコンテナ内で実行するか、yt-dlpをインストールしてください")]
    YtdlpNotFound,

    #[error("Cookie検出エラー: {0}")]
    CookieDetection(String),

    #[error("ダウンロードエラー: {0}")]
    DownloadFailed(String),

    #[error("yt-dlpプロセスエラー: {0}")]
    ProcessError(String),

    #[error("進捗パースエラー: {0}")]
    ProgressParseError(String),

    #[error("IO エラー: {0}")]
    IoError(#[from] std::io::Error),

    #[error("その他のエラー: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, YtdlError>;
