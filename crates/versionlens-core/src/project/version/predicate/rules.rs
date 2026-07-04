mod dotnet;
mod generic;
mod package;
mod python;

use versionlens_parsers::{Dependency, Ecosystem};

use dotnet::is_dotnet_property_version;
use generic::is_name_version_requirement;
use package::{is_cargo_package_version, is_maven_project_version, is_pub_version};
use python::is_python_project_version;

type ProjectVersionPredicate = fn(&Dependency) -> bool;

const PROJECT_VERSION_PREDICATES: &[(Ecosystem, ProjectVersionPredicate)] = &[
    (Ecosystem::Cargo, is_cargo_package_version),
    (Ecosystem::Composer, is_name_version_requirement),
    (Ecosystem::Dotnet, is_dotnet_property_version),
    (Ecosystem::Maven, is_maven_project_version),
    (Ecosystem::Npm, is_name_version_requirement),
    (Ecosystem::Python, is_python_project_version),
    (Ecosystem::Pub, is_pub_version),
];

pub(super) fn project_version_match(dependency: &Dependency) -> bool {
    PROJECT_VERSION_PREDICATES
        .iter()
        .find_map(|(ecosystem, predicate)| {
            (*ecosystem == dependency.ecosystem).then(|| predicate(dependency))
        })
        .unwrap_or(false)
}
