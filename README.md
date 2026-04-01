<p align="center">
  <img src="docs/img/logo.svg" alt="PreviewProxy" />
</p>

<p align="center">
  <a href="https://github.com/ViGrise/previewproxy/releases/latest"><img src="https://img.shields.io/github/v/release/ViGrise/previewproxy?label=release&color=blue" alt="Latest Release"></a>
  <a href="https://github.com/ViGrise/previewproxy/actions"><img src="https://img.shields.io/github/actions/workflow/status/ViGrise/previewproxy/ci.yml?branch=main&label=CI" alt="CI"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-stable-orange.svg?logo=rust" alt="Rust"></a>
  <a href="https://github.com/tokio-rs/axum"><img src="https://img.shields.io/badge/axum-0.8-blue.svg" alt="Axum"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache%202.0-blue.svg" alt="License: Apache 2.0"></a>
  <a href="https://github.com/ViGrise/previewproxy/pkgs/container/previewproxy"><img src="https://img.shields.io/badge/ghcr.io-previewproxy-blue?logo=docker" alt="GitHub Container Registry"></a>
  <a href="https://github.com/ViGrise/previewproxy/stargazers"><img src="https://img.shields.io/github/stars/ViGrise/previewproxy?style=social" alt="GitHub stars"></a>
  <a href="https://github.com/ViGrise/previewproxy/issues"><img src="https://img.shields.io/github/issues/ViGrise/previewproxy" alt="GitHub issues"></a>
</p>

A fast, self-hosted image proxy written in Rust. Fetch images from HTTP URLs, S3 buckets, or local storage - transform them on-the-fly and serve them with multi-tier caching.

## Features

- **On-the-fly transforms** - resize, crop, rotate, flip, grayscale, brightness/contrast, blur, watermark, format conversion (JPEG, PNG, WebP, AVIF, JXL)
- **Multiple sources** - remote HTTP URLs, S3-compatible buckets, and local filesystem
- **Two request styles** - path-style (`/300x200,webp/https://example.com/img.jpg`) and query-style (`/proxy?url=...&w=300&h=200`)
- **Animated GIF pipeline** - output all frames of an animated GIF, optionally applying transforms to a selected frame range
- **Video thumbnail extraction** - extract a frame from MP4, MKV, AVI and pass it through the normal transform pipeline
- **Multi-tier cache** - L1 in-memory (moka) + L2 disk with singleflight dedup
- **Security** - domain allowlist, optional HMAC request signing, source URL encryption (AES-CBC), configurable CORS origins, SSRF protection (private IP blocking, per-hop allowlist re-validation on redirects), input/output/transform disallow lists.

## Request Styles

### Path-style

```
GET /{transforms}/{image-url}
```

Transforms are comma-separated tokens before the image URL:

```bash
# Resize to 300x200
GET /300x200/https://example.com/photo.jpg
# Resize and convert to WebP
GET /300x200,webp/https://example.com/photo.jpg
# Grayscale only
GET /,grayscale/https://example.com/photo.jpg
# Grayscale and blur
GET /,grayscale,blur:0.8/https://example.com/photo.jpg
```

### Query-style

```
GET /proxy?url=https://example.com/photo.jpg&w=300&h=200&format=webp
```

Query params take precedence when both styles are combined.

## Transform Parameters

