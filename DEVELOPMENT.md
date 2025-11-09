# YouTube Batch Downloader 開発記録

**作成日**: 2025年10月30日
**バージョン**: 0.1.0

## プロジェクト概要

YouTube動画を一括でダウンロードするためのCLIツール。Rust製の単一実行ファイルとyt-dlpを組み合わせ、Docker統合により環境構築不要で即座に利用可能。

## 技術選択の理由

### なぜRustを選んだか

1. **単一バイナリの生成**
   - 依存関係を含めて1.8MBの単一実行ファイルを生成
   - クロスコンパイルが容易（Windows/macOS/Linux対応）
   - 配布が簡単（Dockerイメージも軽量化）

2. **メモリ安全性とパフォーマンス**
   - ゼロコストの抽象化
   - 所有権システムによるメモリリーク防止
   - 並行処理が安全

3. **エラーハンドリングの強力さ**
   - Result型とOption型による明示的なエラー処理
   - thiserrorクレートによる型安全なエラー定義
   - パニックの最小化

4. **長期的なメンテナンス性**
   - 型システムによる安全なリファクタリング
   - コンパイラが多くのバグを事前検出
   - コミュニティの成熟とエコシステムの充実

### なぜyt-dlpをバックエンドに採用したか

**重要な設計判断**: 一からYouTubeダウンロード機能を実装しない

#### 理由

1. **YouTubeの複雑な仕様変更対応**
   - YouTubeは頻繁に仕様を変更（月次レベル）
   - 認証メカニズムの複雑さ（Cookie暗号化、トークン管理）
   - 動画ストリームと音声ストリームの分離とマージ処理

2. **yt-dlpの継続的メンテナンス**
   - 活発なコミュニティによる即座の仕様変更対応
   - 豊富なオプションとフォーマット対応
   - 安定した実績（数年間の運用）

3. **責任の分離**
   - Rust側: CLI、進捗表示、Cookie検出、設定管理
   - yt-dlp側: YouTube通信、動画処理、フォーマット変換
   - それぞれが得意分野に集中できる

#### トレードオフ

- **依存関係の追加**: Pythonとyt-dlpが必要
  - → Dockerで解決（コンテナ内に統合）
- **プロセス起動のオーバーヘッド**: 外部プロセス呼び出し
  - → 実際のダウンロード時間に比べて無視できる

### なぜDockerを統合したか

1. **環境構築の複雑さの解消**
   - yt-dlp、ffmpeg、Pythonの個別インストールが不要
   - プラットフォーム間の差異を吸収

2. **再現性の保証**
   - 開発環境と実行環境の完全一致
   - バージョン管理が容易

3. **セキュリティの向上**
   - コンテナ内でサンドボックス実行
   - ホストシステムへの影響を最小化

## アーキテクチャ設計

### モジュール構成

```
src/
├── main.rs              # エントリポイント、バナー表示
├── cli.rs               # CLI引数定義（clap）
├── quality.rs           # 品質プリセット定義
├── cookie_detector.rs   # Chrome Cookie自動検出
├── ytdlp_wrapper.rs     # yt-dlpプロセス管理
├── progress_parser.rs   # 進捗パーサー
└── error.rs             # エラー型定義
```

### 設計原則

#### 1. 単一責任の原則（SRP）

各モジュールは明確な責任を持つ：

- **cli.rs**: CLI引数のパースと妥当性チェックのみ
- **quality.rs**: 品質プリセットとyt-dlp形式の変換のみ
- **cookie_detector.rs**: Cookie検出ロジックのみ

#### 2. 依存性逆転の原則（DIP）

- `ytdlp_wrapper.rs`は他のモジュールに依存するが、他のモジュールは`ytdlp_wrapper.rs`に依存しない
- エラー型は`error.rs`で一元管理し、全モジュールから利用

#### 3. 開放/閉鎖原則（OCP）

- 品質プリセットは`QualityPreset` enumで拡張可能
- 新しいブラウザ対応は`Browser` enumに追加するだけ

### データフロー

```
1. ユーザー入力
   ↓
2. CLI引数パース（cli.rs）
   ↓
3. 妥当性チェック
   ↓
4. Cookie検出（cookie_detector.rs）
   ↓
5. yt-dlpコマンド生成（ytdlp_wrapper.rs）
   ↓
6. プロセス実行
   ↓
7. 標準出力パース（progress_parser.rs）
   ↓
8. 進捗バー更新（indicatif）
   ↓
9. ダウンロード完了
```

## 主要機能の実装詳細

### 1. Cookie自動検出

#### 課題

