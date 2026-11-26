//! Master's checked-in manifest fixtures, from `tests/fixtures/` of the fork's master, read whole:
//! each test lists what master's `document.rs` expected of its fixture.

use super::expect;

#[test]
fn reads_ansible_galaxy_requirements() {
    let text = "---\nroles:\n  - name: geerlingguy.java\n    version: \"1.9.6\"\n  - src: https://github.com/bennojoy/nginx\n    version: main\n  - include: webserver.yml\ncollections:\n  - name: community.general\n    version: \">=7.0.0\"\n    source: https://galaxy.ansible.com\n  - name: acme.private\n    version: \"1.2.3\"\n    source: https://galaxy.example.test\n";
    expect(
        "requirements.yml",
        text,
        &[
            ("geerlingguy.java", "1.9.6", "galaxy-role"),
            ("https://github.com/bennojoy/nginx", "main", "source"),
            ("community.general", ">=7.0.0", "galaxy"),
            ("acme.private", "1.2.3", "galaxy"),
        ],
    );
}

#[test]
fn reads_bazel_module_dependencies_and_overrides() {
    let text = "module(name = \"demo\", version = \"1.0.0\")\n\nbazel_dep(name = \"rules_cc\", version = \"0.0.9\")\nbazel_dep(\n    name = \"bazel_skylib\",\n    version = \"1.3.0\",\n    repo_name = \"skylib\",\n    dev_dependency = True,\n)\nsingle_version_override(module_name = \"protobuf\", version = \"29.0\", registry = \"https://bcr.example.test\")\ngit_override(module_name = \"rules_local\", remote = \"https://github.com/example/rules_local.git\", commit = \"abc123\")\nlocal_path_override(module_name = \"local_rules\", path = \"../local_rules\")\n";
    expect(
        "MODULE.bazel",
        text,
        &[
            ("rules_cc", "0.0.9", "bazel"),
            ("bazel_skylib", "1.3.0", "bazel"),
            ("protobuf", "29.0", "bazel"),
            ("rules_local", "abc123", "source"),
            ("local_rules", "../local_rules", "directory"),
        ],
    );
}

#[test]
fn reads_cabal_dependencies() {
    let text = "cabal-version: 3.0\nname: demo\nversion: 0.1.0.0\n\nlibrary\n  build-depends:\n      base >= 4.14 && < 5\n    , text ^>= 2.0\n\ntest-suite demo-test\n  build-depends: tasty >= 1.4\n";
    expect(
        "demo.cabal",
        text,
        &[
            ("cabal", "0.1.0.0", "project"),
            ("base", ">= 4.14", "hackage"),
            ("text", "^>= 2.0", "hackage"),
            ("tasty", ">= 1.4", "hackage"),
        ],
    );
}

#[test]
fn reads_cpanfile_dependencies() {
    let text = "requires 'Plack', '1.0';\nrequires 'JSON', '>= 2.00, < 2.80';\nrecommends 'JSON::XS', '2.0';\nconflicts 'JSON', '< 1.0';\n\non 'test' => sub {\n  requires 'Test::More', '>= 0.96, < 2.0';\n};\n\nfeature 'sqlite', 'SQLite support' => sub {\n  recommends 'DBD::SQLite';\n};\n\nbuild_requires 'Module::Build', '0.42';\n";
    expect(
        "cpanfile",
        text,
        &[
            ("Plack", "1.0", "cpan"),
            ("JSON", ">= 2.00, < 2.80", "cpan"),
            ("JSON::XS", "2.0", "cpan"),
            ("JSON", "< 1.0", "cpan"),
            ("Test::More", ">= 0.96, < 2.0", "cpan"),
            ("DBD::SQLite", "", "cpan"),
            ("Module::Build", "0.42", "cpan"),
        ],
    );
}

#[test]
fn reads_dune_project_package_dependencies() {
    let text = "(lang dune 3.17)\n(name demo)\n\n(package\n (name demo)\n (version 1.2.3)\n (depends\n  (ocaml (>= 4.14))\n  (fmt (and (>= 0.6) (< 1.0)))\n  lwt))\n";
    expect(
        "dune-project",
        text,
        &[
            ("dune", "1.2.3", "project"),
            ("ocaml", ">= 4.14", "opam"),
            ("fmt", ">= 0.6", "opam"),
            ("lwt", "", "opam"),
        ],
    );
}

#[test]
fn reads_gleam_toml_dependencies() {
    let text = "name = \"demo\"\nversion = \"1.0.0\"\n\n[dependencies]\ngleam_stdlib = \">= 0.44.0 and < 2.0.0\"\nmy_other_project = { path = \"../my_other_project\" }\nmy_library = { git = \"https://github.com/my-project/my-library\", ref = \"a8b3c5d82\" }\n\n[dev_dependencies]\ngleeunit = \">= 1.0.0 and < 2.0.0\"\n";
    expect(
        "gleam.toml",
        text,
        &[
            ("gleam", "1.0.0", "project"),
            ("gleam_stdlib", ">= 0.44.0 and < 2.0.0", "hex"),
            ("my_other_project", "../my_other_project", "directory"),
            (
                "my_library",
                "https://github.com/my-project/my-library",
                "source",
            ),
            ("gleeunit", ">= 1.0.0 and < 2.0.0", "hex"),
        ],
    );
}

