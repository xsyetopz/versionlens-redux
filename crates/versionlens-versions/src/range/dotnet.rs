use crate::parse::parse_version;

use super::requirements::nuget_requirement;

pub fn is_dotnet_requirement_parseable(requirement: &str) -> bool {
    let requirement = requirement.trim();
    !requirement.is_empty()
        && (parse_version(requirement).is_some() || nuget_requirement(requirement).is_some())
}
