use anyhow::Result;
use semver::Version;

pub fn compare_versions(current: &str, latest: &str) -> std::cmp::Ordering {
    let cur = Version::parse(current).expect("invalid current version");
    let lat = Version::parse(latest).expect("invalid latest version");
    cur.cmp(&lat)
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
}
