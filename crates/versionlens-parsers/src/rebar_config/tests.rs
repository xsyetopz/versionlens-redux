use super::parse_rebar_config;

#[test]
fn parses_rebar_dependency_forms() {
    let dependencies = parse_rebar_config(
        r#"{deps,[
  {cowboy,"2.12.0"},
  {lager, {pkg, lager_fork}},
  {jsx, "3.1.0", {pkg, jsx_fork}},
  {gettext, {git, "https://github.com/elixir-lang/gettext.git", {tag, "0.26.2"}}}
]}.
"#,
    );

    assert_eq!(dependencies.len(), 4);
    assert_eq!(dependencies[0].name, "cowboy");
    assert_eq!(dependencies[0].requirement, "2.12.0");
    assert_eq!(dependencies[1].name, "lager_fork");
    assert_eq!(dependencies[1].hosted_name.as_deref(), Some("lager"));
    assert_eq!(dependencies[2].name, "jsx_fork");
    assert_eq!(dependencies[2].requirement, "3.1.0");
    assert_eq!(dependencies[3].hosted_url.as_deref(), Some("git"));
}

#[test]
fn parses_rebar_plugins() {
    let dependencies = parse_rebar_config(package_file_fixture("parses-rebar-plugins.txt"));

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "rebar_erl_vsn");
    assert_eq!(dependencies[0].requirement, "~> 0.1");
    assert_eq!(dependencies[0].group, "plugins");
}

#[test]
fn parses_rebar_project_plugins() {
    let dependencies = parse_rebar_config(package_file_fixture("parses-rebar-project-plugins.txt"));

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "rebar3_cuttlefish");
    assert_eq!(dependencies[0].requirement, "~> 0.1");
    assert_eq!(dependencies[0].group, "project_plugins");
}

#[test]
fn parses_rebar_profile_plugins() {
    let dependencies = parse_rebar_config(package_file_fixture("parses-rebar-profile-plugins.txt"));

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "rebar3_hex");
    assert_eq!(dependencies[0].requirement, "7.0.11");
    assert_eq!(dependencies[0].group, "plugins.test");
}

#[test]
fn parses_rebar_dependencies_from_compact_list() {
    let dependencies = parse_rebar_config(package_file_fixture(
        "parses-rebar-dependencies-from-compact-list.txt",
    ));

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].name, "cowboy");
    assert_eq!(dependencies[0].requirement, "2.12.0");
    assert_eq!(dependencies[1].name, "jsx");
    assert_eq!(dependencies[1].requirement, "3.1.0");
}

#[test]
fn parses_rebar_profile_dependencies() {
    let dependencies = parse_rebar_config(
        r#"{profiles, [
  {test, [
    {deps, [
      {meck, "1.0.0"}
    ]}
  ]}
]}.
"#,
    );

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "meck");
    assert_eq!(dependencies[0].requirement, "1.0.0");
    assert_eq!(dependencies[0].group, "deps.test");
}

#[test]
fn parses_rebar_compact_profile_dependencies() {
    let dependencies = parse_rebar_config(package_file_fixture(
        "parses-rebar-compact-profile-dependencies.txt",
    ));

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].name, "meck");
    assert_eq!(dependencies[0].requirement, "latest");
    assert_eq!(dependencies[0].group, "deps.test");
    assert_eq!(dependencies[1].name, "proper");
    assert_eq!(dependencies[1].requirement, "1.4.0");
    assert_eq!(dependencies[1].group, "deps.test");
}

#[test]
fn parses_rebar_bare_package_dependencies() {
    let dependencies = parse_rebar_config(package_file_fixture(
        "parses-rebar-bare-package-dependencies.txt",
    ));

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].name, "ranch");
    assert_eq!(dependencies[0].requirement, "latest");
    assert_eq!(dependencies[1].name, "cowboy");
    assert_eq!(dependencies[1].requirement, "2.12.0");
}

#[test]
fn parses_rebar_multiline_dependency_tuple() {
    let dependencies = parse_rebar_config(
        r#"{deps,[
  {gettext,
   {git, "https://github.com/elixir-lang/gettext.git",
    {tag, "0.26.2"}}}
]}.
"#,
    );

    crate::support::tests::assert_single_git_dependency(&dependencies, "gettext");
}

fn package_file_fixture(name: &str) -> &'static str {
    crate::support::tests::fixture(
        "tests/fixtures/versionlens-parsers/src/rebar_config/tests",
        name,
    )
}
