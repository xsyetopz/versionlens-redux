use versionlens_model::Dependency;
use versionlens_providers::provider_id;

pub(crate) fn dependency_signature(dependencies: &[Dependency]) -> String {
    let mut entries = dependencies
        .iter()
        .map(|dependency| {
            serde_json::json!([
                provider_id(dependency.ecosystem),
                dependency.name,
                dependency.group,
                dependency.requirement,
                dependency.hosted_url,
                dependency.hosted_name,
                dependency.canonical_reference,
            ])
            .to_string()
        })
        .collect::<Vec<_>>();
    entries.sort();
    entries.join("\n")
}

#[cfg(test)]
mod tests;
