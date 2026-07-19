use super::ecosystem_for_manifest;
use crate::Ecosystem::{
    Cargo, Composer, Conan, Cpan, Cpp, Cran, Deno, Docker, Dotnet, Dub, Go, Hackage, Haxelib, Hex,
    Julia, LuaRocks, Maven, Nim, Npm, Opam as OpamEcosystem, Pub, Python, Ruby, Swift, Vcpkg, Zig,
};
use crate::ManifestKind::{
    BazelWorkspace, Cabal, CabalProject, CargoToml, ClojureDepsEdn, Cmake, ComposerJson,
    ConanfilePy, ConanfileTxt, Cpanfile, DenoImportMapJson, DenoJson, DockerComposeYaml,
    Dockerfile, DotnetProjectJson, DotnetXml, DubJson, DubSdl, DuneProject, Gemfile, GleamToml,
    GoMod, GradleBuild, GradleSettings, GradleVersionCatalogToml, HaxelibJson, JsrJson,
    JuliaManifestToml, JuliaProjectToml, LeiningenProjectClj, LuaRockspec, MavenPomXml, MesonWrap,
    MixExs, Nimble, NpmPackageJson, NpmPackageJson5, NpmPackageYaml, Opam, PaketDependencies,
    PaketReferences, PnpmYaml, PubspecOverridesYaml, PubspecYaml, PythonPipfile,
    PythonPyprojectToml, PythonRequirementsTxt, RDescription, RebarConfig, RenvLock, RubyGemspec,
    SbtBuild, StackYaml, SwiftPackage, Unknown, VcpkgJson, VersionLensMultiRegistries, XmakeLua,
    ZigBuildZon,
};

#[test]
fn maps_manifest_kinds_to_ecosystems() {
    let cases = [
        (CargoToml, Cargo),
        (ComposerJson, Composer),
        (DenoJson, Deno),
        (DenoImportMapJson, Deno),
        (JsrJson, Deno),
        (DotnetProjectJson, Dotnet),
        (DotnetXml, Dotnet),
        (PaketDependencies, Dotnet),
        (PaketReferences, Dotnet),
        (DockerComposeYaml, Docker),
        (Dockerfile, Docker),
        (DubJson, Dub),
        (DubSdl, Dub),
        (Gemfile, Ruby),
        (RubyGemspec, Ruby),
        (GoMod, Go),
        (MavenPomXml, Maven),
        (GradleBuild, Maven),
        (GradleSettings, Maven),
        (GradleVersionCatalogToml, Maven),
        (SbtBuild, Maven),
        (ClojureDepsEdn, Maven),
        (LeiningenProjectClj, Maven),
        (MixExs, Hex),
        (RebarConfig, Hex),
        (GleamToml, Hex),
        (Opam, OpamEcosystem),
        (DuneProject, OpamEcosystem),
        (Cabal, Hackage),
        (CabalProject, Hackage),
        (StackYaml, Hackage),
        (JuliaProjectToml, Julia),
        (JuliaManifestToml, Julia),
        (RDescription, Cran),
        (RenvLock, Cran),
        (ConanfileTxt, Conan),
        (ConanfilePy, Conan),
        (VcpkgJson, Vcpkg),
        (Cmake, Cpp),
        (XmakeLua, Cpp),
        (MesonWrap, Cpp),
        (BazelWorkspace, Cpp),
        (SwiftPackage, Swift),
        (ZigBuildZon, Zig),
        (Nimble, Nim),
        (LuaRockspec, LuaRocks),
        (Cpanfile, Cpan),
        (HaxelibJson, Haxelib),
        (NpmPackageJson, Npm),
        (NpmPackageJson5, Npm),
        (NpmPackageYaml, Npm),
        (PnpmYaml, Npm),
        (PythonPipfile, Python),
        (PythonPyprojectToml, Python),
        (PythonRequirementsTxt, Python),
        (PubspecOverridesYaml, Pub),
        (PubspecYaml, Pub),
    ];

    for (kind, ecosystem) in cases {
        assert_eq!(ecosystem_for_manifest(kind), Some(ecosystem));
    }
}

#[test]
fn ignores_non_dependency_manifest_kinds() {
    assert_eq!(ecosystem_for_manifest(Unknown), None);
    assert_eq!(ecosystem_for_manifest(VersionLensMultiRegistries), None);
}
