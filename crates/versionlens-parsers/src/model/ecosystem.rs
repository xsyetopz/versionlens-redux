use serde::{Deserialize, Serialize};

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Ecosystem {
    Cargo,
    Composer,
    Deno,
    Dotnet,
    Docker,
    Dub,
    Go,
    Maven,
    Npm,
    Python,
    Pub,
    Ruby,
}

const ECOSYSTEM_CONFIG_NAMES: &[(&str, Ecosystem)] = &[
    ("cargo", Ecosystem::Cargo),
    ("composer", Ecosystem::Composer),
    ("deno", Ecosystem::Deno),
    ("dotnet", Ecosystem::Dotnet),
    ("docker", Ecosystem::Docker),
    ("dub", Ecosystem::Dub),
    ("go", Ecosystem::Go),
    ("golang", Ecosystem::Go),
    ("maven", Ecosystem::Maven),
    ("bun", Ecosystem::Npm),
    ("npm", Ecosystem::Npm),
    ("pnpm", Ecosystem::Npm),
    ("pypi", Ecosystem::Python),
    ("python", Ecosystem::Python),
    ("pub", Ecosystem::Pub),
    ("ruby", Ecosystem::Ruby),
];

const ECOSYSTEM_CONFIG_NAMESPACES: &[&str] = &[
    "cargo", "composer", "deno", "dotnet", "docker", "dub", "golang", "maven", "npm", "pypi",
    "pub", "ruby",
];

pub fn ecosystem_from_config_name(name: &str) -> Option<Ecosystem> {
    ECOSYSTEM_CONFIG_NAMES
        .iter()
        .find_map(|(candidate, ecosystem)| (*candidate == name).then_some(*ecosystem))
}

pub fn ecosystem_config_namespace(ecosystem: Ecosystem) -> &'static str {
    ECOSYSTEM_CONFIG_NAMESPACES[ecosystem as usize]
}
