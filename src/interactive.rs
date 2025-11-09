use std::io::{self, Write};

use crate::quality::QualityPreset;

/// インタラクティブモードでユーザー入力を取得
pub struct InteractiveMode;

impl InteractiveMode {
    /// URLを入力
    pub fn ask_url() -> io::Result<String> {
        println!("\n📺 YouTubeのURLを入力してください:");
        println!("   例: https://www.youtube.com/watch?v=dQw4w9WgXcQ");
        print!("\nURL: ");
        io::stdout().flush()?;

        let mut url = String::new();
        io::stdin().read_line(&mut url)?;
        Ok(url.trim().to_string())
    }

    /// 品質プリセットを選択
    pub fn ask_quality() -> io::Result<QualityPreset> {
        println!("\n🎬 ダウンロード品質を選択してください:");
        println!("   1. 最高画質（4K対応）- デフォルト");
        println!("   2. 最高音質（MP3抽出）");
        println!("   3. 最低画質（プレビュー用）");
        println!("   4. 最小容量（容量優先）");
        print!("\n選択 [1-4, Enter=1]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();

        let quality = match choice {
            "2" => QualityPreset::MaxAudio,
            "3" => QualityPreset::MinVideo,
            "4" => QualityPreset::MinSize,
            _ => QualityPreset::MaxVideo, // デフォルト or "1"
        };

        Ok(quality)
    }

    /// プレイリストかどうか確認
    pub fn ask_playlist() -> io::Result<bool> {
        // URLにplaylist=が含まれているか自動判定するので、ここでは確認のみ
        println!("\n📋 プレイリスト全体をダウンロードしますか？");
        print!("   [y/N]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim().to_lowercase();

        Ok(matches!(choice.as_str(), "y" | "yes" | "はい"))
    }

    /// 字幕をダウンロードするか確認
    pub fn ask_subtitle() -> io::Result<bool> {
        println!("\n💬 字幕もダウンロードしますか？");
        print!("   [y/N]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim().to_lowercase();

        Ok(matches!(choice.as_str(), "y" | "yes" | "はい"))
    }

}
