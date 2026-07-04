use versionlens_parsers::{Dependency, Ecosystem};
use versionlens_vscode_model::{Position, Range};

use super::{can_sort_dependencies, sort_dependency_edits};

#[test]
fn sorts_requirements_dependency_lines_by_package_name() {
    let text = "zeta==1\n# keep\nalpha==1\n";
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
    let text = "zeta😀==1\nalpha==1\n";
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
    let text = "Alpha==1\nalpha==1\nBeta==1\nbeta==1\n";
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
    let text = "alpha==1\nä==1\nÁ==1\na==1\nÄ==1\ná==1\nA==1\n";
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
    let text = "\"zeta==1\"\n\"alpha==1\"\n\"z-optional==1\"\n\"a-optional==1\"\nzeta = \"1\"\nalpha = \"1\"\n";
    let dependencies = vec![
        dependency_with(
            Ecosystem::Python,
            "project.dependencies",
            "zeta",
            range(0, 1, 0, 5),
        ),
        dependency_with(
            Ecosystem::Python,
            "project.dependencies",
            "alpha",
            range(1, 1, 1, 6),
        ),
        dependency_with(
            Ecosystem::Python,
            "project.optional-dependencies.test",
            "z-optional",
            range(2, 1, 2, 11),
        ),
        dependency_with(
            Ecosystem::Python,
            "project.optional-dependencies.test",
            "a-optional",
            range(3, 1, 3, 11),
        ),
        dependency_with(
            Ecosystem::Python,
            "tool.poetry.dependencies",
            "zeta",
            range(4, 0, 4, 4),
        ),
        dependency_with(
            Ecosystem::Python,
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
    let text = "zeta==1\nalpha==1\nz-dev==1\na-dev==1\n";
    let dependencies = vec![
        dependency_with(Ecosystem::Pub, "dependencies", "zeta", range(0, 0, 0, 7)),
        dependency_with(Ecosystem::Pub, "dependencies", "alpha", range(1, 0, 1, 8)),
        dependency_with(
            Ecosystem::Pub,
            "dev_dependencies",
            "z-dev",
            range(2, 0, 2, 8),
        ),
        dependency_with(
            Ecosystem::Pub,
            "dev_dependencies",
            "a-dev",
            range(3, 0, 3, 8),
        ),
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
    let text = "  # zeta\n  zeta: 1\n  # alpha\n  alpha: 1\n";
    let dependencies = vec![
        dependency_with(Ecosystem::Pub, "dependencies", "zeta", range(1, 2, 1, 6)),
        dependency_with(Ecosystem::Pub, "dependencies", "alpha", range(3, 2, 3, 7)),
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
    let text = "  # zeta\r\n  zeta: 1\r\n  # alpha\r\n  alpha: 1\r\n";
    let dependencies = vec![
        dependency_with(Ecosystem::Pub, "dependencies", "zeta", range(1, 2, 1, 6)),
        dependency_with(Ecosystem::Pub, "dependencies", "alpha", range(3, 2, 3, 7)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(edits[0].new_text, "  # alpha\r\n  alpha: 1");
    assert_eq!(edits[1].new_text, "  # zeta\r\n  zeta: 1");
}

#[test]
fn sorts_pub_dependencies_with_inline_comments() {
    let text = "  zeta: 1 # zed\n  alpha: 1 # first\n";
    let dependencies = vec![
        dependency_with(Ecosystem::Pub, "dependencies", "zeta", range(0, 2, 0, 6)),
        dependency_with(Ecosystem::Pub, "dependencies", "alpha", range(1, 2, 1, 7)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(edits[0].new_text, "  alpha: 1 # first");
    assert_eq!(edits[1].new_text, "  zeta: 1 # zed");
}

#[test]
fn sorts_pub_dependencies_with_mixed_comments() {
    let text = "  http: # blank entry with comment\n  glob: # version child property\n    version: '1.2.*'\n";
    let dependencies = vec![
        dependency_with(Ecosystem::Pub, "dependencies", "http", range(0, 2, 0, 6)),
        dependency_with(Ecosystem::Pub, "dependencies", "glob", range(1, 2, 1, 6)),
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
    let text = r#"{"dependencies":{"zeta":"1","alpha":"1"}}"#;
    let dependencies = vec![
        dependency_with(Ecosystem::Npm, "dependencies", "zeta", range(0, 17, 0, 23)),
        dependency_with(Ecosystem::Npm, "dependencies", "alpha", range(0, 28, 0, 35)),
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
    let text = "{\n  \"dependencies\": {\n    \"zeta\": \"1\",\n    \"alpha\": \"1\"\n  }\n}";
    let dependencies = vec![
        dependency_with(Ecosystem::Npm, "dependencies", "zeta", range(2, 4, 2, 15)),
        dependency_with(Ecosystem::Npm, "dependencies", "alpha", range(3, 4, 3, 16)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(edits[0].new_text, "    \"alpha\": \"1\",");
    assert_eq!(edits[1].new_text, "    \"zeta\": \"1\"");
}

#[test]
fn sorts_package_json_dependencies_when_metadata_is_parsed() {
    let text = "{\n  \"version\": \"1.0.0\",\n  \"packageManager\": \"pnpm@9.0.0\",\n  \"dependencies\": {\n    \"zeta\": \"1\",\n    \"alpha\": \"1\"\n  }\n}";
    let dependencies = vec![
        dependency_with(Ecosystem::Npm, "version", "package", range(1, 2, 1, 21)),
        dependency_with(Ecosystem::Npm, "packageManager", "pnpm", range(2, 2, 2, 33)),
        dependency_with(Ecosystem::Npm, "dependencies", "zeta", range(4, 4, 4, 15)),
        dependency_with(Ecosystem::Npm, "dependencies", "alpha", range(5, 4, 5, 16)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(edits[0].new_text, "    \"alpha\": \"1\",");
    assert_eq!(edits[1].new_text, "    \"zeta\": \"1\"");
}

#[test]
fn sorts_smoke_jspm_dependencies() {
    let text = "{\n  \"jspm\": {\n    \"dependencies\": {\n      \"webpack\": \"npm:webpack@*\",\n      \"bluebird\": \"npm:bluebird@^3.7.2\"\n    }\n  }\n}";
    let dependencies = vec![
        dependency_with(
            Ecosystem::Npm,
            "jspm.dependencies",
            "webpack",
            range(3, 6, 3, 15),
        ),
        dependency_with(
            Ecosystem::Npm,
            "jspm.dependencies",
            "bluebird",
            range(4, 6, 4, 16),
        ),
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
    let text = "{\n  \"version\": \"1.0.0\",\n  \"require\": {\n    \"symfony/console\": \"8.1.*\",\n    \"allocine/twigcs\": \"^3.1.3\"\n  }\n}";
    let dependencies = vec![
        dependency_with(Ecosystem::Composer, "version", "1.0.0", range(1, 2, 1, 21)),
        dependency_with(
            Ecosystem::Composer,
            "require",
            "symfony/console",
            range(3, 4, 3, 21),
        ),
        dependency_with(
            Ecosystem::Composer,
            "require",
            "allocine/twigcs",
            range(4, 4, 4, 21),
        ),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(edits[0].new_text, "    \"allocine/twigcs\": \"^3.1.3\",");
    assert_eq!(edits[1].new_text, "    \"symfony/console\": \"8.1.*\"");
}

#[test]
fn sorts_deno_imports_with_npm_imports() {
    let text = "{\n  \"imports\": {\n    \"zeta\": \"jsr:@scope/zeta@1.0.0\",\n    \"chalk\": \"npm:chalk@5.3.0\"\n  }\n}";
    let dependencies = vec![
        dependency_with(Ecosystem::Deno, "imports", "zeta", range(2, 4, 2, 10)),
        dependency_with(Ecosystem::Npm, "imports", "chalk", range(3, 4, 3, 11)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(edits[0].new_text, "    \"chalk\": \"npm:chalk@5.3.0\",");
    assert_eq!(edits[1].new_text, "    \"zeta\": \"jsr:@scope/zeta@1.0.0\"");
}

#[test]
fn sorts_deno_scoped_imports_within_each_scope() {
    let text = "{\n  \"scopes\": {\n    \"https://deno.land/x/app/\": {\n      \"zeta\": \"npm:zeta@1.0.0\",\n      \"chalk\": \"npm:chalk@5.3.0\"\n    },\n    \"https://deno.land/x/other/\": {\n      \"bravo\": \"jsr:@scope/bravo@1.0.0\",\n      \"alpha\": \"jsr:@scope/alpha@1.0.0\"\n    }\n  }\n}";
    let dependencies = vec![
        dependency_with(
            Ecosystem::Npm,
            "scopes.https://deno.land/x/app/",
            "zeta",
            range(3, 6, 3, 12),
        ),
        dependency_with(
            Ecosystem::Npm,
            "scopes.https://deno.land/x/app/",
            "chalk",
            range(4, 6, 4, 13),
        ),
        dependency_with(
            Ecosystem::Deno,
            "scopes.https://deno.land/x/other/",
            "@scope/bravo",
            range(7, 6, 7, 13),
        ),
        dependency_with(
            Ecosystem::Deno,
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
    let text = "catalogs:\n  react18:\n    react-dom: ^19.2.7\n    react: ^18.3.1\n";
    let dependencies = vec![
        dependency_with(
            Ecosystem::Npm,
            "catalogs.react18",
            "react-dom",
            range(2, 4, 2, 13),
        ),
        dependency_with(
            Ecosystem::Npm,
            "catalogs.react18",
            "react",
            range(3, 4, 3, 9),
        ),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(edits[0].new_text, "    react: ^18.3.1");
    assert_eq!(edits[1].new_text, "    react-dom: ^19.2.7");
}

#[test]
fn sorts_package_json_named_workspace_catalogs() {
    let text = "{\n  \"workspaces\": {\n    \"catalogs\": {\n      \"react18\": {\n        \"react-dom\": \"^19.2.7\",\n        \"react\": \"^18.3.1\"\n      }\n    }\n  }\n}";
    let dependencies = vec![
        dependency_with(
            Ecosystem::Npm,
            "workspaces.catalogs.react18",
            "react-dom",
            range(4, 8, 4, 19),
        ),
        dependency_with(
            Ecosystem::Npm,
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
    let text = "[dev-dependencies]\nsyn = \"2\"\naxum-extra = \"0.12\"\n";
    let dependencies = vec![
        dependency_with(
            Ecosystem::Cargo,
            "dev-dependencies",
            "syn",
            range(1, 0, 1, 3),
        ),
        dependency_with(
            Ecosystem::Cargo,
            "dev-dependencies",
            "axum-extra",
            range(2, 0, 2, 10),
        ),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(edits[0].new_text, "axum-extra = \"0.12\"");
    assert_eq!(edits[1].new_text, "syn = \"2\"");
}

#[test]
fn sorts_maven_dependency_nodes() {
    let text = "<dependencies>\n  <dependency>\n    <groupId>org.zeta</groupId>\n    <artifactId>zeta</artifactId>\n    <version>1</version>\n  </dependency>\n  <dependency>\n    <groupId>org.alpha</groupId>\n    <artifactId>alpha</artifactId>\n    <version>1</version>\n  </dependency>\n</dependencies>";
    let dependencies = vec![
        dependency_with(
            Ecosystem::Maven,
            "project.dependencies.dependency",
            "org.zeta:zeta",
            range(1, 2, 5, 15),
        ),
        dependency_with(
            Ecosystem::Maven,
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
    let text = "<dependencyManagement>\n  <dependencies>\n    <dependency>\n      <groupId>org.zeta</groupId>\n      <artifactId>zeta</artifactId>\n      <version>1</version>\n    </dependency>\n    <dependency>\n      <groupId>org.alpha</groupId>\n      <artifactId>alpha</artifactId>\n      <version>1</version>\n    </dependency>\n  </dependencies>\n</dependencyManagement>";
    let dependencies = vec![
        dependency_with(
            Ecosystem::Maven,
            "project.dependencyManagement.dependencies.dependency",
            "org.zeta:zeta",
            range(2, 4, 6, 17),
        ),
        dependency_with(
            Ecosystem::Maven,
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
    let text = "require (\n\tzeta.example/pkg v1.0.0\n\talpha.example/pkg v1.0.0 // indirect\n)\n";
    let dependencies = vec![
        dependency_with(
            Ecosystem::Go,
            "require",
            "zeta.example/pkg",
            range(1, 1, 1, 17),
        ),
        dependency_with(
            Ecosystem::Go,
            "require",
            "alpha.example/pkg",
            range(2, 1, 2, 18),
        ),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(edits[0].new_text, "\talpha.example/pkg v1.0.0 // indirect");
    assert_eq!(edits[1].new_text, "\tzeta.example/pkg v1.0.0");
}

#[test]
fn does_not_sort_go_dependencies_across_require_blocks() {
    let text =
        "require (\n\tzeta.example/pkg v1.0.0\n)\n\nrequire (\n\talpha.example/pkg v1.0.0\n)\n";
    let dependencies = vec![
        dependency_with(
            Ecosystem::Go,
            "require",
            "zeta.example/pkg",
            range(1, 1, 1, 17),
        ),
        dependency_with(
            Ecosystem::Go,
            "require",
            "alpha.example/pkg",
            range(5, 1, 5, 18),
        ),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert!(edits.is_empty());
}

#[test]
fn sorts_ruby_gemfile_dependencies() {
    let text = "gem \"zeta\", \"1.0.0\"\ngem \"alpha\", \"1.0.0\"\n";
    let dependencies = vec![
        dependency_with(Ecosystem::Ruby, "dependencies", "zeta", range(0, 5, 0, 9)),
        dependency_with(Ecosystem::Ruby, "dependencies", "alpha", range(1, 5, 1, 10)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(edits[0].new_text, "gem \"alpha\", \"1.0.0\"");
    assert_eq!(edits[1].new_text, "gem \"zeta\", \"1.0.0\"");
}

#[test]
fn sorts_ruby_github_dependencies_by_gem_name() {
    let text = "gem \"zeta\", github: \"org/zeta\", tag: \"v1.0.0\"\ngem \"alpha\", github: \"org/alpha\", tag: \"v1.0.0\"\n";
    let dependencies = vec![
        Dependency {
            hosted_name: Some("zeta".to_owned()),
            ..dependency_with(
                Ecosystem::Ruby,
                "dependencies",
                "org/zeta",
                range(0, 5, 0, 9),
            )
        },
        Dependency {
            hosted_name: Some("alpha".to_owned()),
            ..dependency_with(
                Ecosystem::Ruby,
                "dependencies",
                "org/alpha",
                range(1, 5, 1, 10),
            )
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

#[test]
fn does_not_sort_ruby_dependencies_across_group_blocks() {
    let text = "group :production do\n  gem \"zeta\", \"1.0.0\"\nend\n\ngroup :production do\n  gem \"alpha\", \"1.0.0\"\nend\n";
    let dependencies = vec![
        dependency_with(
            Ecosystem::Ruby,
            "group :production",
            "zeta",
            range(1, 7, 1, 11),
        ),
        dependency_with(
            Ecosystem::Ruby,
            "group :production",
            "alpha",
            range(5, 7, 5, 12),
        ),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert!(edits.is_empty());
}

#[test]
fn sorts_dotnet_single_line_package_references() {
    let text = "<ItemGroup>\n  <PackageReference Include=\"Zeta.Package\" Version=\"1\" />\n  <PackageReference Include=\"Alpha.Package\" Version=\"1\" />\n</ItemGroup>";
    let dependencies = vec![
        dependency_with(
            Ecosystem::Dotnet,
            "PackageReference",
            "Zeta.Package",
            range(1, 2, 1, 61),
        ),
        dependency_with(
            Ecosystem::Dotnet,
            "PackageReference",
            "Alpha.Package",
            range(2, 2, 2, 62),
        ),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert_eq!(
        edits[0].new_text,
        "  <PackageReference Include=\"Alpha.Package\" Version=\"1\" />"
    );
    assert_eq!(
        edits[1].new_text,
        "  <PackageReference Include=\"Zeta.Package\" Version=\"1\" />"
    );
}

#[test]
fn skips_dotnet_version_insertion_ranges() {
    let text = "<ItemGroup>\n  <PackageReference Include=\"Zeta.Package\" />\n  <PackageReference Include=\"Alpha.Package\" />\n</ItemGroup>";
    let dependencies = vec![
        Dependency {
            requirement_prefix: " Version=\"".to_owned(),
            requirement_suffix: "\"".to_owned(),
            ..dependency_with(
                Ecosystem::Dotnet,
                "PackageReference",
                "Zeta.Package",
                range(1, 28, 1, 40),
            )
        },
        Dependency {
            requirement_prefix: " Version=\"".to_owned(),
            requirement_suffix: "\"".to_owned(),
            ..dependency_with(
                Ecosystem::Dotnet,
                "PackageReference",
                "Alpha.Package",
                range(2, 28, 2, 41),
            )
        },
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert!(edits.is_empty());
}

#[test]
fn skips_sort_when_each_sortable_group_has_one_dependency() {
    let text = "{\n  \"dependencies\": {\n    \"zeta\": \"1\"\n  },\n  \"devDependencies\": {\n    \"alpha\": \"1\"\n  }\n}";
    let dependencies = vec![
        dependency_with(Ecosystem::Npm, "dependencies", "zeta", range(2, 4, 2, 15)),
        dependency_with(
            Ecosystem::Npm,
            "devDependencies",
            "alpha",
            range(5, 4, 5, 16),
        ),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert!(edits.is_empty());
}

fn dependency(name: &str, range: Range) -> Dependency {
    dependency_with(Ecosystem::Python, "requirements", name, range)
}

fn dependency_with(ecosystem: Ecosystem, group: &str, name: &str, range: Range) -> Dependency {
    Dependency {
        name: name.to_owned(),
        requirement: "1".to_owned(),
        ecosystem,
        group: group.to_owned(),
        hosted_url: None,
        hosted_name: None,
        range,
        requirement_range: range,
        requirement_prefix: String::new(),
        requirement_suffix: String::new(),
    }
}

fn range(start_line: u32, start_character: u32, end_line: u32, end_character: u32) -> Range {
    Range {
        start: Position {
            line: start_line,
            character: start_character,
        },
        end: Position {
            line: end_line,
            character: end_character,
        },
    }
}
