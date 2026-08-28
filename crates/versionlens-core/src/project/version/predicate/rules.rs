use versionlens_model::Ecosystem::{
    Cargo, Composer, Cran, Deno, Dotnet, Hackage, Hex, Julia, Maven, Npm, Opam, Pub, Python,
};
use versionlens_model::{Dependency, Ecosystem};

type ProjectVersionPredicate = fn(&Dependency) -> bool;

const PROJECT_VERSION_PREDICATES: &[(Ecosystem, ProjectVersionPredicate)] = &[
    (Cargo, is_cargo_package_version),
    (Composer, is_name_version_requirement),
    (Deno, is_deno_project_version),
    (Dotnet, is_dotnet_property_version),
    (Hex, is_hex_project_version),
    (Hackage, is_hex_project_version),
    (Julia, is_hex_project_version),
    (Cran, is_hex_project_version),
    (Maven, is_maven_project_version),
    (Opam, is_hex_project_version),
    (Npm, is_name_version_requirement),
    (Python, is_python_project_version),
    (Pub, is_pub_version),
];

pub(super) fn project_version_match(dependency: &Dependency) -> bool {
    PROJECT_VERSION_PREDICATES
        .iter()
        .find_map(|(ecosystem, predicate)| {
            (*ecosystem == dependency.ecosystem).then(|| predicate(dependency))
        })
        .unwrap_or(false)
}

pub(super) fn is_dotnet_property_version(dependency: &Dependency) -> bool {
    dependency.group == "PropertyGroup"
        && matches!(dependency.name.as_str(), "Version" | "AssemblyVersion")
}

pub(super) fn is_name_version_requirement(dependency: &Dependency) -> bool {
    dependency.group == "version" && dependency.name == dependency.requirement
}

pub(super) fn is_cargo_package_version(dependency: &Dependency) -> bool {
    dependency.group == "package" && dependency.name == "version"
}

pub(super) fn is_deno_project_version(dependency: &Dependency) -> bool {
    dependency.group == "version" && dependency.name.starts_with('@')
}

pub(super) fn is_hex_project_version(dependency: &Dependency) -> bool {
    dependency.group == "version" && !dependency.name.is_empty()
}

pub(super) fn is_maven_project_version(dependency: &Dependency) -> bool {
    dependency.group == "project.version" && dependency.name == "version"
}

pub(super) fn is_pub_version(dependency: &Dependency) -> bool {
    dependency.group == "version" && dependency.name == "version"
}

pub(super) fn is_python_project_version(dependency: &Dependency) -> bool {
    dependency.group == "project" && dependency.name == "version"
}
