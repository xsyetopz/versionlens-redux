use super::*;

const HEAD: &str = "f548e57e544e1ff5a4c46bf1e1b8685f8e4a348a";
const V410: &str = "8ade135a41bc03ea155e62e844d188df1ea18608";

/// Lines github.com/actions/checkout.git/info/refs advertised on 2026-11-25, as the `git-ref` row
/// captures them, with an annotated `stable` tag added to cover a peeled line.
fn checkout() -> Vec<String> {
    [
        &format!("{HEAD} HEAD"),
        "symref=HEAD:refs/heads/main",
        "4b3204e0bf8cc961de0423eafd557ab191aad421 refs/heads/Update-description",
        &format!("{HEAD} refs/heads/main"),
        "af513c7a016048ae468971c52ed77d9562c7c819 refs/tags/v1.0.0",
        &format!("{V410} refs/tags/v4.1.0"),
        "1111111111111111111111111111111111111111 refs/tags/stable",
        "2222222222222222222222222222222222222222 refs/tags/stable^{}",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

#[test]
fn branch_shows_its_head_commit() {
    let found = resolve("main", &checkout());

    assert_eq!(
        found,
        Ok(Ref::Branch {
            name: "main".into(),
            head: HEAD.into()
        })
    );
}

#[test]
fn tag_without_version_takes_peeled_commit() {
    let found = resolve("stable", &checkout());

    assert_eq!(
        found,
        Ok(Ref::Tag {
            name: "stable".into(),
            commit: "2222222222222222222222222222222222222222".into()
        })
    );
}

#[test]
fn pinned_sha_behind_default_branch_lags() {
    let found = resolve(V410, &checkout()).unwrap();

    assert_eq!(
        found,
        Ref::Pinned {
            branch: "main".into(),
            head: HEAD.into(),
            tag: Some("v4.1.0".into())
        }
    );
    assert!(found.lags(V410));
    assert_eq!(replacement(V410, HEAD), HEAD);
    assert_eq!(replacement("8ade135", HEAD), "f548e57");
}

#[test]
fn pinned_sha_at_branch_head_is_current() {
    let found = resolve("4b3204e0bf8cc961de0423eafd557ab191aad421", &checkout()).unwrap();

    assert!(matches!(&found, Ref::Pinned { branch, .. } if branch == "Update-description"));
    assert!(!found.lags("4b3204e0bf8cc961de0423eafd557ab191aad421"));
}

#[test]
fn unknown_reference_names_itself() {
    assert_eq!(
        resolve("no-such-branch", &checkout()),
        Err("no branch or tag no-such-branch".into())
    );
}
