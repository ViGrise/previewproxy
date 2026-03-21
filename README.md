<h1 align="center">PreviewProxy</h1>

<p align="center">
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-stable-orange.svg?logo=rust" alt="Rust"></a>
  <a href="https://github.com/tokio-rs/axum"><img src="https://img.shields.io/badge/axum-0.8-blue.svg" alt="Axum"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache%202.0-blue.svg" alt="License: Apache 2.0"></a>
  <a href="https://github.com/vigrise/previewproxy/stargazers"><img src="https://img.shields.io/github/stars/vigrise/previewproxy?style=social" alt="GitHub stars"></a>
  <a href="https://github.com/vigrise/previewproxy/issues"><img src="https://img.shields.io/github/issues/vigrise/previewproxy" alt="GitHub issues"></a>
</p>

A fast, self-hosted HTTP image proxy written in Rust. Fetch remote images by URL, transform them on-the-fly, and serve them with multi-tier caching.

## Features

- **On-the-fly transforms** — resize, crop, rotate, flip, grayscale, brightness/contrast, blur, watermark, format conversion (JPEG, PNG, WebP)
- **Two request styles** — path-style (`/300x200,webp/https://example.com/img.jpg`) and query-style (`/proxy?url=...&w=300&h=200`)
- **Multi-tier cache** — L1 in-memory (moka) + L2 disk with singleflight dedup
- **Security** — allowlist of trusted domains + optional HMAC request signing
- **SSRF protection** — private IP blocking, per-hop allowlist re-validation on redirects
- **Structured JSON logging** via [tracing](https://github.com/tokio-rs/tracing)
- **Video thumbnail extraction** — extract a frame from MP4, MKV, AVI and pass it through the normal transform pipeline
- **Docker** support with multi-stage builds (requires `ffmpeg` at runtime)

## Request Styles

### Path-style

```
GET /{transforms}/{image-url}
```

Transforms are comma-separated tokens before the image URL:

```
GET /300x200/https://example.com/photo.jpg          # resize to 300×200
GET /300x200,webp/https://example.com/photo.jpg     # resize + convert to WebP
GET /,grayscale/https://example.com/photo.jpg       # grayscale only
```

### Query-style

```
GET /proxy?url=https://example.com/photo.jpg&w=300&h=200&format=webp
```

Query params take precedence when both styles are combined.

## Transform Parameters

| Param      | Values                          | Description                          |
| ---------- | ------------------------------- | ------------------------------------ |
| `w`        | integer                         | Output width in pixels               |
| `h`        | integer                         | Output height in pixels              |
| `fit`      | `contain` (default), `cover`    | Resize mode                          |
| `format`   | `jpeg`, `png`, `webp`           | Output format                        |
| `q`        | 1–100 (default: 85)             | Compression quality                  |
| `rotate`   | `90`, `180`, `270`              | Rotation degrees                     |
| `flip`     | `h`, `v`                        | Flip horizontal or vertical          |
| `grayscale`| `true`                          | Convert to grayscale                 |
| `bright`   | -100 to 100                     | Brightness adjustment                |
| `contrast` | -100 to 100                     | Contrast adjustment                  |
| `blur`     | float (sigma)                   | Gaussian blur                        |
| `wm`       | URL                             | Watermark image URL                  |
| `t`        | float (seconds, default: 0)     | Video seek time for frame extraction |
| `sig`      | string                          | HMAC-SHA256 signature (if required)  |

## API Endpoints

| Method | Path       | Description                          |
| ------ | ---------- | ------------------------------------ |
| `GET`  | `/health`  | Health check with cache stats        |
| `GET`  | `/proxy`   | Query-style image proxy              |
| `GET`  | `/*path`   | Path-style image proxy               |

## Getting Started

Download the pre-built binary for your platform from the [releases page](https://github.com/vigrise/previewproxy/releases/latest):

```shell
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m | sed 's/x86_64/x86_64/;s/aarch64/aarch64/;s/arm64/aarch64/')
wget "https://github.com/vigrise/previewproxy/releases/latest/download/previewproxy-${OS}-${ARCH}" -O previewproxy
chmod +x previewproxy
./previewproxy --port 8080 --allowed-hosts img.example.com
```

Or pull and run the latest image from GitHub Container Registry:

```shell
docker pull ghcr.io/vigrise/previewproxy:latest
docker run -d -p 8080:8080 \
  -e ALLOWED_HOSTS=img.example.com \
  -e HMAC_KEY=mysecret \
  ghcr.io/vigrise/previewproxy:latest
```

Or with Docker Compose:

```shell
curl -O https://raw.githubusercontent.com/vigrise/previewproxy/main/docker-compose.yml
curl -O https://raw.githubusercontent.com/vigrise/previewproxy/main/.env.sample
cp .env.sample .env
# Edit .env as needed
docker-compose up -d
```

### CLI Reference

Configuration is read from environment variables (`.env` file) or CLI flags — CLI flags take precedence.

| Flag | Env var | Default | Description |
| --- | --- | --- | --- |
| `--port` | `PORT` | `8080` | Server port |
| `--env` | `APP_ENV` | `development` | `development` or `production` |
| `--hmac-key` | `HMAC_KEY` | — | HMAC signing key; omit to disable |
| `--allowed-hosts` | `ALLOWED_HOSTS` | — | Comma-separated allowed domains; empty = allow all |
| `--fetch-timeout-secs` | `FETCH_TIMEOUT_SECS` | `10` | Upstream fetch timeout (seconds) |
| `--max-source-bytes` | `MAX_SOURCE_BYTES` | `20971520` | Max source image size (bytes) |
| `--cache-memory-max-mb` | `CACHE_MEMORY_MAX_MB` | `256` | L1 in-memory cache size (MB) |
| `--cache-memory-ttl-secs` | `CACHE_MEMORY_TTL_SECS` | `3600` | L1 cache TTL (seconds) |
| `--cache-dir` | `CACHE_DIR` | `/tmp/previewproxy` | L2 disk cache directory |
| `--cache-disk-ttl-secs` | `CACHE_DISK_TTL_SECS` | `86400` | L2 cache TTL (seconds) |
| `--cache-disk-max-mb` | `CACHE_DISK_MAX_MB` | — | L2 disk cache size limit (MB); empty = unlimited |
| `--cache-cleanup-interval-secs` | `CACHE_CLEANUP_INTERVAL_SECS` | `600` | Background cleanup interval (seconds) |

---

## Environment Variables

| Variable                      | Default              | Description                                           |
| ----------------------------- | -------------------- | ----------------------------------------------------- |
| `APP_ENV`                     | —                    | `development` or `production`                         |
| `PORT`                        | `8080`               | Server port                                           |
| `HMAC_KEY`                    | —                    | Secret key for HMAC signing; leave empty to disable   |
| `ALLOWED_HOSTS`               | —                    | Comma-separated allowed domains; empty = allow all    |
| `FETCH_TIMEOUT_SECS`          | `10`                 | Upstream fetch timeout                                |
| `MAX_SOURCE_BYTES`            | `20971520` (20 MB)   | Maximum source image size                             |
| `CACHE_MEMORY_MAX_MB`         | `256`                | L1 in-memory cache size                               |
| `CACHE_MEMORY_TTL_SECS`       | `3600`               | L1 cache TTL                                          |
| `CACHE_DIR`                   | `/tmp/previewproxy`  | L2 disk cache directory                               |
| `CACHE_DISK_TTL_SECS`         | `86400`              | L2 cache TTL                                          |
| `CACHE_DISK_MAX_MB`           | —                    | L2 disk cache size limit; empty = unlimited           |
| `CACHE_CLEANUP_INTERVAL_SECS` | `600`                | Background cleanup interval                           |
| `RUST_LOG`                    | `server=info,...`    | Log level filter                                      |

## Security

### Allowlist

Set `ALLOWED_HOSTS` to a comma-separated list of trusted domains. Wildcards match a single label:

```env
ALLOWED_HOSTS=img.example.com,*.cdn.example.com
```

Leave empty to allow any host (open mode — use only in trusted environments).

### HMAC Signing

Set `HMAC_KEY` to require signed requests. The signature is computed as:

```
HMAC-SHA256(key, canonical_string)
```

where `canonical_string` is alphabetically sorted `key=value` pairs (excluding `sig`) joined by `&`, followed by `:` and the decoded image URL. Encode the result as URL-safe base64 (no padding) and pass it as the `sig` parameter.

## Development

**Runtime dependency:** video thumbnail extraction requires the `ffmpeg` binary in `$PATH`.

```shell
# Ubuntu / Debian
sudo apt install ffmpeg
```

```shell
# Start dev server
cargo run

# Or with watch mode
cargo watch -q -x run

# Run tests
cargo test

# Format
cargo fmt

# Lint
cargo clippy
```

## Contributing

Contributions are welcome. Please open an issue or pull request at [github.com/vigrise/previewproxy](https://github.com/vigrise/previewproxy).

## License

Apache 2.0 — see [LICENSE](LICENSE).
