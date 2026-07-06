use versionlens_parsers::Ecosystem::{Conan, Cpan, Cran, Dotnet, Haxelib, Helm, Julia, LuaRocks, Nim, Swift, Terraform, Vcpkg, Zig};
#[test]
fn does_not_sort_ruby_dependencies_across_group_blocks() {
    let text = package_file_fixture("does-not-sort-ruby-dependencies-across-group-blocks.txt");
    let dependencies = vec![
        dependency_with(
            Ruby,
            "group :production",
            "zeta",
            range(1, 7, 1, 11),
        ),
        dependency_with(
            Ruby,
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
    let text = package_file_fixture("sorts-dotnet-single-line-package-references.txt");
    let dependencies = vec![
        dependency_with(
            Dotnet,
            "PackageReference",
            "Zeta.Package",
            range(1, 2, 1, 61),
        ),
        dependency_with(
            Dotnet,
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
    let text = package_file_fixture("skips-dotnet-version-insertion-ranges.txt");
    let dependencies = vec![
        Dependency {
            requirement_prefix: " Version=\"".to_owned(),
            requirement_suffix: "\"".to_owned(),
            ..dependency_with(
                Dotnet,
                "PackageReference",
                "Zeta.Package",
                range(1, 28, 1, 40),
            )
        },
        Dependency {
            requirement_prefix: " Version=\"".to_owned(),
            requirement_suffix: "\"".to_owned(),
            ..dependency_with(
                Dotnet,
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
fn skips_julia_dependencies() {
    let text = package_file_fixture("skips-julia-dependencies.txt");
    let dependencies = vec![
        dependency_with(Julia, "compat", "Zeta", range(1, 0, 1, 10)),
        dependency_with(Julia, "compat", "Alpha", range(2, 0, 2, 11)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert!(!can_sort_dependencies(&dependencies));
    assert!(edits.is_empty());
}

#[test]
fn skips_cran_dependencies() {
    let text = package_file_fixture("skips-cran-dependencies.txt");
    let dependencies = vec![
        dependency_with(Cran, "Imports", "zeta", range(0, 9, 0, 13)),
        dependency_with(Cran, "Imports", "alpha", range(0, 24, 0, 29)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert!(!can_sort_dependencies(&dependencies));
    assert!(edits.is_empty());
}

#[test]
fn skips_conan_dependencies() {
    let text = package_file_fixture("skips-conan-dependencies.txt");
    let dependencies = vec![
        dependency_with(Conan, "requires", "zlib", range(1, 0, 1, 4)),
        dependency_with(Conan, "requires", "poco", range(2, 0, 2, 4)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert!(!can_sort_dependencies(&dependencies));
    assert!(edits.is_empty());
}

#[test]
fn skips_vcpkg_dependencies() {
    let text = package_file_fixture("skips-vcpkg-dependencies.txt");
    let dependencies = vec![
        dependency_with(
            Vcpkg,
            "dependencies",
            "zlib",
            range(0, 18, 0, 22),
        ),
        dependency_with(Vcpkg, "dependencies", "fmt", range(0, 32, 0, 35)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert!(!can_sort_dependencies(&dependencies));
    assert!(edits.is_empty());
}

#[test]
fn skips_swift_package_dependencies() {
    let text = package_file_fixture("skips-swift-package-dependencies.txt");
    let dependencies = vec![
        dependency_with(
            Swift,
            "dependencies",
            "zlib",
            range(1, 60, 1, 65),
        ),
        dependency_with(Swift, "dependencies", "fmt", range(2, 59, 2, 65)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert!(!can_sort_dependencies(&dependencies));
    assert!(edits.is_empty());
}

#[test]
fn skips_zig_zon_dependencies() {
    let text = package_file_fixture("skips-zig-zon-dependencies.txt");
    let dependencies = vec![
        dependency_with(Zig, "dependencies", "zlib", range(0, 44, 0, 72)),
        dependency_with(Zig, "dependencies", "fmt", range(0, 95, 0, 122)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert!(!can_sort_dependencies(&dependencies));
    assert!(edits.is_empty());
}

#[test]
fn skips_nimble_dependencies() {
    let text = package_file_fixture("skips-nimble-dependencies.txt");
    let dependencies = vec![
        dependency_with(Nim, "requires", "zlib", range(0, 10, 0, 14)),
        dependency_with(Nim, "requires", "fmt", range(1, 10, 1, 13)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert!(!can_sort_dependencies(&dependencies));
    assert!(edits.is_empty());
}

#[test]
fn skips_luarocks_rockspec_dependencies() {
    let text = package_file_fixture("skips-luarocks-rockspec-dependencies.txt");
    let dependencies = vec![
        dependency_with(
            LuaRocks,
            "dependencies",
            "zlib",
            range(1, 4, 1, 8),
        ),
        dependency_with(
            LuaRocks,
            "dependencies",
            "busted",
            range(2, 4, 2, 10),
        ),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert!(!can_sort_dependencies(&dependencies));
    assert!(edits.is_empty());
}

#[test]
fn skips_cpanfile_dependencies() {
    let text = package_file_fixture("skips-cpanfile-dependencies.txt");
    let dependencies = vec![
        dependency_with(Cpan, "requires", "Zeta", range(0, 10, 0, 14)),
        dependency_with(Cpan, "requires", "Alpha", range(1, 10, 1, 15)),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert!(!can_sort_dependencies(&dependencies));
    assert!(edits.is_empty());
}

#[test]
fn skips_haxelib_json_dependencies() {
    let text = package_file_fixture("skips-haxelib-json-dependencies.txt");
    let dependencies = vec![
        dependency_with(
            Haxelib,
            "dependencies",
            "zeta",
            range(0, 18, 0, 22),
        ),
        dependency_with(
            Haxelib,
            "dependencies",
            "alpha",
            range(0, 35, 0, 40),
        ),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert!(!can_sort_dependencies(&dependencies));
    assert!(edits.is_empty());
}

#[test]
fn skips_terraform_required_providers() {
    let text = package_file_fixture("skips-terraform-required-providers.txt");
    let dependencies = vec![
        dependency_with(
            Terraform,
            "required_providers",
            "hashicorp/zeta",
            range(0, 33, 0, 37),
        ),
        dependency_with(
            Terraform,
            "required_providers",
            "hashicorp/alpha",
            range(0, 47, 0, 52),
        ),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert!(!can_sort_dependencies(&dependencies));
    assert!(edits.is_empty());
}

#[test]
fn skips_helm_chart_dependencies() {
    let text = package_file_fixture("skips-helm-chart-dependencies.txt");
    let dependencies = vec![
        dependency_with(Helm, "dependencies", "zeta", range(1, 10, 1, 14)),
        dependency_with(
            Helm,
            "dependencies",
            "alpha",
            range(3, 10, 3, 15),
        ),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert!(!can_sort_dependencies(&dependencies));
    assert!(edits.is_empty());
}

#[test]
fn skips_sort_when_each_sortable_group_has_one_dependency() {
    let text = package_file_fixture("skips-sort-when-each-sortable-group-has-one-dependency.txt");
    let dependencies = vec![
        dependency_with(Npm, "dependencies", "zeta", range(2, 4, 2, 15)),
        dependency_with(
            Npm,
            "devDependencies",
            "alpha",
            range(5, 4, 5, 16),
        ),
    ];

    let edits = sort_dependency_edits(text, &dependencies);

    assert!(edits.is_empty());
}

fn dependency(name: &str, range: Range) -> Dependency {
    dependency_with(Python, "requirements", name, range)
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
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
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
