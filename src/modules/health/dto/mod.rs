use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct HealthResponse {
  pub status: String,
  pub cache_memory_items: u64,
  pub cache_disk_bytes: u64,
  pub cache_disk_bytes_as_of: u64,
}
