//! Package-manager forms the fork's master fixtures hold and the shared rows now read: npm
//! aliases, bundled names in list and object form, override selectors, pub's blank, commented,
//! hosted and SDK dependencies, Composer's `php` platform package, Poetry's `python`, gems
//! from GitHub without a ref, Deno imports without a version and deno.land URLs, and dub
//! repositories and inline sub-packages.

use super::master::expect;

#[test]
fn reads_npm_aliases_under_their_target_name() {
    // Arrange
    let text = r#"{"dependencies": {"types-react": "npm:@types/react@^19.0.0", "lodash4": "npm:lodash@4.17.21", "bare": "npm:left-pad"}}"#;

    // Act and Assert
    expect(
        "package.json",
        text,
        &[
            ("@types/react", "^19.0.0", "npm"),
            ("lodash", "4.17.21", "npm"),
            ("left-pad", "", "npm"),
        ],
    );
}

#[test]
fn reads_bundled_names_in_list_and_object_form() {
    // Arrange
    let list = r#"{"bundleDependencies": ["alpha"], "bundledDependencies": ["beta"], "trustedDependencies": ["gamma"]}"#;
    let object = r#"{"bundleDependencies": {"delta": true}}"#;

    // Act and Assert
    expect(
        "package.json",
        list,
        &[
            ("alpha", "", "npm"),
            ("beta", "", "npm"),
            ("gamma", "", "npm"),
        ],
    );
    expect("package.json", object, &[("delta", "", "npm")]);
}

#[test]
fn reads_override_selectors_as_package_names() {
    // Arrange
    let npm = r#"{"overrides": {"foo": {".": "1.0.0", "bar": "2.0.0"}, "axios@<1": "0.28.0", "@scope/pkg@1.2.3": "1.2.4"}}"#;
    let yarn =
        r#"{"resolutions": {"**/webpack/terser": "5.0.0", "@babel/core/@babel/parser": "7.0.0"}}"#;
    let pnpm = r#"{"pnpm": {"overrides": {"express>cookie": "0.7.0", "qs@<6.10": "6.10.3"}}}"#;

    // Act and Assert
    expect(
        "package.json",
        npm,
        &[
            ("foo", "1.0.0", "npm"),
            ("bar", "2.0.0", "npm"),
            ("axios", "0.28.0", "npm"),
            ("@scope/pkg", "1.2.4", "npm"),
        ],
    );
    expect(
        "package.json",
        yarn,
        &[
            ("terser", "5.0.0", "npm"),
            ("@babel/parser", "7.0.0", "npm"),
        ],
    );
    expect(
        "package.json",
        pnpm,
        &[("cookie", "0.7.0", "npm"), ("qs", "6.10.3", "npm")],
    );
}

#[test]
fn reads_pub_blank_commented_hosted_and_sdk_dependencies() {
    // Arrange
    let text = "dependencies:\n  equatable:\n  bloc: # state\n  hosted_dep:\n    hosted: https://pub.example.com\n  pinned:\n    hosted: https://pub.example.com\n    version: ^1.0.0\n  flutter:\n    sdk: flutter\ndev_dependencies:\n  lints: ^5.0.0\n";

    // Act and Assert
    expect(
        "pubspec.yaml",
        text,
        &[
            ("equatable", "", "pub"),
            ("bloc", "", "pub"),
            ("hosted_dep", "", "pub"),
            ("pinned", "^1.0.0", "pub"),
            ("flutter", "flutter", "source"),
            ("lints", "^5.0.0", "pub"),
        ],
    );
}

#[test]
fn reads_composer_php_as_runtime() {
    // Arrange
    let text = r#"{"require": {"php": "^8.3", "monolog/monolog": "^3.0"}, "require-dev": {"php": ">=8.1"}}"#;

    // Act and Assert
    expect(
        "composer.json",
        text,
        &[
            ("php", "^8.3", "php"),
            ("monolog/monolog", "^3.0", "composer"),
            ("php", ">=8.1", "php"),
        ],
    );
}

#[test]
fn reads_poetry_python_in_string_and_table_form() {
    // Arrange
    let text = "[tool.poetry.dependencies]\npython = \"^3.12\"\nclick = \"^8.1\"\n\n[tool.poetry.group.dev.dependencies.python]\nversion = \"^3.13\"\n";

    // Act and Assert
    expect(
        "pyproject.toml",
        text,
        &[
            ("python", "^3.12", "python"),
            ("click", "^8.1", "pypi"),
            ("python", "^3.13", "python"),
        ],
    );
}

#[test]
fn reads_github_gems_without_ref_from_default_branch() {
    // Arrange
    let text = "gem \"devise\", github: \"heartcombo/devise\"\ngem \"rspec-core\", :github => \"rspec/rspec-core\", require: false\n";

    // Act and Assert
    expect(
        "Gemfile",
        text,
        &[
            ("heartcombo/devise", "heartcombo/devise", "source"),
            ("rspec/rspec-core", "rspec/rspec-core", "source"),
        ],
    );
}

