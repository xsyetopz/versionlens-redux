use versionlens_model::Ecosystem::Cpp;

#[test]
fn parses_cpp_cmake_dependencies() {
    let text = package_file_fixture("parses-cpp-cmake-dependencies.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/CMakeLists.txt".to_owned(),
        language_id: "cmake".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 4);
    assert_eq!(dependencies[0].ecosystem, Cpp);
    assert_eq!(dependencies[0].group, "FetchContent_Declare");
    assert_eq!(dependencies[0].name, "fmt");
    assert_eq!(dependencies[0].requirement, "10.2.1");
    assert_eq!(dependencies[0].hosted_name.as_deref(), Some("fmtlib/fmt"));
    assert_eq!(dependencies[1].group, "ExternalProject_Add");
    assert_eq!(dependencies[1].name, "zlib_project");
    assert_eq!(dependencies[1].requirement, "v1.3.1");
    assert_eq!(dependencies[2].group, "CPMAddPackage");
    assert_eq!(dependencies[2].name, "Catch2");
    assert_eq!(extract_range(text, dependencies[2].range), "Catch2");
    assert_eq!(dependencies[2].requirement, "v3.6.0");
    assert_eq!(dependencies[3].group, "CPMAddPackage");
    assert_eq!(dependencies[3].name, "spdlog");
    assert_eq!(extract_range(text, dependencies[3].range), "spdlog");
    assert_eq!(dependencies[3].requirement, "1.14.1");
    assert_eq!(dependencies[3].hosted_name.as_deref(), Some("gabime/spdlog"));
}

#[test]
fn parses_cpp_xmake_dependencies() {
    let text = package_file_fixture("parses-cpp-xmake-dependencies.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/xmake.lua".to_owned(),
        language_id: "lua".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].ecosystem, Cpp);
    assert_eq!(dependencies[0].group, "add_requires");
    assert_eq!(dependencies[0].name, "zlib");
    assert_eq!(dependencies[0].requirement, "1.3.1");
    assert_eq!(dependencies[1].name, "fmt");
    assert_eq!(dependencies[1].requirement, "10.2.1");
    assert_eq!(dependencies[2].name, "openssl");
    assert_eq!(dependencies[2].requirement, "*");
    assert_eq!(dependencies[2].requirement_prefix, " ");
    assert_eq!(extract_range(text, dependencies[2].requirement_range), "");
}

#[test]
fn parses_cpp_meson_wrap_dependency() {
    let text = package_file_fixture("parses-cpp-meson-wrap-dependency.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/subprojects/gtest.wrap".to_owned(),
        language_id: "meson".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].ecosystem, Cpp);
    assert_eq!(dependencies[0].group, "meson.wrap");
    assert_eq!(dependencies[0].name, "googletest-1.14.0");
    assert_eq!(dependencies[0].requirement, "v1.14.0");
    assert_eq!(extract_range(text, dependencies[0].requirement_range), "v1.14.0");
    assert_eq!(dependencies[0].hosted_name.as_deref(), Some("google/googletest"));
}

#[test]
fn parses_cpp_bazel_workspace_dependency() {
    let text = package_file_fixture("parses-cpp-bazel-workspace-dependency.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/WORKSPACE".to_owned(),
        language_id: "starlark".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].ecosystem, Cpp);
    assert_eq!(dependencies[0].group, "http_archive");
    assert_eq!(dependencies[0].name, "rules_cc");
    assert_eq!(dependencies[0].requirement, "0.0.9");
    assert_eq!(dependencies[0].hosted_name.as_deref(), Some("bazelbuild/rules_cc"));
}
