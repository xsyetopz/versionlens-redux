pub(super) const NPM_DEPENDENCY_PATHS: &[&str] = &[
    "version",
    "packageManager",
    "dependencies",
    "devDependencies",
    "peerDependencies",
    "optionalDependencies",
    "overrides",
    "overrides.*",
    "jspm.dependencies",
    "jspm.devDependencies",
    "jspm.peerDependencies",
    "jspm.optionalDependencies",
    "pnpm.overrides",
    "pnpm.overrides.*",
];

pub(super) const COMPOSER_DEPENDENCY_PATHS: &[&str] = &["version", "require", "require-dev"];
pub(super) const DENO_DEPENDENCY_PATHS: &[&str] = &["imports"];
pub(super) const DOTNET_PROJECT_DEPENDENCY_PATHS: &[&str] = &[
    "dependencies",
    "frameworks.*.dependencies",
    "runtimes.*.dependencies",
];
pub(super) const DUB_DEPENDENCY_PATHS: &[&str] = &["dependencies", "versions"];

pub(super) fn dependency_paths<'a>(paths: &'a [&'a str], default: &'a [&'a str]) -> &'a [&'a str] {
    if paths.is_empty() { default } else { paths }
}
