use super::*;

fn v(text: &str) -> Version {
    Version::parse(text).unwrap()
}

fn versions(list: &[&str]) -> Vec<Version> {
    list.iter().map(|t| v(t)).collect()
}

#[test]
fn orders_numbers_prereleases_and_tags() {
    assert!(v("1.10.0") > v("1.9.9"));
    assert!(v("1.0.0") > v("1.0.0-rc.10"));
    assert!(v("1.0.0-rc.10") > v("1.0.0-rc.9"));
    assert_eq!(v("1.0"), v("v1.0.0"));
    assert!(v("2.0rc1").is_prerelease());
    assert!(!v("33.0.0-jre").is_prerelease());
    assert_eq!(v("1.2.3.RELEASE").tag, "");
    assert_eq!(v("20-alpine3.19").variant(), "alpine");
}

#[test]
fn matches_requirement_dialects() {
    let cases = [
        ("^1.2.3", Bare::Exact, "1.9.0", true),
        ("^1.2.3", Bare::Exact, "2.0.0", false),
        ("^0.2.3", Bare::Exact, "0.3.0", false),
        ("~1.2.3", Bare::Exact, "1.2.9", true),
        ("~1.2.3", Bare::Exact, "1.3.0", false),
        ("~> 1.2", Bare::Exact, "1.9", true),
        ("~> 1.2.3", Bare::Exact, "1.3.0", false),
        ("~=1.4", Bare::Exact, "1.9.1", true),
        (">=1.0, <2.0", Bare::Exact, "1.5", true),
        (">= 1.0 < 2.0", Bare::Exact, "2.5", false),
        ("1.2.3", Bare::Caret, "1.4.0", true),
        ("1.2.3", Bare::Exact, "1.4.0", false),
        ("1.0", Bare::Min, "3.0", true),
        ("20", Bare::Prefix, "20.11.1", true),
        ("1.x", Bare::Exact, "1.4.0", true),
        ("==1.*", Bare::Exact, "2.0", false),
        ("^1 || ^2", Bare::Exact, "2.3.0", true),
        ("1.0.0 - 2.0.0", Bare::Exact, "2.0.0", true),
        ("[1.0,2.0)", Bare::Min, "2.0", false),
        ("[1.0]", Bare::Min, "1.0", true),
        ("*", Bare::Exact, "9.9.9", true),
        ("1.0.1+", Bare::Exact, "2.3.0", true),
        ("1.0.1+", Bare::Exact, "1.0.0", false),
        ("1.+", Bare::Exact, "1.9", true),
        ("1.+", Bare::Exact, "2.0", false),
    ];
    for (req, bare, version, expected) in cases {
        assert_eq!(
            Req::parse(req, bare).unwrap().matches(&v(version)),
            expected,
            "{req} vs {version}"
        );
    }
    assert!(Req::parse("workspace:*", Bare::Exact).is_none());
}

fn suggested(req: &str, list: &[Version]) -> Vec<String> {
    let plan = plan(&Req::parse(req, Bare::Exact).unwrap(), list, None, false).unwrap();
    plan.picks
        .iter()
        .map(|(label, v)| format!("{label} {}", v.raw))
        .collect()
}

#[test]
fn suggests_as_the_original_does() {
    let list = versions(&[
        "1.2.3", "1.2.9", "1.4.0", "2.0.0", "2.1.0", "3.0.0", "3.0.2",
    ]);
    // A range counts from the version it resolves to, and can bump up to it.
    assert_eq!(
        suggested("^1.2.3", &list),
        ["latest 3.0.2", "major 2.1.0", "bump 1.4.0"]
    );
    // A pin counts from itself.
    assert_eq!(
        suggested("1.2.3", &list),
        ["latest 3.0.2", "major 2.1.0", "minor 1.4.0", "patch 1.2.9"]
    );
    // A range already on the latest offers only to raise its floor to it.
    assert_eq!(suggested(">=2.0", &list), ["latest 3.0.2"]);
    assert_eq!(suggested("3.0.2", &list), Vec::<String>::new());
    // Nothing matches, so the latest is all there is to offer.
    assert_eq!(suggested("^9.0.0", &list), ["latest 3.0.2"]);
}

