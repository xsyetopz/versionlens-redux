use versionlens_model::Ecosystem::{AnsibleGalaxy, Bazel, Cpan, Cran, Haxelib, Helm, Julia, Nix, Terraform};
#[test]
fn parses_cpanfile_dependencies() {
    let text = package_file_fixture("parses-cpanfile-dependencies.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/cpanfile".to_owned(),
        language_id: "perl".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 7);
    assert_eq!(dependencies[0].ecosystem, Cpan);
    assert_eq!(dependencies[0].group, "requires");
    assert_eq!(dependencies[0].name, "Plack");
    assert_eq!(dependencies[0].requirement, "1.0");
    assert_eq!(dependencies[1].name, "JSON");
    assert_eq!(dependencies[1].requirement, ">= 2.00, < 2.80");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "2.00, < 2.80"
    );
    assert_eq!(dependencies[2].group, "recommends");
    assert_eq!(dependencies[3].group, "conflicts");
    assert_eq!(dependencies[4].group, "test.requires");
    assert_eq!(dependencies[4].name, "Test::More");
    assert_eq!(dependencies[5].group, "feature.sqlite.recommends");
    assert_eq!(dependencies[5].name, "DBD::SQLite");
    assert_eq!(dependencies[5].requirement, "0");
    assert_eq!(dependencies[6].group, "build.requires");
    assert_eq!(dependencies[6].name, "Module::Build");
}

#[test]
fn parses_r_description_dependencies() {
    let text = package_file_fixture("parses-r-description-dependencies.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/DESCRIPTION".to_owned(),
        language_id: "plaintext".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 7);
    assert_eq!(dependencies[0].ecosystem, Cran);
    assert_eq!(dependencies[0].group, "Version");
    assert_eq!(dependencies[0].name, "demo");
    assert_eq!(dependencies[0].requirement, "0.1.0");
    assert_eq!(dependencies[1].name, "R");
    assert_eq!(dependencies[1].requirement, ">= 4.3");
    assert_eq!(dependencies[2].name, "dplyr");
    assert_eq!(dependencies[2].requirement, ">= 1.1.0");
    assert_eq!(dependencies[3].name, "tidyr");
    assert_eq!(dependencies[3].requirement, "latest");
    assert_eq!(dependencies[4].group, "Imports");
    assert_eq!(dependencies[5].group, "Suggests");
    assert_eq!(dependencies[6].group, "LinkingTo");
}

#[test]
fn parses_renv_lock_repository_packages_and_sources() {
    let text = package_file_fixture("parses-renv-lock-repository-packages-and-sources.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/renv.lock".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].ecosystem, Cran);
    assert_eq!(dependencies[0].group, "Packages");
    assert_eq!(dependencies[0].name, "dplyr");
    assert_eq!(dependencies[0].requirement, "1.1.4");
    assert_eq!(dependencies[0].hosted_url, None);
    assert_eq!(dependencies[1].name, "localpkg");
    assert_eq!(dependencies[1].hosted_url, Some("local".to_owned()));
}

#[test]
fn parses_julia_project_dependencies_and_sources() {
    let text = package_file_fixture("parses-julia-project-dependencies-and-sources.lock");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Project.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 5);
    assert_eq!(dependencies[0].ecosystem, Julia);
    assert_eq!(dependencies[0].group, "version");
    assert_eq!(dependencies[0].name, "Demo");
    assert_eq!(dependencies[0].requirement, "0.1.0");
    assert_eq!(dependencies[1].group, "compat");
    assert_eq!(dependencies[1].name, "Example");
    assert_eq!(dependencies[1].requirement, "1.2");
    assert_eq!(dependencies[2].name, "LocalPkg");
    assert_eq!(dependencies[2].requirement, "deps/LocalPkg");
    assert_eq!(dependencies[2].hosted_url, Some("path".to_owned()));
    assert_eq!(dependencies[3].name, "GitPkg");
    assert_eq!(
        dependencies[3].requirement,
        "https://github.com/example/GitPkg.jl"
    );
    assert_eq!(dependencies[3].hosted_url, Some("git".to_owned()));
    assert_eq!(dependencies[4].name, "julia");
    assert_eq!(dependencies[4].requirement, "1.10");
}

