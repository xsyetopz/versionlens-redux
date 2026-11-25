use super::*;

#[test]
fn takes_web_host_of_each_remote_url_form() {
    let cases = [
        (
            "https://gitlab.example.com/group/project.git",
            "gitlab.example.com",
        ),
        (
            "https://user@gitlab.example.com:8443/group/project",
            "gitlab.example.com:8443",
        ),
        (
            "git@gitlab.example.com:group/project.git",
            "gitlab.example.com",
        ),
        (
            "ssh://git@gitlab.example.com:2222/group/project.git",
            "gitlab.example.com",
        ),
        ("GitLab.Example.com:group/project", "gitlab.example.com"),
    ];
    for (url, expected) in cases {
        assert_eq!(host(url).as_deref(), Some(expected), "{url}");
    }
    assert_eq!(host("/srv/repos/project.git"), None);
}

#[test]
fn prefers_origin_over_first_remote() {
    let config = "[core]\n\tbare = false\n[remote \"upstream\"]\n\turl = https://gitlab.com/a/b.git\n[remote \"origin\"]\n\turl = git@git.example.org:c/d.git\n\tfetch = +refs/heads/*:refs/remotes/origin/*\n";
    let urls = remote_urls(config);
    assert_eq!(urls.len(), 2);
    assert_eq!(
        urls[1],
        ("origin".into(), "git@git.example.org:c/d.git".into())
    );
}
