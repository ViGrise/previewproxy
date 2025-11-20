use axum::{extract::State, Json};

use crate::modules::health::{dto::HealthResponse, service};
use crate::modules::AppState;

pub async fn index(State(state): State<AppState>) -> Json<HealthResponse> {
  let result = service::index(
    state.cache.memory_item_count(),
    state.cache.disk_total_bytes(),
    state.cache.disk_total_bytes_as_of(),
  )
  .await;
  Json(result)
}
