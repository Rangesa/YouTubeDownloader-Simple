use clap::Parser;
use std::path::PathBuf;

use crate::quality::QualityPreset;

/// YouTube動画一括ダウンローダー
///
/// 自分のYouTube動画やプレイリストを一括でダウンロードするCLIツール。
/// Chrome/Firefox/Edgeのブラウザクッキーを自動検出してプライベート動画にも対応。
#[derive(Parser, Debug)]
#[command(name = "ytdl")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// ダウンロード対象のURL（動画URLまたはプレイリストURL）
    ///
    /// URLを指定しない場合はインタラクティブモードで起動します。
    #[arg(value_name = "URL")]
    pub url: Option<String>,

    /// インタラクティブモードをスキップ（CI/CDなど自動実行時用）
    #[arg(long = "non-interactive")]
    pub non_interactive: bool,

    /// ダウンロード品質プリセット
    ///
    /// - max-video: 最高画質（4K対応）
    /// - max-audio: 最高音質（音声のみ、MP3変換）
    /// - min-video: 最低画質（プレビュー用）
    /// - min-size: 最小容量
    #[arg(short = 'q', long = "quality", default_value = "max-video")]
    pub quality: QualityPreset,

    /// 出力先ディレクトリ（デフォルト: exeと同じフォルダ）
    #[arg(short = 'o', long = "output")]
    pub output_dir: Option<PathBuf>,

    /// 使用するブラウザのCookie（YouTube認証用）
    ///
    /// YouTubeのBot対策により、ブラウザのCookieがほぼ必須です。
    /// 指定されたブラウザのCookieを自動検出します。
    /// デフォルト: chrome
    /// 無効化する場合は --no-cookies を使用してください。
    #[arg(short = 'c', long = "cookies", default_value = "chrome")]
    pub cookie_browser: Option<String>,

    /// Cookieを使用しない（Bot判定される可能性が高い）
    #[arg(long = "no-cookies", conflicts_with = "cookies")]
    pub no_cookies: bool,

    /// プレイリスト全体をダウンロード
    #[arg(short = 'p', long = "playlist")]
    pub playlist: bool,

    /// プレイリストの開始位置（1から始まる）
    #[arg(long = "from")]
    pub playlist_start: Option<usize>,

    /// プレイリストの終了位置（1から始まる）
    #[arg(long = "to")]
    pub playlist_end: Option<usize>,

    /// 字幕も保存
    #[arg(short = 's', long = "subtitle")]
    pub download_subtitle: bool,

    /// 説明文・メタデータも保存
    #[arg(short = 'm', long = "metadata")]
    pub save_metadata: bool,

    /// 帯域制限（例: 1M, 500K）
    #[arg(long = "limit-rate")]
    pub rate_limit: Option<String>,

    /// リトライ回数
    #[arg(short = 'r', long = "retry", default_value = "3")]
    pub retry_count: usize,

    /// 詳細ログ表示
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// ファイル名フォーマット
    ///
    /// yt-dlpのフォーマット文字列を指定可能。
    /// 例: "%(upload_date)s_%(title)s.%(ext)s"
    /// デフォルト: "%(title)s-%(id)s.%(ext)s"
    #[arg(long = "output-template")]
    pub output_template: Option<String>,

    /// ダウンロード済みアーカイブファイル（中断再開・重複回避用）
    /// デフォルト: exeと同じフォルダに "downloaded.txt" を作成
    #[arg(long = "download-archive")]
    pub download_archive: Option<PathBuf>,

    /// アーカイブ機能を無効化（毎回全てダウンロードし直す）
    #[arg(long = "no-archive")]
    pub no_archive: bool,
}

impl Cli {
    /// 設定の妥当性チェック
    pub fn validate(&self) -> Result<(), String> {
        // プレイリスト範囲の妥当性チェック
        if let (Some(start), Some(end)) = (self.playlist_start, self.playlist_end) {
            if start > end {
                return Err(format!(
                    "プレイリスト開始位置({})が終了位置({})より大きいです",
                    start, end
                ));
            }
            if start == 0 || end == 0 {
                return Err("プレイリスト位置は1から始まります".to_string());
            }
        }

        // 出力ディレクトリのチェック（存在しない場合は警告のみ）
        if let Some(output) = &self.output_dir {
            if !output.exists() {
                eprintln!(
                    "警告: 出力ディレクトリ '{}' が存在しません。自動作成します。",
                    output.display()
                );
            }
        }

        Ok(())
    }

    /// 現在の設定を表示
    pub fn display_config(&self) {
        println!("=== ダウンロード設定 ===");
        if let Some(url) = &self.url {
            println!("URL: {}", url);
        }
        println!("品質: {} ({})",
            format!("{:?}", self.quality).to_lowercase(),
            self.quality.description()
        );
        if let Some(output) = &self.output_dir {
            println!("出力先: {}", output.display());
        } else {
            println!("出力先: exeと同じフォルダ");
        }

        if let Some(browser) = &self.cookie_browser {
            println!("Cookie: {} ブラウザから自動検出", browser);
        } else {
            println!("Cookie: 使用しない（公開動画のみ）");
        }

        if self.playlist {
            print!("プレイリスト: 全体");
            if let Some(start) = self.playlist_start {
                print!(" (開始: {})", start);
            }
            if let Some(end) = self.playlist_end {
                print!(" (終了: {})", end);
            }
            println!();
        }

        if self.download_subtitle {
            println!("字幕: ダウンロードする");
        }

        if self.save_metadata {
            println!("メタデータ: 保存する");
        }

        if let Some(rate) = &self.rate_limit {
            println!("帯域制限: {}", rate);
        }

        println!("リトライ回数: {}", self.retry_count);
        println!("========================\n");
    }
}