| Param       | Values                                                                    | Description                                                                                                                                                                       |
| ----------- | ------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `w`         | integer                                                                   | Output width in pixels                                                                                                                                                            |
| `h`         | integer                                                                   | Output height in pixels                                                                                                                                                           |
| `fit`       | `contain` (default), `cover`                                              | Resize mode                                                                                                                                                                       |
| `format`    | `jpeg`, `png`, `webp`, `avif`, `jxl`, `gif`, `bmp`, `tiff`, `ico`, `best` | Output format; `best` encodes in multiple formats and returns the smallest result                                                                                                 |
| `q`         | 1-100 (default: 85)                                                       | Compression quality                                                                                                                                                               |
| `rotate`    | `90`, `180`, `270`                                                        | Rotation degrees                                                                                                                                                                  |
| `flip`      | `h`, `v`                                                                  | Flip horizontal or vertical                                                                                                                                                       |
| `grayscale` | `true`                                                                    | Convert to grayscale                                                                                                                                                              |
| `bright`    | -100 to 100                                                               | Brightness adjustment                                                                                                                                                             |
| `contrast`  | -100 to 100                                                               | Contrast adjustment                                                                                                                                                               |
| `blur`      | float (sigma)                                                             | Gaussian blur                                                                                                                                                                     |
| `wm`        | URL                                                                       | Watermark image URL                                                                                                                                                               |
| `seek`      | `5.0`, `0.5r`, `auto` (default: `0`)                                      | Video seek: absolute seconds, relative ratio, or auto (middle frame)                                                                                                              |
| `gif_anim`  | `all`, `N`, `N-M`, `-N`                                                   | Animated GIF: output all frames, apply transforms starting at frame N, to frame range N-M, or to last N frames                                                                    |
| `gif_af`    | `true`, `1`                                                               | GIF all-frames: output all frames, not just the `gif_anim` range; style transforms apply only to frames in range, geometric transforms (resize, rotate, flip) apply to all frames |
| `sig`       | string                                                                    | HMAC-SHA256 signature (if required)                                                                                                                                               |

## API Endpoints

| Method | Path      | Description                   |
| ------ | --------- | ----------------------------- |
| `GET`  | `/health` | Health check with cache stats |
| `GET`  | `/proxy`  | Query-style image proxy       |
| `GET`  | `/*path`  | Path-style image proxy        |

## Getting Started

### Linux / macOS

```shell
curl -s -o- https://raw.githubusercontent.com/ViGrise/previewproxy/main/install.sh | sudo bash
```

```shell
wget -qO- https://raw.githubusercontent.com/ViGrise/previewproxy/main/install.sh | sudo bash
```

Installs to `/usr/local/bin`. Override with env vars:

| Var           | Default          | Description                |
| ------------- | ---------------- | -------------------------- |
| `INSTALL_DIR` | `/usr/local/bin` | Destination directory      |
| `VERSION`     | `latest`         | Release tag, e.g. `v1.0.0` |

### Windows

```powershell
irm https://raw.githubusercontent.com/ViGrise/previewproxy/main/install.ps1 | iex
```

Installs to `%LOCALAPPDATA%\previewproxy\bin` and adds it to your user `PATH`. Override with flags:

| Flag          | Default                           | Description                |
| ------------- | --------------------------------- | -------------------------- |
| `-InstallDir` | `%LOCALAPPDATA%\previewproxy\bin` | Destination directory      |
| `-Version`    | `latest`                          | Release tag, e.g. `v1.0.0` |

### Docker

```shell
docker run -d -p 8080:8080 \
  -e PP_ALLOWED_HOSTS=img.example.com \
  -e PP_HMAC_KEY=mysecret \
  ghcr.io/vigrise/previewproxy:latest
```

Or with Docker Compose:

```shell
curl -O https://raw.githubusercontent.com/ViGrise/previewproxy/main/docker-compose.yml
curl -O https://raw.githubusercontent.com/ViGrise/previewproxy/main/.env.sample
cp .env.sample .env
# Edit .env as needed
docker-compose up -d
```

### Upgrade

To be upgrade to latest version, you can rerun the installation script or use the following command:

```shell
previewproxy upgrade
```

### CLI Reference

Configuration is read from environment variables (`.env` file) or CLI flags - CLI flags take precedence.

