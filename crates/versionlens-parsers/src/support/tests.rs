use versionlens_model::Dependency;

pub(crate) fn fixture(base: &str, name: &str) -> &'static str {
    versionlens_test_support::static_fixture!(base, name).expect("test fixture must be readable")
}

pub(crate) struct DependencyExpectation<'a> {
    pub(crate) index: usize,
    pub(crate) ecosystem: versionlens_model::Ecosystem,
    pub(crate) group: &'a str,
    pub(crate) name: &'a str,
    pub(crate) requirement: &'a str,
}

impl<'a> DependencyExpectation<'a> {
    pub(crate) fn new(
        index: usize,
        ecosystem: versionlens_model::Ecosystem,
        group: &'a str,
        name: &'a str,
        requirement: &'a str,
    ) -> Self {
        Self {
            index,
            ecosystem,
            group,
            name,
            requirement,
        }
    }
}

pub(crate) fn assert_dependency(
    dependencies: &[versionlens_model::Dependency],
    expected: DependencyExpectation<'_>,
) {
    let dependency = &dependencies[expected.index];
    assert_eq!(dependency.ecosystem, expected.ecosystem);
    assert_eq!(dependency.group, expected.group);
    assert_eq!(dependency.name, expected.name);
    assert_eq!(dependency.requirement, expected.requirement);
}

pub(crate) fn assert_dependency_metadata(
    dependencies: &[Dependency],
    index: usize,
    group: &str,
    name: &str,
    requirement: &str,
) {
    assert_eq!(dependencies[index].group, group);
    assert_eq!(dependencies[index].name, name);
    assert_eq!(dependencies[index].requirement, requirement);
}

pub(crate) fn assert_dependency_group(
    dependencies: &[Dependency],
    expected_len: usize,
    index: usize,
    ecosystem: versionlens_model::Ecosystem,
    group: &str,
) {
    assert_eq!(dependencies.len(), expected_len);
    assert_eq!(dependencies[index].ecosystem, ecosystem);
    assert_eq!(dependencies[index].group, group);
}

pub(crate) fn assert_dependency_requirement_range(
    text: &str,
    dependencies: &[Dependency],
    index: usize,
    expected: &str,
) {
    assert_eq!(
        crate::document::test_support::extract_range(text, dependencies[index].requirement_range,),
        expected
    );
}

pub(crate) fn assert_two_dependency_requirements(
    dependencies: &[Dependency],
    first_name: &str,
    first_requirement: &str,
    second_name: &str,
    second_requirement: &str,
) {
    assert_eq!(dependencies[0].name, first_name);
    assert_eq!(dependencies[0].requirement, first_requirement);
    assert_eq!(dependencies[1].name, second_name);
    assert_eq!(dependencies[1].requirement, second_requirement);
}

pub(crate) fn assert_requirement_range_ends_at_dependency(
    dependencies: &[Dependency],
    index: usize,
) {
    assert_eq!(
        dependencies[index].requirement_range.start,
        dependencies[index].range.end
    );
    assert_eq!(
        dependencies[index].requirement_range.end,
        dependencies[index].range.end
    );
}

pub(crate) fn assert_single_git_dependency(dependencies: &[Dependency], name: &str) {
    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, name);
    assert_eq!(
        dependencies[0].requirement,
        "https://github.com/elixir-lang/gettext.git"
    );
    assert_eq!(dependencies[0].hosted_url.as_deref(), Some("git"));
}

pub(crate) fn assert_dependency_with_range(
    text: &str,
    dependencies: &[Dependency],
    expected: DependencyExpectation<'_>,
    extracted_range: &str,
) {
    let index = expected.index;
    assert_dependency(dependencies, expected);
    assert_dependency_requirement_range(text, dependencies, index, extracted_range);
}

pub(crate) fn parse_test_document(
    text: &str,
    uri: &str,
    language: &str,
) -> Vec<versionlens_model::Dependency> {
    crate::parse_document(&versionlens_model::DocumentInput::new(
        uri.to_owned(),
        language.to_owned(),
        text.to_owned(),
        None,
    ))
}

pub(crate) fn assert_auth_entries(entries: &[crate::NpmAuthEntry], expected: &[(&str, &str)]) {
    assert_eq!(entries.len(), expected.len());
    for (entry, (registry, value)) in entries.iter().zip(expected) {
        assert_eq!(entry.registry, *registry);
        assert_eq!(entry.header_value, *value);
    }
}

pub(crate) fn assert_named_dependency(
    dependencies: &[versionlens_model::Dependency],
    index: usize,
    name: &str,
    requirement: &str,
) {
    assert_eq!(dependencies[index].name, name);
    assert_eq!(dependencies[index].requirement, requirement);
}
