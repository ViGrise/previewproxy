use clap::Parser;
use previewproxy::modules::cli::{Cli, Commands};

fn main() {
  // Load --env-file before full CLI parse so env vars are available to clap
  let args: Vec<String> = std::env::args().collect();
  let env_file = args
    .windows(2)
    .find(|w| w[0] == "--env-file")
    .map(|w| w[1].as_str())
    .or_else(|| args.iter().find_map(|a| a.strip_prefix("--env-file=")));

  if let Some(path) = env_file {
    dotenvy::from_filename(path).unwrap_or_else(|e| {
      eprintln!("error loading env file '{path}': {e}");
      std::process::exit(1);
    });
  } else {
    dotenvy::dotenv().ok();
  }

  let cli = Cli::parse();
  cli.apply_to_env();

  let rt = tokio::runtime::Runtime::new().unwrap();

  match cli.command {
    Some(Commands::Upgrade) => {
      rt.block_on(async {
        if let Err(e) = previewproxy::modules::cli::subcommands::upgrade::run_upgrade().await {
          eprintln!("upgrade failed: {e}");
          std::process::exit(1);
        }
      });
    }
    Some(Commands::EncryptUrl { url, key }) => {
      match previewproxy::modules::cli::subcommands::encrypt_url::run_encrypt_url(&url, &key) {
        Ok(blob) => {
          println!("{blob}");
          eprintln!("/enc/{blob}");
        }
        Err(e) => {
          eprintln!("encrypt-url failed: {e}");
          std::process::exit(1);
        }
      }
    }
    None | Some(Commands::Serve) => {
      previewproxy::common::config::telemetry::setup_tracing();
      let cfg = previewproxy::common::config::Configuration::new();
      rt.block_on(async {
        use previewproxy::modules;

        let metrics = modules::metrics::Metrics::new(&cfg.prometheus_namespace);

        // Set startup gauges
        metrics.workers.set(cfg.max_concurrent_requests as i64);
        metrics.buffer_default_size_bytes.set(cfg.max_source_bytes as f64);
        metrics.buffer_max_size_bytes.set(cfg.max_source_bytes as f64);

        let cache = modules::cache::manager::CacheManager::new(&cfg, metrics.clone());
        let cache_clone = cache.clone();
        let interval = cfg.cache_cleanup_interval_secs;
        tokio::spawn(async move {
          let mut ticker = tokio::time::interval(std::time::Duration::from_secs(interval));
          loop {
            ticker.tick().await;
            cache_clone.run_cleanup().await;
          }
        });

        // Spawn Prometheus metrics server if configured
        if let Some(bind) = cfg.prometheus_bind {
          let metrics_router = modules::metrics::prometheus::router(metrics.clone());
          let metrics_listener = tokio::net::TcpListener::bind(bind).await.unwrap_or_else(|e| {
            tracing::error!("Failed to bind Prometheus listener on {bind}: {e}");
            std::process::exit(1);
          });
          tracing::info!("Prometheus metrics listening on http://{bind}/metrics");
          tokio::spawn(async move {
            axum::serve(metrics_listener, metrics_router).await.unwrap();
          });
        }

        let app = previewproxy::app::router(cfg.clone(), cache, metrics).await;
        let listener = tokio::net::TcpListener::bind(cfg.listen_address)
          .await
          .unwrap();
        tracing::info!("Listening on http://{}", cfg.listen_address);
        axum::serve(listener, app)
          .with_graceful_shutdown(previewproxy::common::config::shutdown::shutdown_signal())
          .await
          .unwrap();
      });
    }
  }
}
