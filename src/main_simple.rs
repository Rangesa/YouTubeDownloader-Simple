mod cli;
mod cookie_detector;
mod error;
mod interactive;
mod progress_parser;
mod quality;
mod updater;
mod ytdlp_wrapper;

use clap::Parser;
use cli::Cli;
use error::Result;
use interactive::InteractiveMode;
use updater::Updater;
use ytdlp_wrapper::YtdlpWrapper;

/// メインエントリポイント
fn main() {
    // エラーが発生した場合の終了コードを設定
    std::process::exit(match run() {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("\nエラー: {}", e);
            eprintln!("\nEnterキーを押して終了...");
            let mut input = String::new();
            let _ = std::io::stdin().read_line(&mut input);
            1
        }
    });
}

/// 実際の処理を実行
fn run() -> Result<()> {
    // CLIの引数をパース
    let mut cli = Cli::parse();

    // バナー表示
    print_banner();

    // yt-dlp自動更新
    println!("🔄 yt-dlpを最新版に更新中...");
    if let Err(e) = Updater::update_ytdlp() {
        eprintln!("警告: yt-dlp更新失敗: {}", e);
        eprintln!("続行します...\n");
    }

    // yt-dlpが利用可能かチェック
    println!("\n📦 yt-dlpの確認中...");
    YtdlpWrapper::check_ytdlp_available()?;

    // Simple版: デフォルトでCookie無効（明示的に--cookiesが指定された場合のみ有効）
    let args: Vec<String> = std::env::args().collect();
    let has_cookies_arg = args.iter().any(|arg| arg.starts_with("--cookies") || arg == "-c");

    if cli.no_cookies || !has_cookies_arg {
        cli.cookie_browser = None;
    }

    // 出力ディレクトリのデフォルト設定（exeと同じフォルダ）
    if cli.output_dir.is_none() {
        cli.output_dir = Some(
            std::env::current_exe()
                .ok()
                .and_then(|path| path.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| std::path::PathBuf::from("."))
        );
    }

    // アーカイブファイルのデフォルト設定
    if cli.download_archive.is_none() && !cli.no_archive {
        let archive_path = cli.output_dir.as_ref()
            .map(|dir| dir.join("downloaded.txt"))
            .unwrap_or_else(|| std::path::PathBuf::from("downloaded.txt"));
        cli.download_archive = Some(archive_path);
    }

    // インタラクティブモード
    if cli.url.is_none() && !cli.non_interactive {
        println!("\n🎮 インタラクティブモードで起動しました");

        // URL入力
        let url = InteractiveMode::ask_url()
            .map_err(|e| error::YtdlError::Other(format!("入力エラー: {}", e)))?;

        if url.is_empty() {
            eprintln!("エラー: URLが入力されませんでした");
            std::process::exit(1);
        }
        cli.url = Some(url);

        // 品質選択
        cli.quality = InteractiveMode::ask_quality()
            .map_err(|e| error::YtdlError::Other(format!("入力エラー: {}", e)))?;

        // プレイリストか確認（URLに"playlist"が含まれている場合のみ）
        if cli.url.as_ref().unwrap().contains("playlist") {
            cli.playlist = InteractiveMode::ask_playlist()
                .map_err(|e| error::YtdlError::Other(format!("入力エラー: {}", e)))?;
        }

        // 字幕確認
        cli.download_subtitle = InteractiveMode::ask_subtitle()
            .map_err(|e| error::YtdlError::Other(format!("入力エラー: {}", e)))?;
    } else if cli.url.is_none() {
        eprintln!("エラー: URLを指定してください");
        std::process::exit(1);
    }

    // 設定の妥当性チェック
    if let Err(e) = cli.validate() {
        eprintln!("設定エラー: {}", e);
        std::process::exit(1);
    }

    // 設定を表示
    println!();
    cli.display_config();
    println!();

    // ダウンロード実行
    let wrapper = YtdlpWrapper::new(cli);
    wrapper.download()?;

    // 完了メッセージ
    println!("\n✅ すべてのダウンロードが完了しました！");
    println!("📁 ファイルはexeと同じフォルダに保存されています\n");

    // Windows環境では終了前に待機
    #[cfg(target_os = "windows")]
    {
        println!("Enterキーを押して終了...");
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);
    }

    Ok(())
}

/// バナーを表示
fn print_banner() {
    println!(
        r#"
╔═══════════════════════════════════════════════════╗
║   YouTube Batch Downloader (Simple)               ║
║   シンプル版 - Cookie不要                         ║
╚═══════════════════════════════════════════════════╝
"#
    );
}
