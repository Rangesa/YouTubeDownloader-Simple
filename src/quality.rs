use clap::ValueEnum;

/// ダウンロード品質プリセット
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum QualityPreset {
    /// 最高画質（4K対応、ベスト動画+ベスト音声）
    #[value(name = "max-video")]
    MaxVideo,

    /// 最高音質（音声のみ抽出、mp3変換）
    #[value(name = "max-audio")]
    MaxAudio,

    /// 最低画質（プレビュー用、低解像度）
    #[value(name = "min-video")]
    MinVideo,

    /// 最小容量（容量優先、品質は最低限）
    #[value(name = "min-size")]
    MinSize,
}

impl QualityPreset {
    /// yt-dlpのフォーマット指定文字列を生成
    pub fn to_ytdlp_format(&self) -> String {
        match self {
            // 最高画質: ベストビデオ+ベストオーディオ、または単体でベスト
            QualityPreset::MaxVideo => "bestvideo+bestaudio/best".to_string(),

            // 最高音質: ベストオーディオのみ（後でmp3に変換）
            QualityPreset::MaxAudio => "bestaudio".to_string(),

            // 最低画質: ワーストビデオ+ワーストオーディオ
            QualityPreset::MinVideo => "worstvideo+worstaudio/worst".to_string(),

            // 最小容量: ワーストでmp4形式のもの
            QualityPreset::MinSize => "worst[ext=mp4]".to_string(),
        }
    }

    /// 音声のみの抽出が必要か判定
    pub fn needs_audio_extraction(&self) -> bool {
        matches!(self, QualityPreset::MaxAudio)
    }

    /// 説明文を取得
    pub fn description(&self) -> &str {
        match self {
            QualityPreset::MaxVideo => "最高画質（4K対応）",
            QualityPreset::MaxAudio => "最高音質（音声のみ）",
            QualityPreset::MinVideo => "最低画質（プレビュー用）",
            QualityPreset::MinSize => "最小容量",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_strings() {
        assert_eq!(
            QualityPreset::MaxVideo.to_ytdlp_format(),
            "bestvideo+bestaudio/best"
        );
        assert_eq!(QualityPreset::MaxAudio.to_ytdlp_format(), "bestaudio");
        assert_eq!(
            QualityPreset::MinVideo.to_ytdlp_format(),
            "worstvideo+worstaudio/worst"
        );
        assert_eq!(QualityPreset::MinSize.to_ytdlp_format(), "worst[ext=mp4]");
    }

    #[test]
    fn test_audio_extraction_flag() {
        assert!(!QualityPreset::MaxVideo.needs_audio_extraction());
        assert!(QualityPreset::MaxAudio.needs_audio_extraction());
        assert!(!QualityPreset::MinVideo.needs_audio_extraction());
        assert!(!QualityPreset::MinSize.needs_audio_extraction());
    }
}
