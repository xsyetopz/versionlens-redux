use std::fmt;
use std::fmt::Write as _;

/// A validated GitHub owner/repository identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GithubRepository {
    owner: String,
    name: String,
}

impl GithubRepository {
    /// Parse an exact `owner/repository` identity using GitHub's path-safe grammar.
    pub fn parse(value: &str) -> Option<Self> {
        let (owner, name) = value.split_once('/')?;
        if value.matches('/').count() != 1
            || !valid_segment(owner)
            || !valid_segment(name)
            || owner == "."
            || owner == ".."
            || name == "."
            || name == ".."
        {
            return None;
        }
        Some(Self {
            owner: owner.to_owned(),
            name: name.to_owned(),
        })
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tags_url(&self) -> String {
        self.api_url("https://api.github.com/repos", "/tags")
    }

    /// Build an API URL with owner and repository encoded as separate path components.
    pub fn api_url(&self, base: &str, suffix: &str) -> String {
        format!(
            "{}/{}/{}{}",
            base.trim_end_matches('/'),
            encode_path_component(&self.owner),
            encode_path_component(&self.name),
            suffix
        )
    }
}

impl fmt::Display for GithubRepository {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}/{}", self.owner, self.name)
    }
}

fn valid_segment(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn encode_path_component(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(byte as char);
        } else {
            let _ = write!(encoded, "%{byte:02X}");
        }
    }
    encoded
}