| Flag                                 | Env var                               | Default                        | Description                                                                                                                             |
| ------------------------------------ | ------------------------------------- | ------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------- |
| `--port`, `-p`                       | `PP_PORT`                             | `8080`                         | Server port                                                                                                                             |
| `--env`, `-E`                        | `PP_APP_ENV`                          | `development`                  | `development` or `production`                                                                                                           |
| `--max-concurrent-requests`          | `PP_MAX_CONCURRENT_REQUESTS`          | `256`                          | Max number of concurrent requests before returning 503                                                                                  |
| `--rust-log`                         | `RUST_LOG`                            | `previewproxy=info,...`        | Log level filter                                                                                                                        |
| `--hmac-key`, `-k`                   | `PP_HMAC_KEY`                         | -                              | HMAC signing key; omit to disable                                                                                                       |
| `--allowed-hosts`, `-a`              | `PP_ALLOWED_HOSTS`                    | -                              | Comma-separated allowed domains; empty = allow all                                                                                      |
| `--source-url-encryption-key`        | `PP_SOURCE_URL_ENCRYPTION_KEY`        | -                              | Hex-encoded AES key for source URL encryption (32/48/64 hex chars = AES-128/192/256); omit to disable                                   |
| `--fetch-timeout-secs`, `-t`         | `PP_FETCH_TIMEOUT_SECS`               | `10`                           | Upstream fetch timeout (seconds)                                                                                                        |
| `--max-source-bytes`, `-s`           | `PP_MAX_SOURCE_BYTES`                 | `20971520`                     | Max source image size (bytes)                                                                                                           |
| `--cache-memory-max-mb`              | `PP_CACHE_MEMORY_MAX_MB`              | `256`                          | L1 in-memory cache size (MB)                                                                                                            |
| `--cache-memory-ttl-secs`            | `PP_CACHE_MEMORY_TTL_SECS`            | `3600`                         | L1 cache TTL (seconds)                                                                                                                  |
| `--cache-dir`, `-D`                  | `PP_CACHE_DIR`                        | `/tmp/previewproxy`            | L2 disk cache directory                                                                                                                 |
| `--cache-disk-ttl-secs`              | `PP_CACHE_DISK_TTL_SECS`              | `86400`                        | L2 cache TTL (seconds)                                                                                                                  |
| `--cache-disk-max-mb`                | `PP_CACHE_DISK_MAX_MB`                | -                              | L2 disk cache size limit (MB); empty = unlimited                                                                                        |
| `--cache-cleanup-interval-secs`      | `PP_CACHE_CLEANUP_INTERVAL_SECS`      | `600`                          | Background cleanup interval (seconds)                                                                                                   |
| `--s3-enabled`                       | `PP_S3_ENABLED`                       | `false`                        | Enable S3 as an image source                                                                                                            |
| `--s3-bucket`                        | `PP_S3_BUCKET`                        | -                              | S3 bucket name (required if S3 enabled)                                                                                                 |
| `--s3-region`                        | `PP_S3_REGION`                        | `us-east-1`                    | S3 region                                                                                                                               |
| `--s3-access-key-id`                 | `PP_S3_ACCESS_KEY_ID`                 | -                              | S3 access key ID (required if S3 enabled)                                                                                               |
| `--s3-secret-access-key`             | `PP_S3_SECRET_ACCESS_KEY`             | -                              | S3 secret access key (required if S3 enabled)                                                                                           |
| `--s3-endpoint`                      | `PP_S3_ENDPOINT`                      | -                              | Custom S3 endpoint URL (for Cloudflare R2, RustFS, etc.); omit for AWS                                                                  |
| `--local-enabled`                    | `PP_LOCAL_ENABLED`                    | `false`                        | Enable local filesystem as an image source                                                                                              |
| `--local-base-dir`                   | `PP_LOCAL_BASE_DIR`                   | -                              | Root directory for local file serving (required if local enabled)                                                                       |
| `--ffmpeg-path`                      | `PP_FFMPEG_PATH`                      | `ffmpeg`                       | Path to the ffmpeg binary                                                                                                               |
| `--ffprobe-path`                     | `PP_FFPROBE_PATH`                     | `ffprobe` (same dir as ffmpeg) | Path to the ffprobe binary                                                                                                              |
| `--cors-allow-origin`                | `PP_CORS_ALLOW_ORIGIN`                | `*`                            | Comma-separated allowed CORS origins; `*` = allow all; wildcards (`*.example.com`) match a single subdomain label                       |
| `--cors-max-age-secs`                | `PP_CORS_MAX_AGE_SECS`                | `600`                          | CORS preflight cache duration (seconds)                                                                                                 |
| `--input-disallow-list`              | `PP_INPUT_DISALLOW_LIST`              | -                              | Comma-separated input formats to block: `jpeg`, `png`, `gif`, `webp`, `avif`, `jxl`, `bmp`, `tiff`, `pdf`, `psd`, `video`               |
| `--output-disallow-list`             | `PP_OUTPUT_DISALLOW_LIST`             | -                              | Comma-separated output formats to block: `jpeg`, `png`, `gif`, `webp`, `avif`, `jxl`, `bmp`, `tiff`, `ico`                              |
| `--transform-disallow-list`          | `PP_TRANSFORM_DISALLOW_LIST`          | -                              | Comma-separated transforms to block: `resize`, `rotate`, `flip`, `grayscale`, `brightness`, `contrast`, `blur`, `watermark`, `gif_anim` |
| `--url-aliases`                      | `PP_URL_ALIASES`                      | -                              | Comma-separated alias definitions: `name=https://base.url,name2=https://other.url`; enables `name:/path` URL scheme in requests         |
| `--best-format-complexity-threshold` | `PP_BEST_FORMAT_COMPLEXITY_THRESHOLD` | `5.5`                          | Sobel edge density threshold; images below this are treated as low-complexity (lossless candidates included)                            |
| `--best-format-max-resolution`       | `PP_BEST_FORMAT_MAX_RESOLUTION`       | -                              | When set, images with resolution (megapixels) above this skip the multi-encode trial and pick one format                                |
| `--best-format-by-default`           | `PP_BEST_FORMAT_BY_DEFAULT`           | `false`                        | When true and no format is specified, use best-format selection instead of returning source format                                      |
| `--best-format-allow-skips`          | `PP_BEST_FORMAT_ALLOW_SKIPS`          | `false`                        | When true, skip re-encoding if best format matches source format and no other transforms are applied                                    |
| `--best-format-preferred-formats`    | `PP_BEST_FORMAT_PREFERRED_FORMATS`    | `jpeg,webp,png`                | Comma-separated formats to trial; add `avif` or `jxl` for better compression at the cost of slower encoding                             |

