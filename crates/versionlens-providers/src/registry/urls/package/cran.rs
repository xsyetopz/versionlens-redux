use super::super::trim_end_slash;

pub(in crate::registry::urls) fn cran_registry_url(_: &str) -> String {
    "https://cloud.r-project.org/src/contrib/PACKAGES.gz".to_owned()
}

pub(in crate::registry::urls) fn cran_registry_url_with_base(base_url: &str, _: &str) -> String {
    let base_url = trim_end_slash(base_url);
    if base_url.ends_with("/PACKAGES")
        || base_url.ends_with("/PACKAGES.gz")
        || base_url.ends_with("/PACKAGES.rds")
    {
        return base_url.to_owned();
    }
    format!("{base_url}/src/contrib/PACKAGES")
}
