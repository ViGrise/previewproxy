use axum::{Json, extract::State};
use tracing::info;

use crate::modules::AppState;
use crate::modules::health::{dto::HealthResponse, service};

#[tracing::instrument(skip(state))]
pub async fn index(State(state): State<AppState>) -> Json<HealthResponse> {
  info!("health check called");
  let result = service::index(
    state.cache.memory_item_count(),
    state.cache.disk_total_bytes(),
    state.cache.disk_total_bytes_as_of(),
  )
  .await;
  info!(status = %result.status, "health check response");
  Json(result)
}
