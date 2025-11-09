# ===========================
# Stage 1: Rustビルドステージ
# ===========================
FROM rust:1.83-alpine AS builder

# 必要なビルドツールをインストール
RUN apk add --no-cache musl-dev

WORKDIR /build

# 依存関係のみを先にビルド（キャッシュ最適化）
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release --target x86_64-unknown-linux-musl && \
    rm -rf src

# 実際のソースコードをコピーしてビルド
COPY src ./src
RUN touch src/main.rs && \
    cargo build --release --target x86_64-unknown-linux-musl

# バイナリのサイズを確認
RUN ls -lh target/x86_64-unknown-linux-musl/release/ytdl

# ===========================
# Stage 2: 最終実行イメージ
# ===========================
FROM python:3.11-alpine

# 作業ディレクトリ
WORKDIR /app

# ffmpeg（動画処理用）とsqlite（Cookie読み取り用）をインストール
RUN apk add --no-cache \
    ffmpeg \
    sqlite \
    ca-certificates \
    && rm -rf /var/cache/apk/*

# yt-dlpをインストール
RUN pip install --no-cache-dir yt-dlp

# Rustバイナリをコピー
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/ytdl /usr/local/bin/ytdl

# 実行権限を付与
RUN chmod +x /usr/local/bin/ytdl

# ボリュームマウントポイント
VOLUME ["/downloads", "/cookies"]

# デフォルトの出力先を設定
ENV OUTPUT_DIR=/downloads

# エントリポイント
ENTRYPOINT ["ytdl"]

# デフォルトのコマンド（ヘルプ表示）
CMD ["--help"]