---

## Security

### Upstream Allowlist

Set `PP_ALLOWED_HOSTS` to a comma-separated list of trusted upstream domains. Wildcards match a single label:

```ini
PP_ALLOWED_HOSTS=img.example.com,*.cdn.example.com
```

Leave empty to allow any host (open mode - use only in trusted environments).

### HMAC Signing

Set `PP_HMAC_KEY` to require signed requests. The signature is computed as:

```
HMAC-SHA256(key, canonical_string)
```

where `canonical_string` is alphabetically sorted `key=value` pairs (excluding `sig`) joined by `&`, followed by `:` and the decoded image URL. Encode the result as URL-safe base64 (no padding) and pass it as the `sig` parameter.

### CORS

Set `PP_CORS_ALLOW_ORIGIN` to restrict which browser origins may access the proxy. Wildcards match a single subdomain label:

```ini
PP_CORS_ALLOW_ORIGIN=https://app.example.com,*.cdn.example.com
```

Leave as `*` (default) to allow any origin.

### Formats & Transforms Disallow Lists

Block specific input formats, output formats, or transforms via env vars. Each accepts a comma-separated list of tokens; unknown tokens are ignored with a warning.

**Input formats** (`PP_INPUT_DISALLOW_LIST`) - reject requests whose source image matches a blocked format (returns 400):

```ini
# Block video thumbnails and PDF inputs
PP_INPUT_DISALLOW_LIST=video,pdf
```

Tokens: `jpeg`, `png`, `gif`, `webp`, `avif`, `jxl`, `bmp`, `tiff`, `pdf`, `psd`, `video`

**Output formats** (`PP_OUTPUT_DISALLOW_LIST`) - reject requests that would produce a blocked format (returns 400). Defaults to empty (allow all):

```ini
# Allow all output formats
PP_OUTPUT_DISALLOW_LIST=
```

