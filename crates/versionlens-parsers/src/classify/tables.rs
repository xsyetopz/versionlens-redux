use versionlens_model::ManifestKind;

use super::uri::file_name;
use versionlens_model::ManifestKind::{
    AnsibleGalaxyRequirementsYaml, BazelModule, BazelWorkspace, CabalProject, CargoToml,
    ClojureDepsEdn, Cmake, CocoaPodsPodfile, ComposerJson, ConanfilePy, ConanfileTxt, Cpanfile,
    DenoImportMapJson, DenoJson, DotnetProjectJson, DotnetXml, DubJson, DubSdl, DuneProject,
    Gemfile, GleamToml, GoMod, GradleBuild, GradleSettings, GradleVersionCatalogToml, HaxelibJson,
    HelmChartYaml, JsrJson, JuliaManifestToml, JuliaProjectToml, KustomizationYaml,
    LeiningenProjectClj, MavenPomXml, MixExs, NixFlake, NpmPackageJson, NpmPackageJson5,
    NpmPackageYaml, Opam, PaketDependencies, PaketReferences, PubspecOverridesYaml, PubspecYaml,
    RDescription, RebarConfig, RenvLock, SbtBuild, StackYaml, SwiftPackage, VcpkgJson, XmakeLua,
    ZigBuildZon,
};

pub(super) const EARLY_FILE_MANIFESTS: &[(&str, ManifestKind)] = &[
    ("Cargo.toml", CargoToml),
    ("composer.json", ComposerJson),
    ("deno.json", DenoJson),
    ("deno.jsonc", DenoJson),
    ("import_map.json", DenoImportMapJson),
    ("jsr.json", JsrJson),
    ("jsr.jsonc", JsrJson),
    ("packages.config", DotnetXml),
    ("paket.dependencies", PaketDependencies),
    ("paket.references", PaketReferences),
    ("project.json", DotnetProjectJson),
    ("libs.versions.toml", GradleVersionCatalogToml),
    ("build.gradle", GradleBuild),
    ("build.gradle.kts", GradleBuild),
    ("settings.gradle", GradleSettings),
    ("settings.gradle.kts", GradleSettings),
    ("build.sbt", SbtBuild),
    ("deps.edn", ClojureDepsEdn),
    ("project.clj", LeiningenProjectClj),
    ("pom.xml", MavenPomXml),
    ("mix.exs", MixExs),
    ("rebar.config", RebarConfig),
    ("gleam.toml", GleamToml),
    ("opam", Opam),
    ("dune-project", DuneProject),
    ("cabal.project", CabalProject),
    ("stack.yaml", StackYaml),
    ("stack.yml", StackYaml),
    ("Project.toml", JuliaProjectToml),
    ("Manifest.toml", JuliaManifestToml),
    ("DESCRIPTION", RDescription),
    ("renv.lock", RenvLock),
    ("conanfile.txt", ConanfileTxt),
    ("conanfile.py", ConanfilePy),
    ("vcpkg.json", VcpkgJson),
    ("CMakeLists.txt", Cmake),
    ("xmake.lua", XmakeLua),
    ("WORKSPACE", BazelWorkspace),
    ("WORKSPACE.bazel", BazelWorkspace),
    ("Package.swift", SwiftPackage),
    ("build.zig.zon", ZigBuildZon),
    ("cpanfile", Cpanfile),
    ("haxelib.json", HaxelibJson),
    ("Chart.yaml", HelmChartYaml),
    ("requirements.yml", AnsibleGalaxyRequirementsYaml),
    ("requirements.yaml", AnsibleGalaxyRequirementsYaml),
    ("MODULE.bazel", BazelModule),
    ("flake.nix", NixFlake),
    ("kustomization.yaml", KustomizationYaml),
    ("kustomization.yml", KustomizationYaml),
    ("Kustomization", KustomizationYaml),
    ("Podfile", CocoaPodsPodfile),
];

pub(super) const LATE_FILE_MANIFESTS: &[(&str, ManifestKind)] = &[
    ("dub.json", DubJson),
    ("dub.selections.json", DubJson),
    ("dub.sdl", DubSdl),
    ("Gemfile", Gemfile),
    ("go.mod", GoMod),
    ("go.work", GoMod),
    ("package.json", NpmPackageJson),
    ("package.json5", NpmPackageJson5),
    ("package.yaml", NpmPackageYaml),
    ("package.yml", NpmPackageYaml),
];

pub(super) const PUBSPEC_FILE_MANIFESTS: &[(&str, ManifestKind)] = &[
    ("pubspec_overrides.yaml", PubspecOverridesYaml),
    ("pubspec.yaml", PubspecYaml),
    ("pubspec.yml", PubspecYaml),
];

pub(super) fn exact_file_manifest(
    uri: &str,
    manifests: &[(&str, ManifestKind)],
) -> Option<ManifestKind> {
    let name = file_name(uri)?;
    manifests
        .iter()
        .find_map(|(expected, kind)| name.eq_ignore_ascii_case(expected).then_some(*kind))
}
