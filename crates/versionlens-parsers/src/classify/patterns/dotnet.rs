use super::super::uri::{file_name, has_extension};

pub(super) fn is_dotnet_xml_uri(uri: &str) -> bool {
    if is_dotnet_generated_uri(uri) {
        return false;
    }

    matches!(
        file_name(uri),
        Some(name)
            if has_extension(name, ["csproj", "fsproj", "vbproj", "targets", "props"])
    )
}

fn is_dotnet_generated_uri(uri: &str) -> bool {
    uri.split('/')
        .any(|segment| segment.eq_ignore_ascii_case("obj") || segment.eq_ignore_ascii_case("bin"))
}
