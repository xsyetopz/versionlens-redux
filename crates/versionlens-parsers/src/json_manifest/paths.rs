pub(super) const NPM_DEPENDENCY_PATHS: &[&str] = &[
    "version",
    "packageManager",
    "devEngines.packageManager",
    "engines",
    "dependencies",
    "devDependencies",
    "peerDependencies",
    "optionalDependencies",
    "bundledDependencies",
    "bundleDependencies",
    "trustedDependencies",
    "resolutions",
    "overrides",
    "overrides.*",
    "jspm.dependencies",
    "jspm.devDependencies",
    "jspm.peerDependencies",
    "jspm.optionalDependencies",
    "pnpm.overrides",
    "pnpm.overrides.*",
    "pnpm.packageExtensions.*.dependencies",
    "pnpm.packageExtensions.*.devDependencies",
    "pnpm.packageExtensions.*.peerDependencies",
    "pnpm.packageExtensions.*.optionalDependencies",
    "catalog",
    "catalogs.*",
    "workspaces.catalog",
    "workspaces.catalogs.*",
];

pub(super) const COMPOSER_DEPENDENCY_PATHS: &[&str] = &[
    "version",
    "require",
    "require-dev",
    "conflict",
    "replace",
    "provide",
];
pub(super) const DENO_DEPENDENCY_PATHS: &[&str] = &["version", "imports", "scopes"];
pub(super) const JSR_DEPENDENCY_PATHS: &[&str] = &["version"];
pub(super) const DOTNET_PROJECT_DEPENDENCY_PATHS: &[&str] = &[
    "dependencies",
    "frameworks.*.dependencies",
    "runtimes.*.dependencies",
];
pub(super) const DUB_DEPENDENCY_PATHS: &[&str] =
    &["dependencies", "versions", "configurations.*.dependencies"];

pub(super) fn dependency_paths<'a>(paths: &'a [&'a str], default: &'a [&'a str]) -> &'a [&'a str] {
    if paths.is_empty() { default } else { paths }
}
