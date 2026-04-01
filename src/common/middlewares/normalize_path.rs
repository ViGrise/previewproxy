use tower_http::normalize_path::NormalizePathLayer;
use tracing::debug;

/// Middleware that normalizes paths.
///
/// Any trailing slashes from request paths will be removed. For example, a request with `/foo/`
/// will be changed to `/foo` before reaching the inner service.
#[tracing::instrument]
pub fn normalize_path_layer() -> NormalizePathLayer {
  debug!("path normalization (trim trailing slash) applied");
  NormalizePathLayer::trim_trailing_slash()
}
