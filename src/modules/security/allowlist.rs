use ipnet::IpNet;
use std::net::IpAddr;
use std::sync::LazyLock;
use tracing::debug;

/// Host-based allowlist for outbound fetch requests.
///
/// When constructed with an empty list (`open = true`), all hosts are allowed.
/// Entries support exact matches (`example.com`) and single-label wildcards
/// (`*.example.com` matches `www.example.com` but not `a.b.example.com`).
pub struct Allowlist {
  entries: Vec<String>,
  /// When true, all hosts are allowed (no list configured).
  open: bool,
}

impl Allowlist {
  pub fn new(entries: Vec<String>) -> Self {
    let open = entries.is_empty();
    Self { entries, open }
  }

  #[tracing::instrument(skip(self), fields(host = %host))]
  pub fn is_allowed(&self, host: &str) -> bool {
    if self.open {
      debug!(host = %host, result = "allowed", reason = "open_mode", "allowlist check");
      return true;
    }
    let allowed = self.entries.iter().any(|entry| matches_host(entry, host));
    debug!(host = %host, result = if allowed { "allowed" } else { "denied" }, "allowlist check");
    allowed
  }
}

fn matches_host(pattern: &str, host: &str) -> bool {
  let pattern = pattern.to_ascii_lowercase();
  let host = host.to_ascii_lowercase();
  if let Some(suffix) = pattern
    .strip_prefix('*')
    .filter(|suffix| suffix.starts_with('.'))
  {
    // Wildcard: host must be exactly one label + "." + suffix
    if let Some(label) = host.strip_suffix(suffix) {
      return !label.is_empty() && !label.contains('.');
    }
    false
  } else {
    pattern == host
  }
}

static PRIVATE_RANGES: LazyLock<Vec<IpNet>> = LazyLock::new(|| {
  [
    "::/128",
    "224.0.0.0/4",
    "240.0.0.0/4",
    "255.255.255.255/32",
    "192.0.0.0/24",
    "192.0.2.0/24",
    "198.51.100.0/24",
    "203.0.113.0/24",
    "ff00::/8",
    "64:ff9b::/96",
    "2002::/16",
    "2001:db8::/32",
    "10.0.0.0/8",
    "172.16.0.0/12",
    "192.168.0.0/16",
    "127.0.0.0/8",
    "169.254.0.0/16",
    "0.0.0.0/8",
    "100.64.0.0/10",
    "198.18.0.0/15",
    "::1/128",
    "fe80::/10",
    "fc00::/7",
  ]
  .into_iter()
  .map(|range| range.parse().expect("valid IP range"))
  .collect()
});

/// Returns true if `ip` falls within any RFC-1918, loopback, link-local, or
/// other non-routable range. Used to block SSRF attempts.
#[tracing::instrument]
pub fn is_private_ip(ip: IpAddr) -> bool {
  let ip = match ip {
    IpAddr::V6(ip) => ip.to_ipv4().map(IpAddr::V4).unwrap_or(IpAddr::V6(ip)),
    ip => ip,
  };
  let result = PRIVATE_RANGES.iter().any(|net| net.contains(&ip));
  debug!(ip = %ip, is_private = result, "private ip check");
  result
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn security_wildcard_boundary_and_case() {
    let list = Allowlist::new(vec!["*.example.com".into()]);
    assert!(!list.is_allowed("evilexample.com"));
    assert!(list.is_allowed("WWW.EXAMPLE.COM"));
  }

  #[test]
  fn security_non_routable_addresses() {
    for ip in [
      "::",
      "::ffff:127.0.0.1",
      "::192.168.1.1",
      "224.0.0.1",
      "240.0.0.1",
      "255.255.255.255",
      "192.0.0.1",
      "192.0.2.1",
      "198.51.100.1",
      "203.0.113.1",
      "ff02::1",
      "64:ff9b::808:808",
      "2002:808:808::",
      "2001:db8::1",
    ] {
      assert!(is_private_ip(ip.parse().unwrap()), "{ip}");
    }
  }

  #[test]
  fn test_exact_match() {
    let list = Allowlist::new(vec!["example.com".to_string()]);
    assert!(list.is_allowed("example.com"));
    assert!(!list.is_allowed("www.example.com"));
    assert!(!list.is_allowed("other.com"));
  }
  #[test]
  fn test_wildcard_single_label() {
    let list = Allowlist::new(vec!["*.example.com".to_string()]);
    assert!(list.is_allowed("www.example.com"));
    assert!(!list.is_allowed("a.b.example.com"));
    assert!(!list.is_allowed("example.com"));
  }
  #[test]
  fn test_empty_allows_all() {
    let list = Allowlist::new(vec![]);
    assert!(list.is_allowed("anything.com"));
    assert!(list.is_allowed("10.0.0.1")); // open mode ignores private IP check in allowlist
  }
  #[test]
  fn test_private_ip_always_blocked() {
    use std::net::IpAddr;
    assert!(is_private_ip("10.0.0.1".parse::<IpAddr>().unwrap()));
    assert!(is_private_ip("192.168.1.1".parse::<IpAddr>().unwrap()));
    assert!(is_private_ip("127.0.0.1".parse::<IpAddr>().unwrap()));
    assert!(is_private_ip("169.254.1.1".parse::<IpAddr>().unwrap()));
    assert!(is_private_ip("172.16.0.1".parse::<IpAddr>().unwrap()));
    assert!(!is_private_ip("8.8.8.8".parse::<IpAddr>().unwrap()));
    assert!(!is_private_ip("1.1.1.1".parse::<IpAddr>().unwrap()));
  }
}