#[test]
fn parses_julia_manifest_entries() {
    let text = package_file_fixture("parses-julia-manifest-entries.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Manifest.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].ecosystem, Julia);
    assert_eq!(dependencies[0].group, "deps");
    assert_eq!(dependencies[0].name, "Example");
    assert_eq!(dependencies[0].requirement, "1.2.3");
    assert_eq!(dependencies[1].name, "LocalPkg");
    assert_eq!(
        dependencies[1].requirement,
        "/home/user/.julia/dev/LocalPkg"
    );
    assert_eq!(dependencies[1].hosted_url, Some("path".to_owned()));
    assert_eq!(dependencies[2].name, "GitPkg");
    assert_eq!(
        dependencies[2].requirement,
        "https://github.com/example/GitPkg.jl.git"
    );
    assert_eq!(dependencies[2].hosted_url, Some("git".to_owned()));
}

#[test]
fn parses_mix_exs_dependencies() {
    let text = package_file_fixture("parses-mix-exs-dependencies.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/mix.exs".to_owned(),
        language_id: "elixir".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 4);
    assert_eq!(dependencies[0].ecosystem, Hex);
    assert_eq!(dependencies[0].group, "deps");
    assert_eq!(dependencies[0].name, "plug");
    assert_eq!(dependencies[0].requirement, ">= 1.15.0");
    assert_eq!(dependencies[1].name, "phoenix");
    assert_eq!(dependencies[1].requirement, "~> 1.7");
    assert_eq!(dependencies[1].group, "deps.dev,test");
    assert_eq!(dependencies[2].name, "gettext");
    assert_eq!(
        dependencies[2].requirement,
        "https://github.com/elixir-lang/gettext.git"
    );
    assert_eq!(dependencies[2].hosted_url, Some("git".to_owned()));
    assert_eq!(dependencies[3].name, "local_dependency");
    assert_eq!(dependencies[3].requirement, "../local_dependency");
    assert_eq!(dependencies[3].hosted_url, Some("path".to_owned()));
}

#[test]
fn parses_haxelib_json_dependencies() {
    let text = package_file_fixture("parses-haxelib-json-dependencies.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/haxelib.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].ecosystem, Haxelib);
    assert_eq!(dependencies[0].group, "dependencies");
    assert_eq!(dependencies[0].name, "tink_core");
    assert_eq!(dependencies[0].requirement, "2.0.0");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "2.0.0"
    );
    assert_eq!(dependencies[1].group, "dependencies");
    assert_eq!(dependencies[1].name, "tink_macro");
    assert_eq!(dependencies[1].requirement, "");
    assert_eq!(dependencies[1].hosted_url, Some("latest".to_owned()));
}

#[test]
fn parses_terraform_required_providers() {
    let text = package_file_fixture("parses-terraform-required-providers.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/main.tf".to_owned(),
        language_id: "terraform".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].ecosystem, Terraform);
    assert_eq!(dependencies[0].group, "required_providers");
    assert_eq!(dependencies[0].name, "hashicorp/aws");
    assert_eq!(dependencies[0].hosted_name, Some("aws".to_owned()));
    assert_eq!(dependencies[0].requirement, "~> 5.0");
    assert_eq!(dependencies[0].requirement_prefix, "");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "5.0"
    );
    assert_eq!(dependencies[1].name, "hashicorp/random");
    assert_eq!(dependencies[1].hosted_name, Some("random".to_owned()));
    assert_eq!(dependencies[1].requirement, "~> 3.6");
    assert_eq!(dependencies[2].name, "terraform.io/builtin/terraform");
    assert_eq!(dependencies[2].requirement, "latest");
    assert_eq!(dependencies[2].hosted_url, Some("builtin".to_owned()));
}

#[test]
fn parses_helm_chart_dependencies() {
    let text = package_file_fixture("parses-helm-chart-dependencies.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Chart.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 4);
    assert_eq!(dependencies[0].ecosystem, Helm);
    assert_eq!(dependencies[0].group, "dependencies");
    assert_eq!(dependencies[0].name, "apache");
    assert_eq!(
        dependencies[0].hosted_url,
        Some("https://example.com/charts".to_owned())
    );
    assert_eq!(dependencies[0].requirement, "1.2.3");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "1.2.3"
    );
    assert_eq!(dependencies[1].name, "database");
    assert_eq!(dependencies[1].hosted_name, Some("mysql".to_owned()));
    assert_eq!(
        dependencies[1].hosted_url,
        Some("oci://registry.example.com/charts".to_owned())
    );
    assert_eq!(dependencies[1].requirement, "~3.2.1");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "3.2.1"
    );
    assert_eq!(dependencies[2].name, "local");
    assert_eq!(dependencies[2].hosted_url, Some("file".to_owned()));
    assert_eq!(dependencies[3].name, "repo-alias");
    assert_eq!(dependencies[3].hosted_url, Some("repo-alias".to_owned()));
}

