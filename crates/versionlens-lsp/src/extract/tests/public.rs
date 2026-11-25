//! Dependencies the fork's master read from public repositories that the rows first missed.

use super::{expect, sources};

#[test]
fn reads_npm_engines_runtimes_only() {
    // Arrange: a VS Code extension's `vscode` engine names no npm package.
    let text = r#"{"engines":{"node":">=20.9.0","vscode":"^1.90.0","pnpm":"^9"},"dependencies":{"a":"1.0.0"}}"#;

    // Act and Assert
    expect(
        "/w/package.json",
        text,
        &[
            ("node", ">=20.9.0", "npm"),
            ("pnpm", "^9", "npm"),
            ("a", "1.0.0", "npm"),
        ],
    );
}

#[test]
fn reads_cabal_entries_with_and_without_versions() {
    // Arrange: hackage.haskell.org cabal files list one entry per line in leading- or
    // trailing-comma style; `tested-with` names compilers, not packages.
    let text = "cabal-version: 2.4\nname: x\ntested-with: GHC ==8.6.5\nlibrary\n  build-depends:\n      base >=4.12 && <5\n    , text\n    , containers ^>=0.6\n  if flag(dev)\n    build-depends: mtl,\n                   transformers >=0.5,\n";

    // Act and Assert
    expect(
        "/w/x.cabal",
        text,
        &[
            ("base", ">=4.12", "hackage"),
            ("text", "", "hackage"),
            ("containers", "^>=0.6", "hackage"),
            ("mtl", "", "hackage"),
            ("transformers", ">=0.5", "hackage"),
        ],
    );
}

#[test]
fn reads_bazel_override_arguments_in_either_order() {
    // Arrange: github.com/bazelbuild/bazel MODULE.bazel puts `patches` before `version`.
    let text = "single_version_override(\n    module_name = \"grpc\",\n    patches = [\"//:a.patch\"],\n    version = \"1.66.0\",\n)\nsingle_version_override(version = \"2.0\", module_name = \"zlib\")\n";

    // Act and Assert
    expect(
        "/w/MODULE.bazel",
        text,
        &[("grpc", "1.66.0", "bazel"), ("zlib", "2.0", "bazel")],
    );
}

#[test]
fn resolves_msbuild_property_from_same_file() {
    // Arrange: learn.microsoft.com/visualstudio/msbuild/msbuild-properties
    let text = "<Project><PropertyGroup><AVersion>1.2.3</AVersion></PropertyGroup><ItemGroup><PackageVersion Include=\"A\" Version=\"$(AVersion)\" /></ItemGroup></Project>";

    // Act and Assert
    expect(
        "/w/Directory.Packages.props",
        text,
        &[("A", "1.2.3", "nuget")],
    );
}

#[test]
fn labels_property_defined_elsewhere() {
    // Arrange: a parent POM or Directory.Build.props defines the property.
    let csproj = "<Project><ItemGroup><PackageReference Include=\"A\" Version=\"$(AVersion)\" /></ItemGroup></Project>";
    let pom = "<project><dependencies><dependency><groupId>g</groupId><artifactId>a</artifactId><version>${a.version}</version></dependency></dependencies></project>";

    // Act
    let nuget = sources("/w/x.csproj", csproj);
    let maven = sources("/w/pom.xml", pom);

    // Assert
    assert_eq!(nuget, [("A".to_owned(), "version property".to_owned())]);
    assert_eq!(maven, [("g:a".to_owned(), "version property".to_owned())]);
}

#[test]
fn labels_gradle_version_variable() {
    // Arrange: square/okhttp's build.gradle.kts takes `alpnBootVersion` from buildSrc.
    let text = "dependencies.create(\"org.mortbay.jetty.alpn:alpn-boot:$alpnBootVersion\")\nimplementation(\"g:b:${bVersion}\")\n";

    // Act
    let deps = sources("/w/build.gradle.kts", text);

    // Assert
    assert_eq!(
        deps,
        [
            (
                "org.mortbay.jetty.alpn:alpn-boot".to_owned(),
                "version property".to_owned()
            ),
            ("g:b".to_owned(), "version property".to_owned()),
        ]
    );
}
