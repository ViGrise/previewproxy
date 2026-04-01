pub mod prometheus;

use ::prometheus::{
  Gauge, Histogram, HistogramOpts, HistogramVec, IntCounter, IntCounterVec, IntGauge, IntGaugeVec,
  Opts, Registry,
};
use std::sync::Arc;

pub struct Metrics {
  pub registry: Registry,
  // Counters
  pub requests_total: IntCounter,
  pub status_codes_total: IntCounterVec,
  pub errors_total: IntCounterVec,
  // Histograms
  pub request_duration_seconds: Histogram,
  pub request_span_duration_seconds: HistogramVec,
  pub buffer_size_bytes: Histogram,
  // Gauges
  pub workers: IntGauge,
  pub requests_in_progress: IntGauge,
  pub images_in_progress: IntGauge,
  pub workers_utilization: Gauge,
  pub buffer_default_size_bytes: Gauge,
  pub buffer_max_size_bytes: Gauge,
  // Cache
  pub cache_hits_total: IntCounterVec,
  pub cache_misses_total: IntCounterVec,
  pub cache_memory_size_bytes: Gauge,
  pub cache_disk_size_bytes: Gauge,
  pub cache_entries: IntGaugeVec,
}

impl Metrics {
  pub fn new(namespace: &str) -> Arc<Self> {
    let registry = Registry::new();

    macro_rules! register {
      ($metric:expr) => {{
        let m = $metric;
        registry.register(Box::new(m.clone())).unwrap();
        m
      }};
    }

    let ns = namespace;

    let requests_total = register!(IntCounter::with_opts(
      Opts::new("requests_total", "Total number of HTTP requests processed").namespace(ns)
    )
    .unwrap());

    let status_codes_total = register!(IntCounterVec::new(
      Opts::new("status_codes_total", "Response status codes").namespace(ns),
      &["status"]
    )
    .unwrap());

    let errors_total = register!(IntCounterVec::new(
      Opts::new("errors_total", "Errors by type").namespace(ns),
      &["type"]
    )
    .unwrap());

    let request_duration_seconds = register!(Histogram::with_opts(
      HistogramOpts::new("request_duration_seconds", "Full request latency in seconds")
        .namespace(ns)
        .buckets(vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0])
    )
    .unwrap());

    let request_span_duration_seconds = register!(HistogramVec::new(
      HistogramOpts::new(
        "request_span_duration_seconds",
        "Request latency by span in seconds"
      )
      .namespace(ns)
      .buckets(vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
      &["span"]
    )
    .unwrap());

    let buffer_size_bytes = register!(Histogram::with_opts(
      HistogramOpts::new("buffer_size_bytes", "Download buffer sizes in bytes")
        .namespace(ns)
        .buckets(vec![
          1_024.0,
          10_240.0,
          102_400.0,
          1_048_576.0,
          5_242_880.0,
          10_485_760.0,
          20_971_520.0,
        ])
    )
    .unwrap());

    let workers = register!(IntGauge::with_opts(
      Opts::new("workers", "Configured number of workers (max_concurrent_requests)").namespace(ns)
    )
    .unwrap());

    let requests_in_progress = register!(IntGauge::with_opts(
      Opts::new("requests_in_progress", "Number of requests currently in progress").namespace(ns)
    )
    .unwrap());

    let images_in_progress = register!(IntGauge::with_opts(
      Opts::new("images_in_progress", "Number of images currently being processed").namespace(ns)
    )
    .unwrap());

    let workers_utilization = register!(Gauge::with_opts(
      Opts::new(
        "workers_utilization",
        "Percentage of workers utilization (requests_in_progress / workers * 100)"
      )
      .namespace(ns)
    )
    .unwrap());

    let buffer_default_size_bytes = register!(Gauge::with_opts(
      Opts::new("buffer_default_size_bytes", "Calibrated default buffer size in bytes").namespace(ns)
    )
    .unwrap());

    let buffer_max_size_bytes = register!(Gauge::with_opts(
      Opts::new("buffer_max_size_bytes", "Calibrated maximum buffer size in bytes").namespace(ns)
    )
    .unwrap());

    let cache_hits_total = register!(IntCounterVec::new(
      Opts::new("cache_hits_total", "Cache hits by layer").namespace(ns),
      &["layer"]
    )
    .unwrap());

    let cache_misses_total = register!(IntCounterVec::new(
      Opts::new("cache_misses_total", "Cache misses by layer").namespace(ns),
      &["layer"]
    )
    .unwrap());

    let cache_memory_size_bytes = register!(Gauge::with_opts(
      Opts::new("cache_memory_size_bytes", "Current memory cache size in bytes").namespace(ns)
    )
    .unwrap());

    let cache_disk_size_bytes = register!(Gauge::with_opts(
      Opts::new("cache_disk_size_bytes", "Current disk cache size in bytes").namespace(ns)
    )
    .unwrap());

    let cache_entries = register!(IntGaugeVec::new(
      Opts::new("cache_entries", "Current cache entry count by layer").namespace(ns),
      &["layer"]
    )
    .unwrap());

    Arc::new(Self {
      registry,
      requests_total,
      status_codes_total,
      errors_total,
      request_duration_seconds,
      request_span_duration_seconds,
      buffer_size_bytes,
      workers,
      requests_in_progress,
      images_in_progress,
      workers_utilization,
      buffer_default_size_bytes,
      buffer_max_size_bytes,
      cache_hits_total,
      cache_misses_total,
      cache_memory_size_bytes,
      cache_disk_size_bytes,
      cache_entries,
    })
  }

  /// Update workers_utilization derived gauge.
  /// Call this after every change to requests_in_progress.
  pub fn update_utilization(&self) {
    let workers = self.workers.get();
    if workers > 0 {
      let utilization = self.requests_in_progress.get() as f64 / workers as f64 * 100.0;
      self.workers_utilization.set(utilization);
    }
  }
}