#[test]
fn reads_deno_imports_without_version_and_deno_land_urls() {
    // Arrange
    let text = r#"{"imports": {"@std/assert": "jsr:@std/assert", "chalk": "npm:chalk", "empty": "jsr:@std/assert@", "std": "https://deno.land/std/mod.ts", "oak": "https://deno.land/x/oak@v12.6.1/mod.ts"}}"#;

    // Act and Assert
    expect(
        "deno.json",
        text,
        &[
            ("@std/assert", "", "jsr"),
            ("chalk", "", "npm"),
            ("std", "", "deno-land"),
            ("oak", "v12.6.1", "deno-land"),
        ],
    );
}

#[test]
fn reads_dub_repositories_and_inline_sub_packages() {
    // Arrange
    let text = r#"{"name": "root", "dependencies": {"remote": {"repository": "git+https://github.com/o/r.git", "version": "6fd8e2dd19de59755b5531ccd7e481fd1df8967d"}}, "subPackages": ["./selector", {"name": "sub", "dependencies": {"standardpaths": "~>0.2.1"}, "configurations": [{"name": "c", "dependencies": {"x": "1.0.0"}}]}]}"#;

    // Act and Assert
    expect(
        "dub.json",
        text,
        &[
            ("remote", "git+https://github.com/o/r.git", "source"),
            ("standardpaths", "~>0.2.1", "dub"),
            ("x", "1.0.0", "dub"),
        ],
    );
}

#[test]
fn reads_pom_references_to_properties_and_inherited_coordinates() {
    // Arrange
    let text = "<project>\n  <parent>\n    <groupId>org.parent</groupId>\n    <artifactId>parent-pom</artifactId>\n    <version>3.4.5</version>\n  </parent>\n  <artifactId>child-app</artifactId>\n  <properties>\n    <example.group>com.example</example.group>\n  </properties>\n  <dependencies>\n    <dependency>\n      <groupId>${project.groupId}</groupId>\n      <artifactId>runtime</artifactId>\n      <version>${project.parent.version}</version>\n    </dependency>\n    <dependency>\n      <groupId> ${example.group} </groupId>\n      <artifactId>${project.artifactId}</artifactId>\n      <version>${project.version}</version>\n    </dependency>\n  </dependencies>\n</project>\n";

    // Act and Assert
    expect(
        "pom.xml",
        text,
        &[
            ("org.parent:parent-pom", "3.4.5", "maven"),
            ("org.parent:runtime", "3.4.5", "maven"),
            ("com.example:child-app", "3.4.5", "maven"),
        ],
    );
}

#[test]
fn reads_paket_lines_without_version() {
    // Arrange
    let text = "source https://api.nuget.org/v3/index.json\nnuget Newtonsoft.Json 13.0.3\nnuget Paket.Core prerelease\nnuget NUnit\nnuget Serilog copy_local: false\n";

    // Act and Assert
    expect(
        "paket.dependencies",
        text,
        &[
            ("Newtonsoft.Json", "13.0.3", "nuget"),
            ("Paket.Core", "", "nuget"),
            ("NUnit", "", "nuget"),
            ("Serilog", "", "nuget"),
        ],
    );
}

#[test]
fn reads_each_pom_dependency_sharing_a_property() {
    // Arrange
    let text = "<project>\n  <properties>\n    <spring.version>6.1.0</spring.version>\n  </properties>\n  <dependencies>\n    <dependency>\n      <groupId>org.springframework</groupId>\n      <artifactId>spring-core</artifactId>\n      <version>${spring.version}</version>\n    </dependency>\n    <dependency>\n      <groupId>org.springframework</groupId>\n      <artifactId>spring-web</artifactId>\n      <version>${spring.version}</version>\n    </dependency>\n  </dependencies>\n</project>\n";

    // Act and Assert
    expect(
        "pom.xml",
        text,
        &[
            ("org.springframework:spring-core", "6.1.0", "maven"),
            ("org.springframework:spring-web", "6.1.0", "maven"),
        ],
    );
}

#[test]
fn reads_flow_style_yaml_dependencies() {
    // Arrange: YAML 1.2 flow collections, which pub and conda read as their block forms.
    let pubspec = "name: app\ndependencies: {http: ^1.2.0, path: \"1.9.0\"}\n";
    let conda = "name: env\ndependencies: [numpy=1.26, {pip: [requests==2.31.0]}]\n";

    // Act and Assert
    expect(
        "pubspec.yaml",
        pubspec,
        &[("http", "^1.2.0", "pub"), ("path", "1.9.0", "pub")],
    );
    expect(
        "environment.yml",
        conda,
        &[("numpy", "1.26", "conda"), ("requests", "==2.31.0", "pypi")],
    );
}
