# YouTube Batch Downloader

**高速・高品質なYouTube動画一括ダウンロードツール**

Rust製の単一実行ファイルで、yt-dlpをバックエンドに使用した軽量かつ強力なCLIツールです。

## 2つのバージョン

このツールには用途に応じた2つのバージョンがあります：

### ytdl.exe - Cookie版（大量ダウンロード用）

- プレイリストやチャンネルの全動画を一括ダウンロード
- デフォルトでChromeのCookieを使用してBot対策を回避
- Chromeを終了する必要あり

### ytdl-simple.exe - シンプル版（単発ダウンロード用）【推奨】

- 単発の動画を素早くダウンロード
- Cookie不要、ブラウザを閉じる必要なし
- すぐに使える

## 特徴

- **2つの実行ファイル** - Cookie版とシンプル版を用途に応じて使い分け
- **単一実行ファイル** - Rustでコンパイルされた軽量バイナリ（1.8MB）
- **ダウンロード済み自動スキップ** - 一度ダウンロードした動画は二度とダウンロードしない
- **exeと同じフォルダに保存** - ファイル管理が簡単
- **品質プリセット** - 最高画質/音質/最低画質/最小容量を簡単切替
- **リアルタイム進捗表示** - 美しいプログレスバーと速度表示
- **自動リトライ** - ネットワークエラー時の自動再試行
- **プレイリスト対応** - 複数動画の一括ダウンロード
- **クロスプラットフォーム** - Windows/macOS/Linux対応

## クイックスタート

### 必要環境

- Windows/macOS/Linux
- yt-dlp（初回起動時に自動インストール）
- ffmpeg

### インストール

#### Windows

