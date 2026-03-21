use clap::Parser;

/// ViGrise PreviewProxy — on-the-fly image proxy and transformer
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
  /// Server port [env: PORT]
  #[arg(long, env = "PORT", default_value = "8080")]
  pub port: u16,

  /// Environment: development or production [env: APP_ENV]
  #[arg(long, env = "APP_ENV", default_value = "development")]
  pub env: String,

  /// HMAC signing key (leave empty to disable) [env: HMAC_KEY]
  #[arg(long, env = "HMAC_KEY")]
  pub hmac_key: Option<String>,

  /// Comma-separated allowed upstream hosts (empty = allow all) [env: ALLOWED_HOSTS]
  #[arg(long, env = "ALLOWED_HOSTS", default_value = "")]
  pub allowed_hosts: String,

  /// Upstream fetch timeout in seconds [env: FETCH_TIMEOUT_SECS]
  #[arg(long, env = "FETCH_TIMEOUT_SECS", default_value = "10")]
  pub fetch_timeout_secs: u64,

  /// Maximum source image size in bytes [env: MAX_SOURCE_BYTES]
  #[arg(long, env = "MAX_SOURCE_BYTES", default_value = "20971520")]
  pub max_source_bytes: u64,

  /// L1 in-memory cache size in MB [env: CACHE_MEMORY_MAX_MB]
  #[arg(long, env = "CACHE_MEMORY_MAX_MB", default_value = "256")]
  pub cache_memory_max_mb: u64,

  /// L1 in-memory cache TTL in seconds [env: CACHE_MEMORY_TTL_SECS]
  #[arg(long, env = "CACHE_MEMORY_TTL_SECS", default_value = "3600")]
  pub cache_memory_ttl_secs: u64,

  /// L2 disk cache directory [env: CACHE_DIR]
  #[arg(long, env = "CACHE_DIR", default_value = "/tmp/previewproxy")]
  pub cache_dir: String,

  /// L2 disk cache TTL in seconds [env: CACHE_DISK_TTL_SECS]
  #[arg(long, env = "CACHE_DISK_TTL_SECS", default_value = "86400")]
  pub cache_disk_ttl_secs: u64,

  /// L2 disk cache max size in MB (empty = unlimited) [env: CACHE_DISK_MAX_MB]
  #[arg(long, env = "CACHE_DISK_MAX_MB", default_value = "")]
  pub cache_disk_max_mb: String,

  /// Cache cleanup interval in seconds [env: CACHE_CLEANUP_INTERVAL_SECS]
  #[arg(long, env = "CACHE_CLEANUP_INTERVAL_SECS", default_value = "600")]
  pub cache_cleanup_interval_secs: u64,

  /// Path to the ffmpeg binary [env: FFMPEG_PATH]
  #[arg(long, env = "FFMPEG_PATH", default_value = "ffmpeg")]
  pub ffmpeg_path: String,
}

impl Cli {
  pub fn apply_to_env(&self) {
    unsafe {
      std::env::set_var("PORT", self.port.to_string());
      std::env::set_var("APP_ENV", &self.env);
      std::env::set_var("HMAC_KEY", self.hmac_key.as_deref().unwrap_or(""));
      std::env::set_var("ALLOWED_HOSTS", &self.allowed_hosts);
      std::env::set_var("FETCH_TIMEOUT_SECS", self.fetch_timeout_secs.to_string());
      std::env::set_var("MAX_SOURCE_BYTES", self.max_source_bytes.to_string());
      std::env::set_var("CACHE_MEMORY_MAX_MB", self.cache_memory_max_mb.to_string());
      std::env::set_var(
        "CACHE_MEMORY_TTL_SECS",
        self.cache_memory_ttl_secs.to_string(),
      );
      std::env::set_var("CACHE_DIR", &self.cache_dir);
      std::env::set_var("CACHE_DISK_TTL_SECS", self.cache_disk_ttl_secs.to_string());
      std::env::set_var("CACHE_DISK_MAX_MB", &self.cache_disk_max_mb);
      std::env::set_var(
        "CACHE_CLEANUP_INTERVAL_SECS",
        self.cache_cleanup_interval_secs.to_string(),
      );
      std::env::set_var("FFMPEG_PATH", &self.ffmpeg_path);
    }
  }
}
