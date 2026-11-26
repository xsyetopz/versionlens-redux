//! A branch or commit that a CI file pins in place of a tag, read against the references its
//! repository advertises. Git's smart HTTP answers `info/refs?service=git-upload-pack` with one
//! `<sha> <ref>` line per branch and tag, a peeled `<ref>^{}` line for each annotated tag, and a
//! `symref=HEAD:refs/heads/<branch>` capability naming the default branch
//! (git-scm.com/docs/http-protocol, "Smart Clients", and git-scm.com/docs/protocol-capabilities,
//! "symref"). GitHub, GitLab, and Forgejo all serve it, so one catalog row, `git-ref`, lists them.

/// The registry whose version list is a repository's advertised references.
pub(crate) const GIT_REF: &str = "git-ref";

/// Where a pinned reference stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Ref {
    /// A branch, which moves to its head commit on every run.
    Branch { name: String, head: String },
    /// A tag that names no version, such as `stable`, at its commit.
    Tag { name: String, commit: String },
    /// A commit SHA, measured against the head of the branch it is the head of, else the default
    /// branch, with the tag that points at it, if any.
    Pinned {
        branch: String,
        head: String,
        tag: Option<String>,
    },
}

impl Ref {
    /// A pinned commit that is not its branch's head: every commit before the head lags it.
    pub(crate) fn lags(&self, pin: &str) -> bool {
        matches!(self, Self::Pinned { head, .. } if !same_commit(pin, head))
    }
}

/// The first seven hex digits, as `git log --oneline` abbreviates a commit.
pub(crate) fn short(sha: &str) -> &str {
    &sha[..sha.len().min(7)]
}

/// The head SHA written as long as the pin it replaces, since a full-length pin stays full
/// (GitHub Actions takes only a full SHA as immutable:
/// docs.github.com/en/actions/reference/security/secure-use, "Using third-party actions").
pub(crate) fn replacement(pin: &str, head: &str) -> String {
    head[..pin.len().clamp(7, head.len())].to_owned()
}

fn same_commit(pin: &str, sha: &str) -> bool {
    pin.len() >= 7 && sha.starts_with(&pin.to_ascii_lowercase())
}

fn is_sha(req: &str) -> bool {
    (7..=40).contains(&req.len()) && req.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Reads `req`, a branch, a tag, or a commit, against `refs`, the `git-ref` row's captures:
/// `<sha> HEAD`, `<sha> refs/heads/<name>`, `<sha> refs/tags/<name>[^{}]`, and
/// `symref=HEAD:refs/heads/<name>`.
///
/// # Errors
/// Names a reference the repository does not advertise.
pub(crate) fn resolve(req: &str, refs: &[String]) -> Result<Ref, String> {
    let mut head = None;
    let mut default = None;
    let mut branches = Vec::new();
    let mut tags: Vec<(&str, &str)> = Vec::new();
    for line in refs {
        if let Some(name) = line.strip_prefix("symref=HEAD:refs/heads/") {
            default = Some(name);
            continue;
        }
        let Some((sha, name)) = line.split_once(' ') else {
            continue;
        };
        if name == "HEAD" {
            head = Some(sha);
        } else if let Some(branch) = name.strip_prefix("refs/heads/") {
            branches.push((branch, sha));
        } else if let Some(tag) = name.strip_prefix("refs/tags/") {
            // A peeled line follows its annotated tag and names the commit the tag points at.
            let tag = tag.strip_suffix("^{}").unwrap_or(tag);
            tags.retain(|(t, _)| *t != tag);
            tags.push((tag, sha));
        }
    }
    let req = req.strip_prefix("refs/heads/").unwrap_or(req);
    if let Some((name, sha)) = branches.iter().find(|(b, _)| *b == req) {
        return Ok(Ref::Branch {
            name: (*name).to_owned(),
            head: (*sha).to_owned(),
        });
    }
    let tag = req.strip_prefix("refs/tags/").unwrap_or(req);
    if let Some((name, sha)) = tags.iter().find(|(t, _)| *t == tag) {
        return Ok(Ref::Tag {
            name: (*name).to_owned(),
            commit: (*sha).to_owned(),
        });
    }
    if !is_sha(req) {
        return Err(format!("no branch or tag {req}"));
    }
    // The branch the commit heads, else the default one that `HEAD` names or points at.
    let (branch, sha) = branches
        .iter()
        .find(|(_, sha)| same_commit(req, sha))
        .or_else(|| branches.iter().find(|(b, _)| Some(*b) == default))
        .or_else(|| branches.iter().find(|(_, sha)| Some(*sha) == head))
        .ok_or_else(|| "advertises no default branch".to_owned())?;
    Ok(Ref::Pinned {
        branch: (*branch).to_owned(),
        head: (*sha).to_owned(),
        tag: tags
            .iter()
            .find(|(_, sha)| same_commit(req, sha))
            .map(|(t, _)| (*t).to_owned()),
    })
}

#[cfg(test)]
mod tests;
