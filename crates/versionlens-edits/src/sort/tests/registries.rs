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