#[test]
fn parses_ansible_galaxy_requirements() {
    let text = package_file_fixture("parses-ansible-galaxy-requirements.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/requirements.yml".to_owned(),
        language_id: "yaml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 4);
    assert_eq!(dependencies[0].ecosystem, AnsibleGalaxy);
    assert_eq!(dependencies[0].group, "roles");
    assert_eq!(dependencies[0].name, "geerlingguy.java");
    assert_eq!(dependencies[0].requirement, "1.9.6");
    assert_eq!(dependencies[1].name, "nginx");
    assert_eq!(dependencies[1].hosted_url, Some("git".to_owned()));
    assert_eq!(dependencies[1].requirement, "main");
    assert_eq!(dependencies[2].group, "collections");
    assert_eq!(dependencies[2].name, "community.general");
    assert_eq!(dependencies[2].requirement, ">=7.0.0");
    assert_eq!(
        dependencies[2].hosted_url,
        Some("https://galaxy.ansible.com".to_owned())
    );
    assert_eq!(dependencies[2].requirement_prefix, ">=");
    assert_eq!(
        extract_range(text, dependencies[2].requirement_range),
        "7.0.0"
    );
    assert_eq!(dependencies[3].name, "acme.private");
    assert_eq!(
        dependencies[3].hosted_url,
        Some("https://galaxy.example.test".to_owned())
    );
}

#[test]
fn parses_bazel_module_dependencies_and_overrides() {
    let text = package_file_fixture("parses-bazel-module-dependencies-and-overrides.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/MODULE.bazel".to_owned(),
        language_id: "starlark".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 5);
    assert_eq!(dependencies[0].ecosystem, Bazel);
    assert_eq!(dependencies[0].group, "bazel_dep");
    assert_eq!(dependencies[0].name, "rules_cc");
    assert_eq!(dependencies[0].requirement, "0.0.9");
    assert_eq!(dependencies[1].name, "bazel_skylib");
    assert_eq!(dependencies[1].hosted_name, Some("skylib".to_owned()));
    assert_eq!(dependencies[1].requirement, "1.3.0");
    assert_eq!(dependencies[2].group, "single_version_override");
    assert_eq!(dependencies[2].name, "protobuf");
    assert_eq!(dependencies[2].requirement, "29.0");
    assert_eq!(
        dependencies[2].hosted_url,
        Some("https://bcr.example.test".to_owned())
    );
    assert_eq!(dependencies[3].group, "git_override");
    assert_eq!(dependencies[3].name, "rules_local");
    assert_eq!(dependencies[3].hosted_url, Some("git".to_owned()));
    assert_eq!(dependencies[3].requirement, "abc123");
    assert_eq!(dependencies[4].group, "local_path_override");
    assert_eq!(dependencies[4].name, "local_rules");
    assert_eq!(dependencies[4].hosted_url, Some("path".to_owned()));
    assert_eq!(dependencies[4].requirement, "../local_rules");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "0.0.9"
    );
}

#[test]
fn parses_nix_flake_inputs() {
    let text = package_file_fixture("parses-nix-flake-inputs.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/flake.nix".to_owned(),
        language_id: "nix".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 4);
    assert_eq!(dependencies[0].ecosystem, Nix);
    assert_eq!(dependencies[0].group, "inputs");
    assert_eq!(dependencies[0].name, "NixOS/nixpkgs");
    assert_eq!(dependencies[0].requirement, "nixos-unstable");
    assert_eq!(
        dependencies[0].hosted_url,
        Some("https://api.github.com/repos/NixOS/nixpkgs/tags".to_owned())
    );
    assert_eq!(dependencies[0].hosted_name, Some("nixpkgs".to_owned()));
    assert_eq!(dependencies[1].name, "numtide/flake-utils");
    assert_eq!(dependencies[1].requirement, "v1.0.0");
    assert_eq!(dependencies[2].name, "nix-community/home-manager");
    assert_eq!(dependencies[2].requirement, "24.05");
    assert_eq!(dependencies[3].name, "local");
    assert_eq!(dependencies[3].hosted_url, Some("path".to_owned()));
    assert_eq!(dependencies[3].requirement, "path:../local");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "v1.0.0"
    );
}
