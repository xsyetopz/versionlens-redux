//! Default dependency paths per manifest row, used when the user configures none.
//! The project's own version is not listed for npm or Cargo: it is not a dependency.

pub(super) const NONE: &[&str] = &[];
pub(super) const NPM: &[&str] = &[
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
/// The npm `version` is the project's own, never a dependency.
pub(super) const NPM_OWN_VERSION: &[&str] = &["version"];
/// The Cargo package version, its inherited `version.workspace = true` form,
/// and the workspace's shared package version.
pub(super) const CARGO_OWN_VERSION: &[&str] = &[
    "package.version",
    "package.version.workspace",
    "workspace.package.version",
];
pub(super) const COMPOSER: &[&str] = &[
    "version",
    "require",
    "require-dev",
    "conflict",
    "replace",
    "provide",
];
pub(super) const DENO: &[&str] = &["version", "imports", "scopes"];
pub(super) const JSR: &[&str] = &["version"];
pub(super) const DOTNET_PROJECT_JSON: &[&str] = &[
    "dependencies",
    "frameworks.*.dependencies",
    "runtimes.*.dependencies",
];
/// `dotnet tool` manifests list each tool as an object with a `version`.
pub(super) const DOTNET_TOOLS: &[&str] = &["tools"];
pub(super) const DUB: &[&str] = &["dependencies", "versions", "configurations.*.dependencies"];
pub(super) const CARGO: &[&str] = &[
    "package",
    "dependencies",
    "dependencies.*",
    "dev-dependencies",
    "dev-dependencies.*",
    "build-dependencies",
    "build-dependencies.*",
    "target.*.dependencies",
    "target.*.dependencies.*",
    "target.*.dev-dependencies",
    "target.*.dev-dependencies.*",
    "target.*.build-dependencies",
    "target.*.build-dependencies.*",
    "workspace.dependencies",
    "workspace.dependencies.*",
    "patch.*",
    "patch.*.*",
];
/// Cargo config files hold no dependencies, only overrides of them.
pub(super) const CARGO_CONFIG: &[&str] = &["patch.*", "patch.*.*"];
pub(super) const DOTNET_XML: &[&str] = &[
    "Project.Sdk",
    "Project.PropertyGroup.Version",
    "Project.PropertyGroup.AssemblyVersion",
    "Project.ItemGroup.GlobalPackageReference",
    "Project.ItemGroup.PackageReference",
    "Project.ItemGroup.PackageVersion",
    "Project.ItemGroup.DotNetCliToolReference",
    "packages.package",
    "paket.dependencies",
    "paket.references",
];
pub(super) const MAVEN: &[&str] = &[
    "project.version",
    "project.dependencies.dependency",
    "project.dependencyManagement.dependencies.dependency",
    "project.parent",
    "project.build.plugins.plugin",
    "project.build.pluginManagement.plugins.plugin",
    "project.profiles.profile.dependencies.dependency",
    "project.profiles.profile.dependencyManagement.dependencies.dependency",
    "project.profiles.profile.build.plugins.plugin",
    "project.profiles.profile.build.pluginManagement.plugins.plugin",
];
pub(super) const PNPM: &[&str] = &[
    "dependencies",
    "devDependencies",
    "peerDependencies",
    "optionalDependencies",
    "catalog",
    "catalogs.*.*",
    "overrides",
    "packageExtensions.*.dependencies",
    "packageExtensions.*.devDependencies",
    "packageExtensions.*.peerDependencies",
    "packageExtensions.*.optionalDependencies",
];
pub(super) const PUBSPEC: &[&str] = &[
    "version",
    "workspace",
    "dependencies",
    "dev_dependencies",
    "dependency_overrides",
];
pub(super) const PYPI: &[&str] = &[
    "project",
    "packages",
    "dev-packages",
    "tool.poetry.dependencies",
    "tool.poetry.dev-dependencies",
    "tool.poetry.group.*.dependencies",
    "project.optional-dependencies",
    "dependency-groups",
    "tool.uv.sources",
];
pub(super) const VCPKG: &[&str] = &["dependencies", "features.*.dependencies", "overrides"];
pub(super) const HAXELIB: &[&str] = &["dependencies"];
pub(super) const PUBSPEC_OVERRIDES: &[&str] = &["dependency_overrides"];
