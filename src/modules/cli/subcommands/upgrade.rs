use anyhow::Result;
use semver::Version;

pub fn compare_versions(current: &str, latest: &str) -> std::cmp::Ordering {
    let cur = Version::parse(current).expect("invalid current version");
    let lat = Version::parse(latest).expect("invalid latest version");
    cur.cmp(&lat)
}

pub fn artifact_name() -> &'static str {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return "previewproxy-linux-x86_64";
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    return "previewproxy-linux-arm64";
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return "previewproxy-darwin-x86_64";
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return "previewproxy-darwin-arm64";
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return "previewproxy-windows-x86_64.exe";
    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    return "previewproxy-windows-arm64.exe";
    #[cfg(not(any(
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "windows", target_arch = "aarch64"),
    )))]
    compile_error!("unsupported target platform for self-upgrade");
}

pub fn download_url(tag: &str) -> String {
    format!(
        "https://github.com/vigrise/previewproxy/releases/download/v{}/{}",
        tag,
        artifact_name()
    )
}

pub async fn run_upgrade() -> Result<()> {
    todo!("upgrade not yet implemented")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn test_same_version() {
        assert_eq!(compare_versions("1.3.0", "1.3.0"), Ordering::Equal);
    }

    #[test]
    fn test_latest_is_newer() {
        assert_eq!(compare_versions("1.3.0", "1.4.0"), Ordering::Less);
    }

    #[test]
    fn test_current_is_newer() {
        assert_eq!(compare_versions("1.4.0", "1.3.0"), Ordering::Greater);
    }

    #[test]
    fn test_semver_ordering_correctness() {
        // Must be semver comparison, not lexicographic (1.9 < 1.10)
        assert_eq!(compare_versions("1.9.0", "1.10.0"), Ordering::Less);
    }

    #[test]
    fn test_artifact_name_nonempty() {
        assert!(!artifact_name().is_empty());
    }

    #[test]
    fn test_download_url_format() {
        let url = download_url("1.4.0");
        assert!(
            url.starts_with("https://github.com/vigrise/previewproxy/releases/download/v1.4.0/"),
            "unexpected url: {url}"
        );
        assert!(url.contains("previewproxy-"), "unexpected url: {url}");
    }
}