#[test]
fn plans_patch_minor_and_latest() {
    let list = versions(&["1.2.3", "1.2.9", "1.4.0", "2.0.0", "3.0.0-beta.1"]);
    let plan = plan(
        &Req::parse("^1.2.3", Bare::Exact).unwrap(),
        &list,
        None,
        false,
    )
    .unwrap();
    assert_eq!(plan.resolved.map(|v| v.raw).as_deref(), Some("1.4.0"));
    assert!(!plan.satisfied);
    let with_pre = super::plan(
        &Req::parse("^1.2.3", Bare::Exact).unwrap(),
        &list,
        None,
        true,
    )
    .unwrap();
    assert_eq!(with_pre.latest.raw, "3.0.0-beta.1");
}

#[test]
fn plan_keeps_variant_and_tagged_latest() {
    let list = versions(&["20-alpine", "22-alpine", "23", "22.1-slim"]);
    let plan = plan(
        &Req::parse("20-alpine", Bare::Prefix).unwrap(),
        &list,
        None,
        false,
    )
    .unwrap();
    assert_eq!(plan.latest.raw, "22-alpine");
    let npm = versions(&["1.0.0", "2.0.0", "3.0.0"]);
    let capped = super::plan(
        &Req::parse("^1.0.0", Bare::Exact).unwrap(),
        &npm,
        Some(&v("2.0.0")),
        false,
    )
    .unwrap();
    assert_eq!(capped.latest.raw, "2.0.0");
}

#[test]
fn renders_in_the_users_style() {
    assert_eq!(v("5.0.0").render("v4", false), "v5");
    assert_eq!(v("22.11.0-alpine").render("20-alpine", false), "22-alpine");
    assert_eq!(v("1.9.0").render("1.2", false), "1.9");
    assert_eq!(v("1.2.7").render("1.2", false), "1.2.7");
    assert_eq!(v("2.3.0").render("1.x", false), "2.x");
    assert_eq!(v("1.3.5").render("1.2", true), "1.3.5");
    assert_eq!(v("v1.2.3").render("1.0.0", true), "1.2.3");
    assert_eq!(level(Some(&v("1.2.3")), &v("1.3.0")), "minor");
}

#[test]
fn marks_yanked_and_deprecated_resolutions() {
    let req = Req::parse("^2.0.0", Bare::Exact).unwrap();
    let mut plan = plan(&req, &versions(&["1.0.0", "1.1.0"]), None, false).unwrap();
    plan.mark(&req, &versions(&["2.0.0", "2.1.0", "3.0.0"]), &[]);
    assert_eq!(plan.yanked.map(|y| y.raw), Some("2.1.0".into()));
    let req = Req::parse("^1.0.0", Bare::Exact).unwrap();
    let mut plan = plan_of(&req, &["1.0.0", "1.1.0"]);
    plan.mark(&req, &[], &versions(&["1.1.0"]));
    assert!(plan.deprecated && plan.yanked.is_none());
}

fn plan_of(req: &Req, list: &[&str]) -> Plan {
    plan(req, &versions(list), None, false).unwrap()
}

#[test]
fn lists_other_builds_of_the_resolved_version() {
    let req = Req::parse("1.0.0", Bare::Prefix).unwrap();
    let found = plan_of(&req, &["1.0.0", "1.0.0+build.2", "1.0.0-rc.1", "1.0.1"]);
    let mut all: Vec<String> = found.builds.into_iter().map(|b| b.raw).collect();
    all.extend(found.resolved.map(|r| r.raw));
    all.sort();
    assert_eq!(all, ["1.0.0", "1.0.0+build.2"]);
}

#[test]
fn increments_a_project_version_as_semver_inc_does() {
    let labels = |own: &str| -> Vec<String> {
        increments(own)
            .into_iter()
            .map(|(label, text)| format!("{label} {text}"))
            .collect()
    };
    assert_eq!(
        labels("1.2.3"),
        ["major 2.0.0", "minor 1.3.0", "patch 1.2.4"]
    );
    assert_eq!(
        labels("v0.9"),
        ["major v1.0.0", "minor v0.10.0", "patch v0.9.1"]
    );
    assert_eq!(labels("dev"), ["major 1.0.0", "minor 0.1.0", "patch 0.0.1"]);
    assert_eq!(
        labels("2.0.0-beta.1+build.5"),
        ["release 2.0.0", "prerelease 2.0.0-beta.2"]
    );
    assert_eq!(
        labels("1.0-SNAPSHOT"),
        ["release 1.0.0", "prerelease 1.0.0-SNAPSHOT.0"]
    );
    assert_eq!(labels("1.0.0-3"), ["release 1.0.0", "prerelease 1.0.0-4"]);
}
