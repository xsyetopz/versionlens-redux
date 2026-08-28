use crate::support;
const PACKAGE_JSON_KEYS: &[&str] = &[
    "dependencies",
    "devDependencies",
    "peerDependencies",
    "optionalDependencies",
    "bundledDependencies",
    "bundleDependencies",
    "overrides",
    "packageManager",
    "jspm",
    "pnpm",
    "workspaces",
];

pub(super) fn looks_like_package_json(text: &str) -> bool {
    support::with_json_object(text, |root| {
        PACKAGE_JSON_KEYS.iter().any(|key| root.get(key).is_some())
    })
    .unwrap_or(false)
}
