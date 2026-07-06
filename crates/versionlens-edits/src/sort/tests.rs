use std::fs::read_to_string;
use std::path::PathBuf;
use versionlens_parsers::{Dependency, Ecosystem};
use versionlens_vscode_model::{Position, Range};

use super::{can_sort_dependencies, sort_dependency_edits};
use versionlens_parsers::Ecosystem::{Cargo, Composer, Deno, Go, Maven, Npm, Pub, Python, Ruby};

#[test]
fn sorts_requirements_dependency_lines_by_package_name() {
    let text = package_file_fixture("sorts-requirements-dependency-lines-by-package-name.txt");
    let dependencies = vec![
        dependency("zeta", range(0, 0, 0, 7)),
        dependency("alpha", range(2, 0, 2, 8)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert!(can_sort_dependencies(&dependencies));
    assert_eq!(edits.len(), 2);
    assert_eq!(edits[0].range, range(0, 0, 0, 7));
    assert_eq!(edits[0].new_text, "alpha==1");
    assert_eq!(edits[1].range, range(2, 0, 2, 8));
    assert_eq!(edits[1].new_text, "zeta==1");
}

#[test]
fn sort_edit_line_ranges_count_utf16_code_units() {
    let text = package_file_fixture("sort-edit-line-ranges-count-utf16-code-units.txt");
    let dependencies = vec![
        dependency("zeta😀", range(0, 0, 0, 6)),
        dependency("alpha", range(1, 0, 1, 8)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(edits[0].range, range(0, 0, 0, 9));
    assert_eq!(edits[0].new_text, "alpha==1");
    assert_eq!(edits[1].range, range(1, 0, 1, 8));
    assert_eq!(edits[1].new_text, "zeta😀==1");
}

#[test]
fn sorts_dependency_names_like_upstream_locale_compare() {
    let text = package_file_fixture("sorts-dependency-names-like-upstream-locale-compare.txt");
    let dependencies = vec![
        dependency("Alpha", range(0, 0, 0, 8)),
        dependency("alpha", range(1, 0, 1, 8)),
        dependency("Beta", range(2, 0, 2, 7)),
        dependency("beta", range(3, 0, 3, 7)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 4);
    assert_eq!(edits[0].new_text, "alpha==1");
    assert_eq!(edits[1].new_text, "Alpha==1");
    assert_eq!(edits[2].new_text, "beta==1");
    assert_eq!(edits[3].new_text, "Beta==1");
}

#[test]
fn sorts_accented_dependency_names_like_upstream_locale_compare() {
    let text =
        package_file_fixture("sorts-accented-dependency-names-like-upstream-locale-compare.txt");
    let dependencies = vec![
        dependency("alpha", range(0, 0, 0, 8)),
        dependency("ä", range(1, 0, 1, 5)),
        dependency("Á", range(2, 0, 2, 5)),
        dependency("a", range(3, 0, 3, 4)),
        dependency("Ä", range(4, 0, 4, 5)),
        dependency("á", range(5, 0, 5, 5)),
        dependency("A", range(6, 0, 6, 4)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 7);
    assert_eq!(edits[0].new_text, "a==1");
    assert_eq!(edits[1].new_text, "A==1");
    assert_eq!(edits[2].new_text, "á==1");
    assert_eq!(edits[3].new_text, "Á==1");
    assert_eq!(edits[4].new_text, "ä==1");
    assert_eq!(edits[5].new_text, "Ä==1");
    assert_eq!(edits[6].new_text, "alpha==1");
}

#[test]
fn sorts_pyproject_python_dependency_groups() {
    let text = package_file_fixture("sorts-pyproject-python-dependency-groups.txt");
    let dependencies = vec![
        dependency_with(Python, "project.dependencies", "zeta", range(0, 1, 0, 5)),
        dependency_with(Python, "project.dependencies", "alpha", range(1, 1, 1, 6)),
        dependency_with(
            Python,
            "project.optional-dependencies.test",
            "z-optional",
            range(2, 1, 2, 11),
        ),
        dependency_with(
            Python,
            "project.optional-dependencies.test",
            "a-optional",
            range(3, 1, 3, 11),
        ),
        dependency_with(
            Python,
            "tool.poetry.dependencies",
            "zeta",
            range(4, 0, 4, 4),
        ),
        dependency_with(
            Python,
            "tool.poetry.dependencies",
            "alpha",
            range(5, 0, 5, 5),
        ),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 6);
    assert_eq!(edits[0].new_text, "\"alpha==1\"");
    assert_eq!(edits[1].new_text, "\"zeta==1\"");
    assert_eq!(edits[2].new_text, "\"a-optional==1\"");
    assert_eq!(edits[3].new_text, "\"z-optional==1\"");
    assert_eq!(edits[4].new_text, "alpha = \"1\"");
    assert_eq!(edits[5].new_text, "zeta = \"1\"");
}

#[test]
fn sorts_each_dependency_group_independently() {
    let text = package_file_fixture("sorts-each-dependency-group-independently.txt");
    let dependencies = vec![
        dependency_with(Pub, "dependencies", "zeta", range(0, 0, 0, 7)),
        dependency_with(Pub, "dependencies", "alpha", range(1, 0, 1, 8)),
        dependency_with(Pub, "dev_dependencies", "z-dev", range(2, 0, 2, 8)),
        dependency_with(Pub, "dev_dependencies", "a-dev", range(3, 0, 3, 8)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 4);
    assert_eq!(edits[0].new_text, "alpha==1");
    assert_eq!(edits[1].new_text, "zeta==1");
    assert_eq!(edits[2].new_text, "a-dev==1");
    assert_eq!(edits[3].new_text, "z-dev==1");
}

#[test]
fn sorts_pub_dependencies_with_leading_comments() {
    let text = package_file_fixture("sorts-pub-dependencies-with-leading-comments.txt");
    let dependencies = vec![
        dependency_with(Pub, "dependencies", "zeta", range(1, 2, 1, 6)),
        dependency_with(Pub, "dependencies", "alpha", range(3, 2, 3, 7)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(edits[0].range, range(0, 0, 1, 9));
    assert_eq!(edits[0].new_text, "  # alpha\n  alpha: 1");
    assert_eq!(edits[1].range, range(2, 0, 3, 10));
    assert_eq!(edits[1].new_text, "  # zeta\n  zeta: 1");
}

#[test]
fn sorts_multiline_slots_preserving_original_line_endings() {
    let text = package_file_fixture("sorts-multiline-slots-preserving-original-line-endings.txt");
    let dependencies = vec![
        dependency_with(Pub, "dependencies", "zeta", range(1, 2, 1, 6)),
        dependency_with(Pub, "dependencies", "alpha", range(3, 2, 3, 7)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(edits[0].new_text, "  # alpha\r\n  alpha: 1");
    assert_eq!(edits[1].new_text, "  # zeta\r\n  zeta: 1");
}

#[test]
fn sorts_pub_dependencies_with_inline_comments() {
    let text = package_file_fixture("sorts-pub-dependencies-with-inline-comments.txt");
    let dependencies = vec![
        dependency_with(Pub, "dependencies", "zeta", range(0, 2, 0, 6)),
        dependency_with(Pub, "dependencies", "alpha", range(1, 2, 1, 7)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(edits[0].new_text, "  alpha: 1 # first");
    assert_eq!(edits[1].new_text, "  zeta: 1 # zed");
}

#[test]
fn sorts_pub_dependencies_with_mixed_comments() {
    let text = package_file_fixture("sorts-pub-dependencies-with-mixed-comments.txt");
    let dependencies = vec![
        dependency_with(Pub, "dependencies", "http", range(0, 2, 0, 6)),
        dependency_with(Pub, "dependencies", "glob", range(1, 2, 1, 6)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(
        edits[0].new_text,
        "  glob: # version child property\n    version: '1.2.*'"
    );
    assert_eq!(edits[1].new_text, "  http: # blank entry with comment");
}

#[test]
fn sorts_json_style_dependencies_that_share_a_line() {
    let text = package_file_fixture("sorts-json-style-dependencies-that-share-a-line.txt");
    let dependencies = vec![
        dependency_with(Npm, "dependencies", "zeta", range(0, 17, 0, 23)),
        dependency_with(Npm, "dependencies", "alpha", range(0, 28, 0, 35)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(edits[0].range, range(0, 17, 0, 27));
    assert_eq!(edits[0].new_text, r#""alpha":"1""#);
    assert_eq!(edits[1].range, range(0, 28, 0, 39));
    assert_eq!(edits[1].new_text, r#""zeta":"1""#);
}

#[test]
fn preserves_json_trailing_commas_for_target_slots() {
    let text = package_file_fixture("preserves-json-trailing-commas-for-target-slots.txt");
    let dependencies = vec![
        dependency_with(Npm, "dependencies", "zeta", range(2, 4, 2, 15)),
        dependency_with(Npm, "dependencies", "alpha", range(3, 4, 3, 16)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(edits[0].new_text, "    \"alpha\": \"1\",");
    assert_eq!(edits[1].new_text, "    \"zeta\": \"1\"");
}

#[test]
fn sorts_package_json_dependencies_when_metadata_is_parsed() {
    let text = package_file_fixture("sorts-package-json-dependencies-when-metadata-is-parsed.txt");
    let dependencies = vec![
        dependency_with(Npm, "version", "package", range(1, 2, 1, 21)),
        dependency_with(Npm, "packageManager", "pnpm", range(2, 2, 2, 33)),
        dependency_with(Npm, "dependencies", "zeta", range(4, 4, 4, 15)),
        dependency_with(Npm, "dependencies", "alpha", range(5, 4, 5, 16)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(edits[0].new_text, "    \"alpha\": \"1\",");
    assert_eq!(edits[1].new_text, "    \"zeta\": \"1\"");
}

#[test]
fn sorts_smoke_jspm_dependencies() {
    let text = package_file_fixture("sorts-smoke-jspm-dependencies.txt");
    let dependencies = vec![
        dependency_with(Npm, "jspm.dependencies", "webpack", range(3, 6, 3, 15)),
        dependency_with(Npm, "jspm.dependencies", "bluebird", range(4, 6, 4, 16)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(
        edits[0].new_text,
        "      \"bluebird\": \"npm:bluebird@^3.7.2\","
    );
    assert_eq!(edits[1].new_text, "      \"webpack\": \"npm:webpack@*\"");
}

#[test]
fn sorts_composer_dependencies_without_moving_manifest_version() {
    let text =
        package_file_fixture("sorts-composer-dependencies-without-moving-manifest-version.txt");
    let dependencies = vec![
        dependency_with(Composer, "version", "1.0.0", range(1, 2, 1, 21)),
        dependency_with(Composer, "require", "symfony/console", range(3, 4, 3, 21)),
        dependency_with(Composer, "require", "allocine/twigcs", range(4, 4, 4, 21)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(edits[0].new_text, "    \"allocine/twigcs\": \"^3.1.3\",");
    assert_eq!(edits[1].new_text, "    \"symfony/console\": \"8.1.*\"");
}

#[test]
fn sorts_deno_imports_with_npm_imports() {
    let text = package_file_fixture("sorts-deno-imports-with-npm-imports.txt");
    let dependencies = vec![
        dependency_with(Deno, "imports", "zeta", range(2, 4, 2, 10)),
        dependency_with(Npm, "imports", "chalk", range(3, 4, 3, 11)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(edits[0].new_text, "    \"chalk\": \"npm:chalk@5.3.0\",");
    assert_eq!(edits[1].new_text, "    \"zeta\": \"jsr:@scope/zeta@1.0.0\"");
}

#[test]
fn sorts_deno_scoped_imports_within_each_scope() {
    let text = package_file_fixture("sorts-deno-scoped-imports-within-each-scope.txt");
    let dependencies = vec![
        dependency_with(
            Npm,
            "scopes.https://deno.land/x/app/",
            "zeta",
            range(3, 6, 3, 12),
        ),
        dependency_with(
            Npm,
            "scopes.https://deno.land/x/app/",
            "chalk",
            range(4, 6, 4, 13),
        ),
        dependency_with(
            Deno,
            "scopes.https://deno.land/x/other/",
            "@scope/bravo",
            range(7, 6, 7, 13),
        ),
        dependency_with(
            Deno,
            "scopes.https://deno.land/x/other/",
            "@scope/alpha",
            range(8, 6, 8, 13),
        ),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 4);
    assert_eq!(edits[0].new_text, "      \"chalk\": \"npm:chalk@5.3.0\",");
    assert_eq!(edits[1].new_text, "      \"zeta\": \"npm:zeta@1.0.0\"");
    assert_eq!(
        edits[2].new_text,
        "      \"alpha\": \"jsr:@scope/alpha@1.0.0\","
    );
    assert_eq!(
        edits[3].new_text,
        "      \"bravo\": \"jsr:@scope/bravo@1.0.0\""
    );
}

#[test]
fn sorts_pnpm_workspace_catalogs() {
    let text = package_file_fixture("sorts-pnpm-workspace-catalogs.txt");
    let dependencies = vec![
        dependency_with(Npm, "catalogs.react18", "react-dom", range(2, 4, 2, 13)),
        dependency_with(Npm, "catalogs.react18", "react", range(3, 4, 3, 9)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(edits[0].new_text, "    react: ^18.3.1");
    assert_eq!(edits[1].new_text, "    react-dom: ^19.2.7");
}

#[test]
fn sorts_package_json_named_workspace_catalogs() {
    let text = package_file_fixture("sorts-package-json-named-workspace-catalogs.txt");
    let dependencies = vec![
        dependency_with(
            Npm,
            "workspaces.catalogs.react18",
            "react-dom",
            range(4, 8, 4, 19),
        ),
        dependency_with(
            Npm,
            "workspaces.catalogs.react18",
            "react",
            range(5, 8, 5, 15),
        ),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(edits[0].new_text, "        \"react\": \"^18.3.1\",");
    assert_eq!(edits[1].new_text, "        \"react-dom\": \"^19.2.7\"");
}

#[test]
fn sorts_cargo_dependency_tables() {
    let text = package_file_fixture("sorts-cargo-dependency-tables.txt");
    let dependencies = vec![
        dependency_with(Cargo, "dev-dependencies", "syn", range(1, 0, 1, 3)),
        dependency_with(Cargo, "dev-dependencies", "axum-extra", range(2, 0, 2, 10)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(edits[0].new_text, "axum-extra = \"0.12\"");
    assert_eq!(edits[1].new_text, "syn = \"2\"");
}

#[test]
fn sorts_cargo_renamed_dependencies_by_local_alias() {
    let text = package_file_fixture("sorts-cargo-renamed-dependencies-by-local-alias.txt");
    let dependencies = vec![
        Dependency {
            hosted_name: Some("alpha".to_owned()),
            ..dependency_with(Cargo, "dependencies", "z_alias", range(1, 0, 1, 7))
        },
        Dependency {
            hosted_name: Some("zeta".to_owned()),
            ..dependency_with(Cargo, "dependencies", "a_alias", range(2, 0, 2, 7))
        },
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(
        edits[0].new_text,
        "a_alias = { package = \"zeta\", version = \"1\" }"
    );
    assert_eq!(
        edits[1].new_text,
        "z_alias = { package = \"alpha\", version = \"1\" }"
    );
}

#[test]
fn sorts_composer_package_link_groups() {
    let text = package_file_fixture("sorts-composer-package-link-groups.txt");
    let dependencies = vec![
        dependency_with(Composer, "conflict", "zeta/package", range(0, 0, 0, 14)),
        dependency_with(Composer, "conflict", "alpha/package", range(1, 0, 1, 15)),
        dependency_with(
            Composer,
            "replace",
            "vendor/replaced-zeta",
            range(2, 0, 2, 22),
        ),
        dependency_with(
            Composer,
            "replace",
            "vendor/replaced-alpha",
            range(3, 0, 3, 23),
        ),
        dependency_with(
            Composer,
            "provide",
            "psr/zeta-implementation",
            range(4, 0, 4, 25),
        ),
        dependency_with(
            Composer,
            "provide",
            "psr/alpha-implementation",
            range(5, 0, 5, 26),
        ),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 6);
    assert_eq!(edits[0].new_text, r#""alpha/package": "<2","#);
    assert_eq!(edits[1].new_text, r#""zeta/package": "<2","#);
    assert_eq!(
        edits[2].new_text,
        r#""vendor/replaced-alpha": "self.version","#
    );
    assert_eq!(
        edits[3].new_text,
        r#""vendor/replaced-zeta": "self.version","#
    );
    assert_eq!(edits[4].new_text, r#""psr/alpha-implementation": "1.0","#);
    assert_eq!(edits[5].new_text, r#""psr/zeta-implementation": "1.0""#);
}

#[test]
fn sorts_maven_dependency_nodes() {
    let text = package_file_fixture("sorts-maven-dependency-nodes.txt");
    let dependencies = vec![
        dependency_with(
            Maven,
            "project.dependencies.dependency",
            "org.zeta:zeta",
            range(1, 2, 5, 15),
        ),
        dependency_with(
            Maven,
            "project.dependencies.dependency",
            "org.alpha:alpha",
            range(6, 2, 10, 15),
        ),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(
        edits[0].new_text,
        "  <dependency>\n    <groupId>org.alpha</groupId>\n    <artifactId>alpha</artifactId>\n    <version>1</version>\n  </dependency>"
    );
    assert_eq!(
        edits[1].new_text,
        "  <dependency>\n    <groupId>org.zeta</groupId>\n    <artifactId>zeta</artifactId>\n    <version>1</version>\n  </dependency>"
    );
}

#[test]
fn sorts_maven_dependency_management_nodes() {
    let text = package_file_fixture("sorts-maven-dependency-management-nodes.txt");
    let dependencies = vec![
        dependency_with(
            Maven,
            "project.dependencyManagement.dependencies.dependency",
            "org.zeta:zeta",
            range(2, 4, 6, 17),
        ),
        dependency_with(
            Maven,
            "project.dependencyManagement.dependencies.dependency",
            "org.alpha:alpha",
            range(7, 4, 11, 17),
        ),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(
        edits[0].new_text,
        "    <dependency>\n      <groupId>org.alpha</groupId>\n      <artifactId>alpha</artifactId>\n      <version>1</version>\n    </dependency>"
    );
    assert_eq!(
        edits[1].new_text,
        "    <dependency>\n      <groupId>org.zeta</groupId>\n      <artifactId>zeta</artifactId>\n      <version>1</version>\n    </dependency>"
    );
}

#[test]
fn sorts_go_require_block_dependencies() {
    let text = package_file_fixture("sorts-go-require-block-dependencies.txt");
    let dependencies = vec![
        dependency_with(Go, "require", "zeta.example/pkg", range(1, 1, 1, 17)),
        dependency_with(Go, "require", "alpha.example/pkg", range(2, 1, 2, 18)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(edits[0].new_text, "\talpha.example/pkg v1.0.0 // indirect");
    assert_eq!(edits[1].new_text, "\tzeta.example/pkg v1.0.0");
}

#[test]
fn does_not_sort_go_dependencies_across_require_blocks() {
    let text = package_file_fixture("does-not-sort-go-dependencies-across-require-blocks.txt");
    let dependencies = vec![
        dependency_with(Go, "require", "zeta.example/pkg", range(1, 1, 1, 17)),
        dependency_with(Go, "require", "alpha.example/pkg", range(5, 1, 5, 18)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert!(edits.is_empty());
}

#[test]
fn sorts_ruby_gemfile_dependencies() {
    let text = package_file_fixture("sorts-ruby-gemfile-dependencies.txt");
    let dependencies = vec![
        dependency_with(Ruby, "dependencies", "zeta", range(0, 5, 0, 9)),
        dependency_with(Ruby, "dependencies", "alpha", range(1, 5, 1, 10)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(edits[0].new_text, "gem \"alpha\", \"1.0.0\"");
    assert_eq!(edits[1].new_text, "gem \"zeta\", \"1.0.0\"");
}

#[test]
fn sorts_ruby_github_dependencies_by_gem_name() {
    let text = package_file_fixture("sorts-ruby-github-dependencies-by-gem-name.txt");
    let dependencies = vec![
        Dependency {
            hosted_name: Some("zeta".to_owned()),
            ..dependency_with(Ruby, "dependencies", "org/zeta", range(0, 5, 0, 9))
        },
        Dependency {
            hosted_name: Some("alpha".to_owned()),
            ..dependency_with(Ruby, "dependencies", "org/alpha", range(1, 5, 1, 10))
        },
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(
        edits[0].new_text,
        "gem \"alpha\", github: \"org/alpha\", tag: \"v1.0.0\""
    );
    assert_eq!(
        edits[1].new_text,
        "gem \"zeta\", github: \"org/zeta\", tag: \"v1.0.0\""
    );
}

include!("tests/more.rs");

fn package_file_fixture(name: &str) -> &'static str {
    let path = repo_root()
        .join("tests/fixtures/versionlens-edits/src/sort/tests")
        .join(name);
    let contents = read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read package-file fixture {}: {error}",
            path.display()
        )
    });
    crate::leaked_string(contents)
}

fn repo_root() -> PathBuf {
    let manifest_dir: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("crate should be under crates/")
        .to_path_buf()
}
