mod dotnet;
mod generic;
mod package;
mod python;

use versionlens_parsers::{Dependency, Ecosystem};

use dotnet::is_dotnet_property_version;
use generic::is_name_version_requirement;
use package::{
    is_cargo_package_version, is_deno_project_version, is_hex_project_version,
    is_maven_project_version, is_pub_version,
};
use python::is_python_project_version;
use versionlens_parsers::Ecosystem::{
    Cargo, Composer, Cran, Deno, Dotnet, Hackage, Hex, Julia, Maven, Npm, Opam, Pub, Python,
};

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