#[test]
fn reads_julia_project_dependencies_and_sources() {
    let text = "name = \"Demo\"\nuuid = \"11111111-1111-1111-1111-111111111111\"\nversion = \"0.1.0\"\n\n[deps]\nExample = \"7876af07-990d-54b4-ab0e-23690620f79a\"\nLocalPkg = \"22222222-2222-2222-2222-222222222222\"\nGitPkg = \"33333333-3333-3333-3333-333333333333\"\n\n[compat]\nExample = \"1.2\"\nLocalPkg = \"0.1\"\nGitPkg = \"0.3\"\njulia = \"1.10\"\n\n[sources]\nLocalPkg = { path = \"deps/LocalPkg\" }\nGitPkg = { url = \"https://github.com/example/GitPkg.jl\", rev = \"main\" }\n";
    expect(
        "Project.toml",
        text,
        &[
            ("julia", "0.1.0", "project"),
            ("Example", "1.2", "julia"),
            ("LocalPkg", "0.1", "julia"),
            ("GitPkg", "0.3", "julia"),
            ("julia", "1.10", "julia-lang"),
            ("LocalPkg", "deps/LocalPkg", "directory"),
            ("GitPkg", "https://github.com/example/GitPkg.jl", "source"),
        ],
    );
}

#[test]
fn reads_mix_exs_dependencies() {
    let text = "defmodule Demo.MixProject do\n  use Mix.Project\n\n  defp deps do\n    [\n      {:plug, \">= 1.15.0\"},\n      {:phoenix, \"~> 1.7\", only: [:dev, :test]},\n      {:gettext, git: \"https://github.com/elixir-lang/gettext.git\", tag: \"0.26.2\"},\n      {:local_dependency, path: \"../local_dependency\"}\n    ]\n  end\nend\n";
    expect(
        "mix.exs",
        text,
        &[
            ("plug", ">= 1.15.0", "hex"),
            ("phoenix", "~> 1.7", "hex"),
            (
                "gettext",
                "https://github.com/elixir-lang/gettext.git",
                "source",
            ),
            ("local_dependency", "../local_dependency", "directory"),
        ],
    );
}

#[test]
fn reads_opam_dependencies() {
    let text = "opam-version: \"2.0\"\nname: \"demo\"\nversion: \"1.2.3\"\ndepends: [\n  \"ocaml\" {>= \"4.14\"}\n  \"dune\" {>= \"3.18\" & < \"4.0\"}\n  \"lwt\" {>= \"5.9\" & < \"6.0\" & with-test}\n]\ndepopts: [\n  \"conf-libev\"\n]\nconflicts: [\n  \"oldpkg\" {< \"1.0\"}\n]\n";
    expect(
        "demo.opam",
        text,
        &[
            ("opam", "1.2.3", "project"),
            ("ocaml", "4.14", "opam"),
            ("dune", "3.18", "opam"),
            ("lwt", "5.9", "opam"),
            ("conf-libev", "", "opam"),
            ("oldpkg", "", "opam"),
        ],
    );
}

#[test]
fn reads_r_description_dependencies() {
    let text = "Package: demo\nVersion: 0.1.0\nDepends: R (>= 4.3), dplyr (>= 1.1.0),\n    tidyr\nImports: cli (>= 3.6.0)\nSuggests: testthat (>= 3.0.0)\nLinkingTo: Rcpp (>= 1.0.0)\n";
    expect(
        "DESCRIPTION",
        text,
        &[
            ("r", "0.1.0", "project"),
            ("R", ">= 4.3", "r-lang"),
            ("dplyr", ">= 1.1.0", "cran"),
            ("tidyr", "", "cran"),
            ("cli", ">= 3.6.0", "cran"),
            ("testthat", ">= 3.0.0", "cran"),
            ("Rcpp", ">= 1.0.0", "cran"),
        ],
    );
}

#[test]
fn reads_stack_yaml_extra_deps() {
    let text = "resolver: lts-22.43\nextra-deps:\n  - acme-missiles-0.3\n  - text-2.1.1@rev:0\n  - ./vendor/local-package\n  - git: https://github.com/example/package.git\n    commit: abcdef\n";
    expect(
        "stack.yaml",
        text,
        &[
            ("lts", "22.43", "stackage"),
            ("acme-missiles", "0.3", "hackage"),
            ("text", "2.1.1", "hackage"),
            (
                "./vendor/local-package",
                "./vendor/local-package",
                "directory",
            ),
            ("https://github.com/example/package.git", "abcdef", "source"),
        ],
    );
}
