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
fn same_line_sort_ignores_quoted_and_nested_delimiters() {
    let text = r#"{"dependencies":{"zeta":{"description":"contains , } ] and \"quote\"","path":"C:\\tmp\\[x],{y}","nested":[1,2]},"alpha":{"description":"a,b","nested":{"x":"[braces]"}}}}"#;
    let zeta = u32::try_from(text.find("\"zeta\"").unwrap()).unwrap();
    let alpha = u32::try_from(text.find("\"alpha\"").unwrap()).unwrap();
    let dependencies = vec![
        dependency_with(Npm, "dependencies", "zeta", range(0, zeta, 0, zeta + 6)),
        dependency_with(Npm, "dependencies", "alpha", range(0, alpha, 0, alpha + 7)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert_eq!(edits.len(), 2);
    assert!(edits[0].new_text.contains(r#"description":"a,b"#));
    assert!(edits[0].new_text.contains(r#"nested":{"x":"[braces]"}"#));
    assert!(
        edits[1]
            .new_text
            .contains(r#"contains , } ] and \"quote\""#)
    );
    assert!(edits[1].new_text.contains(r#"nested":[1,2]"#));

    let updated = apply_same_line_edits(text, &edits);
    serde_json::from_str::<serde_json::Value>(&updated).expect("sorted JSON remains parseable");
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
