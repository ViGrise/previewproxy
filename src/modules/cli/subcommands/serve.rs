use anyhow::Result;
use tracing::info;

#[tracing::instrument]
pub async fn run_serve() -> Result<()> {
  info!("server starting");
  todo!("serve not yet extracted")
}
