use super::affecting;
use crate::version::Version;

const ADVISORY: &str = r#"---
identifier: "CVE-2020-1967"
package_slug: "conan/openssl"
affected_range: ">=1.1.1d <=1.1.1f"
fixed_versions:
- "1.1.1g"
"#;

#[test]
fn reports_advisory_only_inside_its_range() {
    let at = |v: &str| affecting(ADVISORY, &Version::parse(v).unwrap());
    assert_eq!(at("1.1.1e").as_deref(), Some("CVE-2020-1967"));
    assert_eq!(at("1.1.1g"), None);
    assert_eq!(at("1.1.1c"), None);
}

#[test]
fn reads_maven_style_ranges() {
    let advisory = "identifier: \"CVE-1\"\naffected_range: \"(,2.0)\"\n";
    assert!(affecting(advisory, &Version::parse("1.9").unwrap()).is_some());
    assert!(affecting(advisory, &Version::parse("2.0").unwrap()).is_none());
}
