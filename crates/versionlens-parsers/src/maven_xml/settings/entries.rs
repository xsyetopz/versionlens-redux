#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MavenRepository {
    pub url: String,
    pub protocol: String,
}

// The Debug output carries raw tokens and feeds cache-scope hashing, so it must
// stay as is and must never be logged.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MavenAuthEntry {
    pub registry: String,
    pub header_value: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MavenMirror {
    pub id: String,
    pub mirror_of: String,
    pub url: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MavenNamedRepository {
    pub id: String,
    pub url: String,
}
