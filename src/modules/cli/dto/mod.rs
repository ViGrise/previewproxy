use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
  /// Run the proxy server (default)
  Serve,
  /// Upgrade the binary to the latest release
  Upgrade,
  /// Encrypt a source URL for use with previewproxy
  EncryptUrl {
    /// The source URL to encrypt
    url: String,
    /// Hex-encoded AES key (16, 24, or 32 bytes) [env: SOURCE_URL_ENCRYPTION_KEY]
    #[arg(long, env = "SOURCE_URL_ENCRYPTION_KEY")]
    key: String,
  },
}