Tokens: `jpeg`, `png`, `gif`, `webp`, `avif`, `jxl`, `bmp`, `tiff`, `ico`

**Transforms** (`PP_TRANSFORM_DISALLOW_LIST`) - reject requests that apply a blocked transform (returns 400). Defaults to empty (allow all):

```ini
# Block watermarking and animated GIF processing only
PP_TRANSFORM_DISALLOW_LIST=watermark,gif_anim
```

Tokens: `resize`, `rotate`, `flip`, `grayscale`, `brightness`, `contrast`, `blur`, `watermark`, `gif_anim`

> **Note:** Input disallow checks are not applied to sources with no `Content-Type` header (e.g. S3 objects without metadata), as the format cannot be determined before fetching.

### SSRF Protection

Private, loopback, link-local, and reserved IP ranges (RFC 1918, RFC 6598, IPv6 ULA) are always blocked. On redirects, each hop's resolved IP and host are re-validated before following, preventing bypass via open redirectors.

### Source URL Encryption

Set `PP_SOURCE_URL_ENCRYPTION_KEY` to hide upstream origins from request logs and CDN traces. Encrypt URLs with AES-CBC before putting them in requests; previewproxy decrypts server-side before fetching.

**Configure the key** (hex-encoded, 32/48/64 hex chars for AES-128/192/256):

```ini
# Generate a key
openssl rand -hex 32

PP_SOURCE_URL_ENCRYPTION_KEY=1eb5b0e971ad7f45324c1bb15c947cb207c43152fa5c6c7f35c4f36e0c18e0f1
```

**Encrypt a URL** using the built-in CLI helper:

```bash
previewproxy encrypt-url "https://cdn.example.com/photo.jpg" [--key <hex-key>]
# stdout: <encrypted-blob>  (pipe-friendly)
# stderr: /enc/<encrypted-blob>  (ready-to-use path)
```

**Use encrypted URLs in requests:**

```bash
# Path-style
GET /300x200,webp/enc/<encrypted-blob>

# Query-style
GET /proxy?url=<encrypted-blob>&enc=1&w=300&h=200
```

The `enc` query flag (any value, or just its presence) signals that `url` is encrypted.

> **Note:** The same source URL always produces the same encrypted blob (deterministic IV). This is intentional for CDN cache-hit compatibility. Rotate your key if you need to invalidate existing blobs.

> **Recommended:** Use with `PP_HMAC_KEY` signing to prevent padding oracle attacks on encrypted URLs.

### URL Aliases

Map short scheme names to real HTTP base URLs to avoid exposing domains in requests:

```ini
PP_URL_ALIASES=mycdn=https://img.example.com,cdn2=https://other.com
```

Then use the alias scheme in path-style or query-style requests:

```bash
# Path-style with alias
GET /300x200,webp/mycdn:/photos/dog.jpg

# Query-style with alias
GET /proxy?url=mycdn:/photos/dog.jpg&w=300
```

- Aliases bypass `PP_ALLOWED_HOSTS` (operator-controlled, implicitly trusted)
- SSRF protection (private IP blocking) still applies
- Base URL must be `http://` or `https://`

## Development

### Requirements

**Runtime**

- `ffmpeg` + `ffprobe` - video frame extraction and duration probing (`apt install ffmpeg` / `brew install ffmpeg`)

**Build-time native libs**

- `libheif` - HEIF/HEIC image support
- `libjxl` - JPEG XL support
- `libdav1d` - AV1 decoder (used by libheif/libjxl)
- `pkg-config`, `cmake`, `nasm` - build toolchain

```shell
# Ubuntu / Debian
sudo apt install ffmpeg pkg-config cmake nasm libheif-dev libjxl-dev libdav1d-dev

# macOS
brew install ffmpeg pkg-config cmake nasm libheif jpeg-xl dav1d
```

### Commands

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

Contributions are welcome. Please open an issue or pull request at [github.com/ViGrise/previewproxy](https://github.com/ViGrise/previewproxy).

## License

Apache 2.0 - see [LICENSE](LICENSE).