- ブラウザのCookieは暗号化されている（特にWindows）
- プラットフォームごとにCookie保存場所が異なる
- プロファイルが複数存在する場合がある

#### 解決策

**yt-dlpの`--cookies-from-browser`機能を活用**

yt-dlpは以下を自動処理：
- Cookie DBの暗号化解除（Windows DPAPI対応）
- プロファイル検索
- SQLiteデータベースの読み取り

Rust側の実装：
```rust
// プラットフォーム検出
#[cfg(target_os = "windows")]
fn get_windows_cookie_path() { /* ... */ }

#[cfg(target_os = "macos")]
fn get_macos_cookie_path() { /* ... */ }

#[cfg(target_os = "linux")]
fn get_linux_cookie_path() { /* ... */ }
```

### 2. 進捗表示

#### 課題

yt-dlpの進捗出力は非構造化テキスト：
```
[download]  45.2% of 123.45MiB at 1.23MiB/s ETA 00:42
```

#### 解決策

**正規表現によるパース + indicatifによる美しい表示**

```rust
let regex = Regex::new(
    r"\[download\]\s+(?P<percent>[\d.]+)%\s+of\s+(?P<total>[\d.]+)(?P<total_unit>[KMG]iB)"
)?;
```

パースした情報をindicatifのProgressBarに反映：
```
[████████████░░░░░░░░] 45.2% | 56.2MB/123.5MB | 1.23MB/s | ETA 00:42
```

### 3. 品質プリセット

#### 設計思想

ユーザーは技術的詳細を知らなくても、目的に応じた品質を選択できるべき。

```rust
pub enum QualityPreset {
    MaxVideo,   // "最高画質" → yt-dlp: "bestvideo+bestaudio/best"
    MaxAudio,   // "最高音質" → yt-dlp: "bestaudio" + MP3変換
    MinVideo,   // "最低画質" → yt-dlp: "worstvideo+worstaudio/worst"
    MinSize,    // "最小容量" → yt-dlp: "worst[ext=mp4]"
}
```

各プリセットは`to_ytdlp_format()`メソッドでyt-dlp形式に変換。

### 4. エラーハンドリング

#### 階層的なエラー設計

```rust
pub enum YtdlError {
    YtdlpNotFound,              // yt-dlpが見つからない
    CookieDetection(String),    // Cookie検出エラー
    DownloadFailed(String),     // ダウンロード失敗
    ProcessError(String),       // プロセス実行エラー
    IoError(#[from] std::io::Error),  // IO エラー
}
```

各エラーは具体的なメッセージを含み、ユーザーが問題を理解しやすい。

## Docker統合

### マルチステージビルド

```dockerfile
# Stage 1: Rustビルド（Alpine Linux）
FROM rust:1.75-alpine AS builder
RUN apk add musl-dev
COPY . /build
RUN cargo build --release --target x86_64-unknown-linux-musl

# Stage 2: 実行環境（Python + Alpine）
FROM python:3.11-alpine
RUN apk add ffmpeg sqlite
RUN pip install yt-dlp
COPY --from=builder /build/target/.../ytdl /usr/local/bin/
```

### 最適化

- **Alpine Linuxの使用**: 最小限のベースイメージ
- **依存関係のキャッシュ**: Cargo.tomlを先にコピーして依存関係をビルド
- **musl静的リンク**: 動的ライブラリ依存を排除
- **strip**: デバッグシンボル削除

最終イメージサイズ: 約150MB（Python + ffmpeg + yt-dlp + Rustバイナリ）

## パフォーマンス最適化

### Cargo.tomlの最適化設定

```toml
[profile.release]
opt-level = "z"      # サイズ最適化
lto = true           # Link Time Optimization
codegen-units = 1    # 単一コード生成ユニット
strip = true         # シンボル削除
```

結果: **1.8MB**の単一バイナリ

### 並列処理の検討

現時点では非同期処理を採用していない理由：

1. **yt-dlpがボトルネック**: ネットワーク速度とYouTubeサーバーの制限
2. **シンプルさの維持**: 同期処理で十分なパフォーマンス
3. **複雑さの回避**: tokioの導入は将来的な拡張時に検討

## セキュリティ考慮事項

### Cookie取扱い

1. **読み取り専用マウント**: `docker-compose.yml`で`:ro`を使用
2. **メモリ上での処理**: yt-dlpに直接渡し、ファイル保存しない
3. **ログ出力抑制**: Cookie内容をログに出力しない

### サンドボックス実行

- Dockerコンテナ内で実行
- ホストシステムへのアクセス制限
- ダウンロード先ディレクトリのみマウント

## テスト戦略

### 現状

- 単体テスト: 一部実装済み（quality.rs、progress_parser.rs）
- 統合テスト: 未実装

