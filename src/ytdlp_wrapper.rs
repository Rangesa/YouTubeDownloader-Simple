use indicatif::{ProgressBar, ProgressStyle};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use crate::cli::Cli;
use crate::cookie_detector::CookieDetector;
use crate::error::{Result, YtdlError};
use crate::progress_parser::ProgressParser;

/// yt-dlpラッパー
///
/// yt-dlpプロセスを管理し、ダウンロードを実行します。
pub struct YtdlpWrapper {
    cli: Cli,
    progress_parser: ProgressParser,
}

impl YtdlpWrapper {
    /// 新しいyt-dlpラッパーを作成
    pub fn new(cli: Cli) -> Self {
        Self {
            cli,
            progress_parser: ProgressParser::new(),
        }
    }

    /// yt-dlpが利用可能かチェック
    pub fn check_ytdlp_available() -> Result<()> {
        let output = Command::new("yt-dlp")
            .arg("--version")
            .output()
            .map_err(|_| YtdlError::YtdlpNotFound)?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("yt-dlp バージョン: {}", version.trim());
            Ok(())
        } else {
            Err(YtdlError::YtdlpNotFound)
        }
    }

    /// ダウンロードを実行
    pub fn download(&self) -> Result<()> {
        // 出力ディレクトリを作成
        if let Some(output_dir) = &self.cli.output_dir {
            if !output_dir.exists() {
                std::fs::create_dir_all(output_dir)?;
            }
        }

        // yt-dlpコマンドを構築
        let mut cmd = self.build_command()?;

        if self.cli.verbose {
            println!("\n実行コマンド: {:?}\n", cmd);
        }

        // プロセスを起動
        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| YtdlError::ProcessError(format!("プロセス起動失敗: {}", e)))?;

        // 進捗バーを作成
        let pb = ProgressBar::new(100);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {percent}% | {msg}")
                .expect("Progress template invalid")
                .progress_chars("#>-"),
        );

        // 標準出力を読み取り
        if let Some(stdout) = child.stdout.take() {
            let mut reader = BufReader::new(stdout);
            let mut buffer = Vec::new();

            // UTF-8でない可能性があるため、バイト単位で読み取り
            loop {
                buffer.clear();
                match reader.read_until(b'\n', &mut buffer) {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        // lossy変換でUTF-8に変換（不正なバイトは置換）
                        let line = String::from_utf8_lossy(&buffer).to_string();
                        let line = line.trim_end();

                if self.cli.verbose {
                    println!("{}", line);
                }

                        // 進捗情報をパース
                        if let Ok(Some(progress)) = self.progress_parser.parse(&line) {
                            pb.set_position(progress.percent as u64);
                            pb.set_message(format!(
                                "{} / {} | {} | ETA {}",
                                progress.downloaded_size_str(),
                                progress.total_size_str(),
                                progress.speed_str(),
                                progress.eta_str()
                            ));
                        } else if line.contains("[download]") {
                            // その他のダウンロード情報も表示
                            pb.println(&line);
                        }
                    }
                    Err(e) => {
                        // 読み取りエラー（通常は発生しない）
                        eprintln!("警告: 出力読み取りエラー: {}", e);
                        break;
                    }
                }
            }
        }

        pb.finish_with_message("完了");

        // stderrも読み取り（エラーメッセージ用）
        let stderr_content = if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            let lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();
            lines.join("\n")
        } else {
            String::new()
        };

        // プロセスの終了を待つ
        let status = child
            .wait()
            .map_err(|e| YtdlError::ProcessError(e.to_string()))?;

        if status.success() {
            println!("\n✓ ダウンロードが正常に完了しました");
            Ok(())
        } else {
            // Bot検出エラーの特別処理
            if stderr_content.contains("Sign in to confirm you're not a bot") {
                eprintln!("\n❌ YouTubeのBot対策により、ブラウザのCookie認証が必要です\n");
                eprintln!("📝 解決方法:");
                eprintln!("  1. Chromeを開いてYouTubeにログインしてください");
                eprintln!("  2. ログイン後、このツールを再度実行してください");
                eprintln!("  3. デフォルトでChromeのCookieを使用します\n");
                eprintln!("別のブラウザを使用する場合:");
                eprintln!("  --cookies firefox  (Firefoxの場合)");
                eprintln!("  --cookies edge     (Edgeの場合)\n");

                return Err(YtdlError::DownloadFailed(
                    "YouTube認証エラー: ブラウザでログインしてください".to_string()
                ));
            }

            // Cookie コピーエラーの特別処理
            if stderr_content.contains("Could not copy Chrome cookie database") {
                eprintln!("\n❌ ChromeのCookieデータベースをコピーできませんでした\n");
                eprintln!("📝 解決方法（以下のいずれかを試してください）:");
                eprintln!("  1. Chromeを完全に終了してから、再度このツールを実行");
                eprintln!("  2. タスクマネージャーでChrome関連プロセスを全て終了");
                eprintln!("  3. Firefoxを使用: ytdl.exe --cookies firefox <URL>");
                eprintln!("  4. Edgeを使用: ytdl.exe --cookies edge <URL>\n");
                eprintln!("💡 ヒント: Chromeが起動中だとCookieファイルがロックされます\n");

                return Err(YtdlError::DownloadFailed(
                    "Cookie読み込みエラー: Chromeを終了してください".to_string()
                ));
            }

            // その他のエラー詳細を表示
            eprintln!("\n❌ yt-dlpエラー詳細:");
            if !stderr_content.is_empty() {
                eprintln!("{}", stderr_content);
            }
            Err(YtdlError::DownloadFailed(format!(
                "yt-dlpがエラーコード{}で終了しました",
                status.code().unwrap_or(-1)
            )))
        }
    }

    /// yt-dlpコマンドを構築
    fn build_command(&self) -> Result<Command> {
        let mut cmd = Command::new("yt-dlp");

        // 基本オプション
        cmd.arg("--newline"); // 進捗を毎行出力
        cmd.arg("--progress"); // 進捗表示を有効化

        // 品質設定
        let format_str = self.cli.quality.to_ytdlp_format();
        cmd.arg("-f").arg(&format_str);

        // 音声抽出が必要な場合
        if self.cli.quality.needs_audio_extraction() {
            cmd.arg("-x"); // 音声抽出
            cmd.arg("--audio-format").arg("mp3"); // MP3形式に変換
            cmd.arg("--audio-quality").arg("0"); // 最高品質
        }

        // Cookie設定
        if let Some(browser) = &self.cli.cookie_browser {
            let detector = CookieDetector::from_str(browser)?;
            let browser_arg = detector.get_ytdlp_browser_arg();
            cmd.arg("--cookies-from-browser").arg(browser_arg);

            if self.cli.verbose {
                println!("🍪 {}ブラウザのCookieを使用します", browser);
            }

            // Cookie検出を試みる（警告のみ）
            if let Err(e) = detector.detect_cookie_path() {
                eprintln!("警告: Cookieパスの検出に失敗しました: {}", e);
                eprintln!("ヒント: {}でYouTubeにログインしていることを確認してください", browser);
            }
        } else if self.cli.verbose {
            println!("⚠️  Cookieを使用しません（Bot判定される可能性があります）");
        }

        // 出力先設定
        let output_template = if let Some(template) = &self.cli.output_template {
            template.clone()
        } else {
            "%(title)s-%(id)s.%(ext)s".to_string()
        };

        let output_path = if let Some(output_dir) = &self.cli.output_dir {
            output_dir.join(output_template).to_string_lossy().to_string()
        } else {
            output_template
        };
        cmd.arg("-o").arg(output_path);

        // プレイリスト設定
        if self.cli.playlist {
            // プレイリスト範囲
            if let Some(start) = self.cli.playlist_start {
                cmd.arg("--playlist-start").arg(start.to_string());
            }
            if let Some(end) = self.cli.playlist_end {
                cmd.arg("--playlist-end").arg(end.to_string());
            }
        } else {
            // 単一動画のみダウンロード
            cmd.arg("--no-playlist");
        }

        // 字幕設定
        if self.cli.download_subtitle {
            cmd.arg("--write-subs"); // 字幕をダウンロード
            cmd.arg("--write-auto-subs"); // 自動生成字幕もダウンロード
            cmd.arg("--sub-lang").arg("ja,en"); // 日本語と英語
        }

        // メタデータ設定
        if self.cli.save_metadata {
            cmd.arg("--write-info-json"); // メタデータをJSONで保存
            cmd.arg("--write-description"); // 説明文を保存
            cmd.arg("--write-thumbnail"); // サムネイルを保存
        }

        // 帯域制限
        if let Some(rate) = &self.cli.rate_limit {
            cmd.arg("--limit-rate").arg(rate);
        }

        // リトライ設定
        cmd.arg("--retries").arg(self.cli.retry_count.to_string());

        // ダウンロードアーカイブ（中断再開用）
        if let Some(archive) = &self.cli.download_archive {
            cmd.arg("--download-archive")
                .arg(archive.to_string_lossy().to_string());
        }

        // その他の推奨オプション
        cmd.arg("--no-warnings"); // 警告を抑制
        // --no-call-home は非推奨になったため削除
        cmd.arg("--ignore-errors"); // エラーが出ても続行
        cmd.arg("--no-continue"); // 部分ダウンロードファイルを再利用しない

        // エンコーディング設定（Windows用）
        #[cfg(target_os = "windows")]
        {
            cmd.arg("--encoding").arg("utf-8");
        }

        // URL
        if let Some(url) = &self.cli.url {
            cmd.arg(url);
        } else {
            return Err(YtdlError::Other("URLが指定されていません".to_string()));
        }

        Ok(cmd)
    }

    /// ドライラン（実際にはダウンロードせず、情報のみ取得）
    #[allow(dead_code)]
    pub fn dry_run(&self) -> Result<()> {
        let mut cmd = Command::new("yt-dlp");
        cmd.arg("--dump-json");
        cmd.arg("--flat-playlist");

        if let Some(url) = &self.cli.url {
            cmd.arg(url);
        } else {
            return Err(YtdlError::Other("URLが指定されていません".to_string()));
        }

        if let Some(browser) = &self.cli.cookie_browser {
            let detector = CookieDetector::from_str(browser)?;
            let browser_arg = detector.get_ytdlp_browser_arg();
            cmd.arg("--cookies-from-browser").arg(browser_arg);
        }

        let output = cmd
            .output()
            .map_err(|e| YtdlError::ProcessError(format!("ドライラン実行失敗: {}", e)))?;

        if output.status.success() {
            let json_output = String::from_utf8_lossy(&output.stdout);
            println!("=== 動画情報 ===");
            println!("{}", json_output);
            Ok(())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(YtdlError::DownloadFailed(format!(
                "情報取得失敗: {}",
                error
            )))
        }
    }
}
