use crate::modules::security::encryption;
use anyhow::{Result, anyhow};

pub fn run_encrypt_url(url: &str, key_hex: &str) -> Result<String> {
  let key = hex::decode(key_hex)
    .map_err(|e| anyhow!("invalid hex key: {e}"))?;
  encryption::encrypt(&key, url).map_err(|e| anyhow!("{e}"))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::modules::security::encryption;

  #[test]
  fn test_encrypt_url_produces_decryptable_blob() {
    let key_hex = "1eb5b0e971ad7f45324c1bb15c947cb207c43152fa5c6c7f35c4f36e0c18e0f1";
    let url = "https://example.com/images/photo.jpg";
    let blob = run_encrypt_url(url, key_hex).unwrap();
    let key = hex::decode(key_hex).unwrap();
    let decrypted = encryption::decrypt(&key, &blob).unwrap();
    assert_eq!(decrypted, url);
  }

  #[test]
  fn test_encrypt_url_invalid_hex_key_errors() {
    let result = run_encrypt_url("https://example.com/img.jpg", "not_hex");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("invalid hex"));
  }

  #[test]
  fn test_encrypt_url_wrong_key_length_errors() {
    // 10 hex chars = 5 bytes, invalid for AES
    let result = run_encrypt_url("https://example.com/img.jpg", "deadbeefdeadbeef1234");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("invalid_key_length"));
  }
}