1. [Releases](https://github.com/Rangesa/YouTubeDownloader-Simple/releases)から最新版をダウンロード
2. `ytdl-simple.exe` をダウンロード（初心者向け）
3. ダブルクリックで起動

#### ソースからビルド

```bash
# リポジトリをクローン
git clone https://github.com/Rangesa/YouTubeDownloader-Simple.git
cd YouTubeDownloader-Simple

# ビルド
cargo build --release

# 2つの実行ファイルが生成されます
# target/release/ytdl.exe (Cookie版)
# target/release/ytdl-simple.exe (シンプル版)
```

## 使い方

### ytdl-simple.exe（推奨）

**単発の動画をダウンロード**

1. `ytdl-simple.exe` をダブルクリック
2. URLを入力
3. Enter連打でダウンロード開始

```
URL: https://www.youtube.com/watch?v=dQw4w9WgXcQ
品質: [Enter] ← 最高画質
字幕: [Enter] ← なし
→ ダウンロード開始
```

### ytdl.exe（大量ダウンロード用）

**チャンネルやプレイリストを一括ダウンロード**

1. **Chromeを完全に終了**
2. `ytdl.exe` をダブルクリック
3. チャンネルURLを入力

```
URL: https://www.youtube.com/@channelname/videos
品質: [Enter]
→ チャンネルの全動画をダウンロード
```

### コマンドラインオプション

```bash
# シンプル版（Cookie不要）
ytdl-simple.exe <URL>

# Cookie版（大量ダウンロード）
ytdl.exe <URL>

# 最高音質でMP3抽出
ytdl-simple.exe -q max-audio <URL>

# 字幕も保存
ytdl-simple.exe -s <URL>

# プレイリスト全体をダウンロード
ytdl.exe -p <プレイリストURL>

# 別のブラウザを使用（Cookie版）
ytdl.exe --cookies firefox <URL>
ytdl.exe --cookies edge <URL>
```

## 保存先

ダウンロードした動画は**exeファイルと同じフォルダ**に保存されます：

```
C:\your\folder\
├── ytdl.exe
├── ytdl-simple.exe
├── downloaded.txt          ← アーカイブファイル（自動作成）
├── 動画タイトル1-ID1.mp4
├── 動画タイトル2-ID2.mp4
└── ...
```

## ダウンロード済み動画の自動スキップ

`downloaded.txt` に動画IDが記録され、**一度ダウンロードした動画は自動的にスキップ**されます。

### アーカイブをリセット

```bash
# 全て再ダウンロードしたい場合
ytdl.exe --no-archive <URL>

# または downloaded.txt を削除
```

## 品質プリセット

| 番号 | プリセット | 説明 |
|------|-----------|------|
| 1 | 最高画質（4K対応） | デフォルト、アーカイブ用 |
| 2 | 最高音質（MP3抽出） | 音楽、ポッドキャスト用 |
| 3 | 最低画質（プレビュー用） | 確認用 |
| 4 | 最小容量（容量優先） | ストレージ節約 |

## トラブルシューティング

### Cookieエラーが発生する（ytdl.exe）

```
ERROR: Could not copy Chrome cookie database
```

**解決方法**:
1. Chromeを完全に終了
2. タスクマネージャーで `chrome.exe` プロセスを全て終了
3. 再実行

または：
```bash
# Firefoxを使用
ytdl.exe --cookies firefox <URL>

# Edgeを使用
ytdl.exe --cookies edge <URL>

# シンプル版に切り替え
ytdl-simple.exe <URL>
```

### Bot判定される（ytdl-simple.exe）

大量ダウンロード時にBot判定されることがあります。

**解決方法**:
```bash
# Cookie版に切り替え
ytdl.exe <URL>
```

## 使い分けガイド

### ytdl-simple.exe を使う場合

- 単発の動画ダウンロード
- すぐにダウンロードしたい
- ブラウザを閉じたくない
- 5本以下の動画

### ytdl.exe を使う場合

- プレイリスト全体のダウンロード
- チャンネルの全動画ダウンロード
- 10本以上の大量ダウンロード
- Bot判定を回避したい

## 開発

### ビルド

```bash
# 両方のバージョンをビルド
cargo build --release

# Cookie版のみ
cargo build --release --bin ytdl

# シンプル版のみ
cargo build --release --bin ytdl-simple
```

### プロジェクト構成

```
youtube-batch-downloader/
├── src/
│   ├── main.rs              # Cookie版のエントリポイント
│   ├── main_simple.rs       # シンプル版のエントリポイント
│   ├── cli.rs               # CLI引数パーサー
│   ├── quality.rs           # 品質プリセット定義
│   ├── cookie_detector.rs   # Cookie自動検出
│   ├── ytdlp_wrapper.rs     # yt-dlpプロセス管理
│   ├── progress_parser.rs   # 進捗パーサー
│   └── error.rs             # エラー型定義
├── Cargo.toml               # Rust依存関係
├── Dockerfile               # Dockerビルド設定
├── docker-compose.yml       # Docker Compose設定
├── 使い方.md                # 詳細な使い方ガイド（日本語）
└── README.md                # このファイル
```

## Docker使用（オプション）

```bash
# イメージをビルド
docker-compose build

# 動画をダウンロード
docker-compose run --rm ytdl -q max-video "https://www.youtube.com/watch?v=VIDEO_ID"
```

## 技術スタック

- **言語**: Rust 1.75+
- **CLI**: clap 4.5
- **進捗表示**: indicatif 0.17
- **エラー処理**: thiserror, anyhow
- **バックエンド**: yt-dlp（Python）
- **動画処理**: ffmpeg

## ライセンス

MIT License

## 注意事項

- YouTubeの利用規約に従ってご利用ください
- 著作権で保護されたコンテンツの無断ダウンロードは違法です
- 自分がアップロードした動画や、ダウンロードが許可されたコンテンツのみをダウンロードしてください
- このツールの使用によって生じたいかなる損害についても、開発者は責任を負いません

## 貢献

バグ報告や機能要望は[GitHub Issues](https://github.com/Rangesa/YouTubeDownloader-Simple/issues)へお願いします。
プルリクエストも歓迎します。

## 関連リンク

- [yt-dlp](https://github.com/yt-dlp/yt-dlp)
- [ffmpeg](https://ffmpeg.org/)
- [Rust](https://www.rust-lang.org/)
