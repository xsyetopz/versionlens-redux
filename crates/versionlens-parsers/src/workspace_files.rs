//! Facts that the workspace and the action resolvers read from TOML and YAML
//! files: workspace member patterns, package identities and action validity.

use marked_yaml::parse_yaml;
use marked_yaml::types::Node as YamlNode;
use toml_edit::{DocumentMut, Item, Value};

/// The `members` and `exclude` patterns of a Cargo `[workspace]` table.
pub struct CargoWorkspaceMembers {
    pub members: Vec<String>,
    pub exclude: Vec<String>,
}

pub fn parse_cargo_workspace_members(text: &str) -> Option<CargoWorkspaceMembers> {
    let document = text.parse::<DocumentMut>().ok()?;
    let workspace = document.get("workspace")?.as_table_like()?;
    Some(CargoWorkspaceMembers {
        members: toml_strings(workspace.get("members")),
        exclude: toml_strings(workspace.get("exclude")),
    })
}

fn toml_strings(item: Option<&Item>) -> Vec<String> {
    item.and_then(Item::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// The `[workspace.package] version` that member crates may inherit.
pub fn parse_cargo_workspace_version(text: &str) -> Option<String> {
    text.parse::<DocumentMut>()
        .ok()?
        .get("workspace")?
        .get("package")?
        .get("version")?
        .as_str()
        .map(str::to_owned)
}

/// The name and version of a Cargo package. A `version.workspace = true`
/// package takes `inherited_version`.
pub fn parse_cargo_package_identity(
    text: &str,
    inherited_version: Option<&str>,
) -> Option<(String, String)> {
    let document = text.parse::<DocumentMut>().ok()?;
    let package = document.get("package")?;
    let version = package.get("version")?;
    let version = version.as_str().map(str::to_owned).or_else(|| {
        version
            .as_table_like()?
            .get("workspace")?
            .as_bool()
            .filter(|inherited| *inherited)
            .and(inherited_version.map(str::to_owned))
    })?;
    Some((package.get("name")?.as_str()?.to_owned(), version))
}

/// The `packages` patterns of a `pnpm-workspace.yaml`.
pub fn parse_pnpm_workspace_packages(text: &str) -> Option<Vec<String>> {
    let document = parse_yaml(0, text).ok()?;
    let root = document.as_mapping()?;
    let YamlNode::Sequence(values) = root.get_node("packages")? else {
        return None;
    };
    Some(
        values
            .iter()
            .filter_map(|value| {
                let YamlNode::Scalar(value) = value else {
                    return None;
                };
                Some(value.as_str().to_owned())
            })
            .collect(),
    )
}

/// Whether an `action.yml` declares a `runs.using` runtime.
pub fn action_manifest_is_valid(text: &str) -> bool {
    let Ok(document) = parse_yaml(0, text) else {
        return false;
    };
    document
        .as_mapping()
        .and_then(|root| root.get_mapping("runs"))
        .and_then(|runs| runs.get_scalar("using"))
        .is_some_and(|using| !using.as_str().trim().is_empty())
}

/// Whether a workflow file is triggered by `workflow_call`.
pub fn reusable_workflow_is_valid(text: &str) -> bool {
    let Ok(document) = parse_yaml(0, text) else {
        return false;
    };
    let Some(root) = document.as_mapping() else {
        return false;
    };
    if root
        .get_scalar("on")
        .is_some_and(|event| event.as_str() == "workflow_call")
    {
        return true;
    }
    if root
        .get_mapping("on")
        .is_some_and(|events| events.get_node("workflow_call").is_some())
    {
        return true;
    }
    root.get_sequence("on").is_some_and(|events| {
        events.iter().any(|event| {
            event
                .as_scalar()
                .is_some_and(|event| event.as_str() == "workflow_call")
        })
    })
}
