use crate::common::errors::ProxyError;
use crate::modules::proxy::fetchable::Fetchable;
use std::path::PathBuf;

pub struct LocalSource {
  base_dir: PathBuf,
  max_bytes: u64,
}

impl LocalSource {
  pub async fn new(base_dir: &str, max_bytes: u64) -> Result<Self, String> {
    let base = tokio::fs::canonicalize(base_dir)
      .await
      .map_err(|e| format!("LOCAL_BASE_DIR '{}' canonicalize failed: {}", base_dir, e))?;
    Ok(Self {
      base_dir: base,
      max_bytes,
    })
  }
}

#[async_trait::async_trait]
impl Fetchable for LocalSource {
  async fn fetch(&self, url: &str) -> Result<(Vec<u8>, Option<String>), ProxyError> {
    let path_str = url.strip_prefix("local:/").unwrap_or(url);
    tracing::debug!(path = path_str, "local fetch start");
    let path = PathBuf::from(path_str);
    let resolved = self.base_dir.join(&path);

    let canonical = tokio::fs::canonicalize(&resolved).await.map_err(|e| {
      if e.kind() == std::io::ErrorKind::NotFound {
        ProxyError::UpstreamNotFound
      } else {
        ProxyError::InternalError(e.to_string())
      }
    })?;

    if !canonical.starts_with(&self.base_dir) {
      return Err(ProxyError::InvalidParams("path not allowed".to_string()));
    }

    let metadata = tokio::fs::metadata(&canonical)
      .await
      .map_err(|e| ProxyError::InternalError(e.to_string()))?;

    if !metadata.file_type().is_file() {
      return Err(ProxyError::InvalidParams("not a regular file".to_string()));
    }

    if metadata.len() > self.max_bytes {
      return Err(ProxyError::SourceTooLarge);
    }

    let bytes = tokio::fs::read(&canonical)
      .await
      .map_err(|e| ProxyError::InternalError(e.to_string()))?;

    tracing::debug!(path = path_str, bytes = bytes.len(), "local fetch complete");
    Ok((bytes, None))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_fetch_happy_path() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("image.jpg");
    let content = b"fake image bytes";
    std::fs::write(&file_path, content).unwrap();

    let source = LocalSource::new(dir.path().to_str().unwrap(), 1024 * 1024)
      .await
      .unwrap();

    let url = format!("local:/image.jpg");
    let (bytes, ct) = source.fetch(&url).await.unwrap();

    assert_eq!(bytes, content);
    assert!(ct.is_none());
  }

  #[tokio::test]
  async fn test_fetch_path_traversal() {
    let dir = tempfile::tempdir().unwrap();
    let outside_dir = tempfile::tempdir().unwrap();
    let outside_file = outside_dir.path().join("outside.txt");
    std::fs::write(&outside_file, b"outside content").unwrap();

    let source = LocalSource::new(dir.path().to_str().unwrap(), 1024 * 1024)
      .await
      .unwrap();

    // Build a traversal URL that resolves to the outside file.
    // dir.path() is e.g. /tmp/aaa, outside_dir.path() is e.g. /tmp/bbb,
    // so from inside dir we go up one level ("..") to /tmp, then into bbb/outside.txt.
    let outside_rel = outside_dir.path().file_name().unwrap().to_str().unwrap();
    let url = format!("local:/../{}/outside.txt", outside_rel);

    let result = source.fetch(&url).await;

    match result {
      Err(ProxyError::InvalidParams(msg)) => assert_eq!(msg, "path not allowed"),
      other => panic!("expected InvalidParams(path not allowed), got {:?}", other),
    }
  }

  #[tokio::test]
  async fn test_fetch_symlink_escape() {
    let dir = tempfile::tempdir().unwrap();
    let outside_dir = tempfile::tempdir().unwrap();
    let outside_file = outside_dir.path().join("secret.txt");
    std::fs::write(&outside_file, b"secret").unwrap();

    let symlink_path = dir.path().join("escape.jpg");
    std::os::unix::fs::symlink(&outside_file, &symlink_path).unwrap();

    let source = LocalSource::new(dir.path().to_str().unwrap(), 1024 * 1024)
      .await
      .unwrap();

    let result = source.fetch("local:/escape.jpg").await;

    match result {
      Err(ProxyError::InvalidParams(msg)) => assert_eq!(msg, "path not allowed"),
      other => panic!("expected InvalidParams(path not allowed), got {:?}", other),
    }
  }

  #[tokio::test]
  async fn test_fetch_missing_file() {
    let dir = tempfile::tempdir().unwrap();

    let source = LocalSource::new(dir.path().to_str().unwrap(), 1024 * 1024)
      .await
      .unwrap();

    let result = source.fetch("local:/nonexistent.jpg").await;

    match result {
      Err(ProxyError::UpstreamNotFound) => {}
      other => panic!("expected UpstreamNotFound, got {:?}", other),
    }
  }

  #[tokio::test]
  async fn test_fetch_directory_path() {
    let dir = tempfile::tempdir().unwrap();
    let subdir = dir.path().join("subdir");
    std::fs::create_dir(&subdir).unwrap();

    let source = LocalSource::new(dir.path().to_str().unwrap(), 1024 * 1024)
      .await
      .unwrap();

    let result = source.fetch("local:/subdir").await;

    match result {
      Err(ProxyError::InvalidParams(msg)) => assert_eq!(msg, "not a regular file"),
      other => panic!(
        "expected InvalidParams(not a regular file), got {:?}",
        other
      ),
    }
  }

  #[tokio::test]
  async fn test_fetch_file_too_large() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("big.bin");
    let content = vec![0u8; 100];
    std::fs::write(&file_path, &content).unwrap();

    let source = LocalSource::new(dir.path().to_str().unwrap(), 50)
      .await
      .unwrap();

    let result = source.fetch("local:/big.bin").await;

    match result {
      Err(ProxyError::SourceTooLarge) => {}
      other => panic!("expected SourceTooLarge, got {:?}", other),
    }
  }
}
