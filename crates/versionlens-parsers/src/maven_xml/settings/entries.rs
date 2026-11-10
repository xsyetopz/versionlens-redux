/// A Maven repository URL with its protocol.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MavenRepository {
    /// The repository URL.
    pub url: String,
    /// The URL protocol, such as `https:`.
    pub protocol: String,
}

// The Debug output carries raw tokens and feeds cache-scope hashing, so it must
// stay as is and must never be logged.
/// The credentials that a Maven `settings.xml` holds for a repository.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MavenAuthEntry {
    /// The registry URL.
    pub registry: String,
    /// The `Authorization` header value to send.
    pub header_value: String,
}

/// A mirror of a Maven `settings.xml`.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MavenMirror {
    /// The repository ID.
    pub id: String,
    /// The repository IDs that the mirror stands in for.
    pub mirror_of: String,
    /// The mirror URL.
    pub url: String,
}

/// A Maven repository with its ID.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MavenNamedRepository {
    /// The repository ID.
    pub id: String,
    /// The repository URL.
    pub url: String,
}
