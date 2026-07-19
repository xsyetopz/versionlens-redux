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
