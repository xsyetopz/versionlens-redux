const PACKAGE_TAGS: &[&str] = &[
    "PackageReference",
    "PackageVersion",
    "GlobalPackageReference",
    "DotNetCliToolReference",
];

pub(in crate::dotnet_xml) fn is_package_tag(tag_name: &str) -> bool {
    PACKAGE_TAGS.contains(&tag_name)
}