### 今後の課題

1. **モックを使った単体テスト**
   - yt-dlpの出力をモックしてパーサーをテスト
   - Cookie検出ロジックのテスト

2. **統合テスト**
   - 実際の公開動画でダウンロードテスト
   - プレイリストダウンロードのテスト

3. **CI/CD統合**
   - GitHub Actionsでビルドとテスト自動化
   - クロスプラットフォームビルドの自動化

## 今後の拡張計画

### 短期（v0.2.0）

- [ ] 並列ダウンロード機能（複数動画の同時DL）
- [ ] ダウンロード履歴管理（重複回避）
- [ ] 設定ファイルサポート（`.ytdlrc`）

### 中期（v0.3.0）

- [ ] インタラクティブモード（対話的な動画選択）
- [ ] 自動字幕翻訳機能
- [ ] プレイリスト自動更新（新規動画の自動DL）

### 長期（v1.0.0）

- [ ] GUI版の開発（Tauri使用）
- [ ] 他プラットフォーム対応（Vimeo、Twitch等）
- [ ] クラウドストレージ連携（Google Drive、S3等）

## 学んだこと

### Rustのベストプラクティス

1. **エラーハンドリングの重要性**
   - Result型とthiserrorの組み合わせが強力
   - エラーメッセージは具体的に

2. **所有権システムの恩恵**
   - メモリリークの心配が不要
   - コンパイラがバグを事前検出

3. **クレート選定の重要性**
   - clap: CLI構築が超簡単
   - indicatif: 進捗表示が美しい
   - thiserror: エラー定義が型安全

### Docker統合のノウハウ

1. **マルチステージビルドの威力**
   - ビルド環境と実行環境の分離
   - イメージサイズの劇的削減

2. **Alpine Linuxの利点と課題**
   - 利点: 軽量、セキュリティ
   - 課題: musl-devが必要、一部ライブラリの互換性

3. **ボリュームマウントの設計**
   - 読み取り専用マウントでセキュリティ向上
   - 環境変数で柔軟な設定

## トラブルシューティング記録

### 遭遇した問題と解決策

#### 問題1: LazyLockの使用

**症状**: `LazyLock`が`std::sync`に存在しない

**原因**: Rust 1.70.0以降が必要

**解決策**: `once_cell`クレートの`LazyLock`を使用、またはRust 1.80+を使用

#### 問題2: Windows環境でのパス区切り

**症状**: Dockerfileのパスがエラー

**原因**: WindowsとLinuxのパス区切り文字の違い

**解決策**: Docker内部ではLinux形式（`/`）を使用

#### 問題3: Cookie暗号化

**症状**: ChromeのCookieが読み取れない

**解決策**: yt-dlpの`--cookies-from-browser`に任せる（暗号化解除を自動処理）

## パフォーマンス測定

### ビルド時間

- Debug: 約18秒
- Release: 約27秒

### バイナリサイズ

- Debug: 約45MB
- Release（最適化後）: 1.8MB

### 実行時オーバーヘッド

- CLI起動: <100ms
- yt-dlp起動: <500ms
- 進捗パース: ほぼゼロ（ストリーム処理）

実際のダウンロード時間はネットワーク速度とYouTubeサーバーに依存。

## 参考資料

### 技術文書

- [yt-dlp Documentation](https://github.com/yt-dlp/yt-dlp#readme)
- [Rust Book](https://doc.rust-lang.org/book/)
- [clap Documentation](https://docs.rs/clap/)
- [Docker Best Practices](https://docs.docker.com/develop/dev-best-practices/)

### 設計参考

- [youtube-dl](https://github.com/ytdl-org/youtube-dl) - オリジナル実装
- [yt-dlp](https://github.com/yt-dlp/yt-dlp) - フォーク版、活発な開発

## まとめ

本プロジェクトは、Rustの強力な型システムとyt-dlpの成熟した機能を組み合わせることで、シンプルかつ強力なYouTubeダウンローダーを実現した。

**成功要因**:
1. 適切な技術選択（Rust + yt-dlp + Docker）
2. 責任の明確な分離（モジュール設計）
3. ユーザー体験の重視（品質プリセット、進捗表示）
4. セキュリティとパフォーマンスのバランス

**今後の展望**:
- 機能拡張（並列DL、履歴管理）
- GUI版の開発
- コミュニティからのフィードバック反映

---

**開発者ノート**: このツールはYouTubeの利用規約に従って使用されることを前提としています。著作権で保護されたコンテンツの無断ダウンロードは違法です。

**作成**: Claude Code (Anthropic) + Human Developer
**最終更新**: 2025年10月30日
