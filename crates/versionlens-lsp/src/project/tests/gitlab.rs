//! GitLab components on the pipeline's own instance, named by the workspace's git remote.

use super::*;

/// Each dependency of `.gitlab-ci.yml` under `root`, as `(name, version, registry)`.
fn pipeline_deps(root: &Path) -> Vec<(String, String, &'static str)> {
    let path = root.join(".gitlab-ci.yml");
    let text = std::fs::read_to_string(&path).unwrap();
    let file = file_type_of(path.to_str().unwrap(), &text).unwrap();
    crate::project::extract(file, &path, &text, &env(root, &[]))
        .into_iter()
        .map(|d| (d.name, text[d.span].to_owned(), d.registry))
        .collect()
}

#[test]
fn ci_server_fqdn_component_resolves_on_remote_gitlab_instance() {
    // docs.gitlab.com/ci/components/ "Use a component", in a clone of a self-hosted project.
    let pipeline = "include:\n  - component: $CI_SERVER_FQDN/my-org/security-components/secret-detection@1.0.0\n";
    let root = project(
        "gitlab-self",
        &[
            (".gitlab-ci.yml", pipeline),
            (
                ".git/config",
                "[remote \"origin\"]\n\turl = git@gitlab.example.com:my-org/app.git\n",
            ),
        ],
    );
    assert_eq!(
        pipeline_deps(&root),
        [(
            "gitlab.example.com/my-org/security-components".to_owned(),
            "1.0.0".to_owned(),
            "gitlab-host"
        )]
    );
    for (name, config) in [
        (
            "gitlab-com",
            Some("[remote \"origin\"]\n\turl = https://gitlab.com/my-org/app.git\n"),
        ),
        (
            "gitlab-github",
            Some("[remote \"origin\"]\n\turl = https://github.com/my-org/app.git\n"),
        ),
        ("gitlab-none", None),
    ] {
        let mut files = vec![(".gitlab-ci.yml", pipeline)];
        files.extend(config.map(|c| (".git/config", c)));
        let root = project(name, &files);
        // A temporary folder may itself sit inside a clone; only one without a remote counts.
        if config.is_none() && git::remote_host(&root).is_some() {
            continue;
        }
        assert_eq!(
            pipeline_deps(&root),
            [(
                "my-org/security-components".to_owned(),
                "1.0.0".to_owned(),
                "gitlab"
            )],
            "{name}"
        );
    }
}

#[test]
fn worktree_takes_repository_remote() {
    let root = project(
        "gitlab-worktree",
        &[
            (
                "repo/.git/config",
                "[remote \"origin\"]\n\turl = https://code.example.net:8443/my-org/app.git\n",
            ),
            ("repo/.git/worktrees/wt/commondir", "../..\n"),
            ("wt/.git", "gitdir: ../repo/.git/worktrees/wt\n"),
            (
                "wt/.gitlab-ci.yml",
                "include:\n  - component: $CI_SERVER_FQDN/ci/deploy/run@2.1\n",
            ),
        ],
    );
    assert_eq!(
        pipeline_deps(&root.join("wt")),
        [(
            "code.example.net:8443/ci/deploy".to_owned(),
            "2.1".to_owned(),
            "gitlab-host"
        )]
    );
}
