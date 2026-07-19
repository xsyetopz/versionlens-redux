use std::ops::Range as ByteRange;

use toml_edit::{DocumentMut, InlineTable, Key, Value as TomlValue};

use crate::positions::offset_range;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Maven;

type VersionRequirement<'a> = Option<(&'a str, Option<ByteRange<usize>>, Option<String>)>;
type InlineVersionRequirement<'a> = Option<(
    &'a str,
    Option<ByteRange<usize>>,
    Option<&'static str>,
    String,
)>;
type GradleDependencyReader =
    for<'a> fn(&'a str, &'a Key, &'a str, &'a TomlValue) -> Option<GradleDependencyInput<'a>>;
type GradleVersionCatalogDependencies = Vec<Dependency>;

pub(crate) fn parse_gradle_version_catalog_toml(text: &str) -> Vec<Dependency> {
    let Ok(document) = text.parse::<DocumentMut>() else {
        return vec![];
    };

    let mut dependencies = vec![];
    collect_versions(text, &document, &mut dependencies);
    collect_alias_table(
        text,
        &document,
        &mut dependencies,
        "libraries",
        gradle_library_dependency,
    );
    collect_alias_table(
        text,
        &document,
        &mut dependencies,
        "plugins",
        gradle_plugin_dependency,
    );
    dependencies
}

fn collect_versions(
    text: &str,
    document: &DocumentMut,
    dependencies: &mut GradleVersionCatalogDependencies,
) {
    let Some(table) = document.get("versions").and_then(|value| value.as_table()) else {
        return;
    };

    for (name, item) in table.iter() {
        let Some(key) = table.key(name) else {
            continue;
        };
        let Some(value) = item.as_value() else {
            continue;
        };
        let Some((requirement, span, prefix)) = gradle_version_requirement(value) else {
            continue;
        };
        dependencies.push(gradle_dependency(GradleDependencyInput {
            text,
            group: "versions",
            name: name.to_owned(),
            requirement,
            hosted_url: Some("version.alias"),
            hosted_name: None,
            range_span: key.span(),
            requirement_span: span,
            requirement_prefix: prefix.unwrap_or_default(),
        }));
    }
}

fn collect_alias_table(
    text: &str,
    document: &DocumentMut,
    dependencies: &mut GradleVersionCatalogDependencies,
    table_name: &str,
    dependency: GradleDependencyReader,
) {
    let Some(table) = document.get(table_name).and_then(|value| value.as_table()) else {
        return;
    };

    for (alias, item) in table.iter() {
        let Some(key) = table.key(alias) else {
            continue;
        };
        let Some(value) = item.as_value() else {
            continue;
        };
        let Some(input) = dependency(text, key, alias, value) else {
            continue;
        };
        dependencies.push(gradle_dependency(input));
    }
}

struct GradleDependencyInput<'a> {
    text: &'a str,
    group: &'static str,
    name: String,
    requirement: &'a str,
    hosted_url: Option<&'static str>,
    hosted_name: Option<&'a str>,
    range_span: Option<ByteRange<usize>>,
    requirement_span: Option<ByteRange<usize>>,
    requirement_prefix: String,
}

fn gradle_dependency(input: GradleDependencyInput<'_>) -> Dependency {
    let range = input
        .range_span
        .map(|span| offset_range(input.text, span.start, span.end))
        .unwrap_or_else(|| offset_range(input.text, 0, 0));
    let requirement_range = input
        .requirement_span
        .map(|span| {
            let content = string_content_bounds(input.text, span.start, span.end);
            offset_range(input.text, content.start, content.end)
        })
        .unwrap_or(range);

    Dependency {
        name: input.name,
        requirement: input.requirement.to_owned(),
        ecosystem: Maven,
        group: input.group.to_owned(),
        hosted_url: input.hosted_url.map(|value| value.to_owned()),
        hosted_name: input.hosted_name.map(|value| value.to_owned()),
        range,
        requirement_range,
        requirement_prefix: input.requirement_prefix,
        requirement_suffix: "".to_owned(),
    }
}

fn gradle_version_requirement(value: &TomlValue) -> VersionRequirement<'_> {
    if let Some(requirement) = value.as_str() {
        return Some((requirement, value.span(), None));
    }

    let inline = value.as_inline_table()?;
    for field in ["prefer", "require", "strictly"] {
        if let Some(version) = inline.get(field)
            && let Some(requirement) = version.as_str()
        {
            return Some((requirement, version.span(), Some(format!("{field} = \""))));
        }
    }

    None
}

fn gradle_library_dependency<'a>(
    text: &'a str,
    key: &'a Key,
    _alias: &'a str,
    value: &'a TomlValue,
) -> Option<GradleDependencyInput<'a>> {
    if let Some(notation) = value.as_str() {
        let (module, requirement) = gradle_module_notation(notation)?;
        let value_span = value.span()?;
        let requirement_start = text
            .get(value_span.start..value_span.end)?
            .find(requirement)
            .map(|index| value_span.start + index)?;
        return Some(GradleDependencyInput {
            text,
            group: "libraries",
            name: module.to_owned(),
            requirement,
            hosted_url: None,
            hosted_name: None,
            range_span: key.span(),
            requirement_span: Some(requirement_start..requirement_start + requirement.len()),
            requirement_prefix: "".to_owned(),
        });
    }

    let inline = value.as_inline_table()?;
    let name = inline_module_name(inline)?;
    let (requirement, requirement_span, hosted_url, requirement_prefix) =
        inline_version_requirement(inline)?;
    Some(GradleDependencyInput {
        text,
        group: "libraries",
        name,
        requirement,
        hosted_url,
        hosted_name: None,
        range_span: key.span(),
        requirement_span,
        requirement_prefix,
    })
}

fn gradle_plugin_dependency<'a>(
    text: &'a str,
    key: &'a Key,
    _alias: &'a str,
    value: &'a TomlValue,
) -> Option<GradleDependencyInput<'a>> {
    if let Some(notation) = value.as_str() {
        let (id, requirement) = notation.rsplit_once(':')?;
        let value_span = value.span()?;
        let requirement_start = text
            .get(value_span.start..value_span.end)?
            .find(requirement)
            .map(|index| value_span.start + index)?;
        return Some(GradleDependencyInput {
            text,
            group: "plugins",
            name: gradle_plugin_marker_name(id),
            requirement,
            hosted_url: None,
            hosted_name: None,
            range_span: key.span(),
            requirement_span: Some(requirement_start..requirement_start + requirement.len()),
            requirement_prefix: "".to_owned(),
        });
    }

    let inline = value.as_inline_table()?;
    let id = inline.get("id")?.as_str()?;
    let (requirement, requirement_span, hosted_url, requirement_prefix) =
        inline_version_requirement(inline)?;
    Some(GradleDependencyInput {
        text,
        group: "plugins",
        name: gradle_plugin_marker_name(id),
        requirement,
        hosted_url,
        hosted_name: None,
        range_span: key.span(),
        requirement_span,
        requirement_prefix,
    })
}

fn gradle_plugin_marker_name(id: &str) -> String {
    format!("{id}:{id}.gradle.plugin")
}

fn inline_module_name(inline: &InlineTable) -> Option<String> {
    if let Some(module) = inline.get("module").and_then(|value| value.as_str()) {
        return Some(module.to_owned());
    }

    let group = inline.get("group")?.as_str()?;
    let artifact = inline.get("name")?.as_str()?;
    Some(format!("{group}:{artifact}"))
}

fn inline_version_requirement(inline: &InlineTable) -> InlineVersionRequirement<'_> {
    if let Some(version) = inline.get("version") {
        if let Some(requirement) = version.as_str() {
            return Some((requirement, version.span(), None, "".to_owned()));
        }
        if let Some((requirement, span, prefix)) = gradle_version_requirement(version) {
            return Some((requirement, span, None, prefix.unwrap_or_default()));
        }
        if let Some(version_ref) = version
            .as_inline_table()
            .and_then(|version| version.get("ref"))
            .and_then(|value| value.as_str())
        {
            return Some((
                version_ref,
                version
                    .as_inline_table()
                    .and_then(|version| version.get("ref"))
                    .and_then(|value| value.span()),
                Some("version.ref"),
                "".to_owned(),
            ));
        }
    }

    if let Some(version_ref) = inline.get("version.ref").and_then(|value| value.as_str()) {
        return Some((
            version_ref,
            inline.get("version.ref").and_then(|value| value.span()),
            Some("version.ref"),
            "".to_owned(),
        ));
    }
    if let Some(version_ref) = inline
        .get("version")
        .and_then(|value| value.as_inline_table())
        .and_then(|version| version.get("ref"))
        .and_then(|value| value.as_str())
    {
        return Some((
            version_ref,
            inline
                .get("version")
                .and_then(|value| value.as_inline_table())
                .and_then(|version| version.get("ref"))
                .and_then(|value| value.span()),
            Some("version.ref"),
            "".to_owned(),
        ));
    }

    None
}

fn gradle_module_notation(notation: &str) -> Option<(&str, &str)> {
    let (group, rest) = notation.split_once(':')?;
    let (artifact, version) = rest.rsplit_once(':')?;
    let module_end = group.len() + 1 + artifact.len();
    Some((&notation[..module_end], version))
}

fn string_content_bounds(text: &str, start: usize, end: usize) -> ByteRange<usize> {
    let content_start = start + usize::from(text.as_bytes().get(start) == Some(&b'"'));
    let content_end = end.saturating_sub(usize::from(
        end > start && text.as_bytes().get(end - 1) == Some(&b'"'),
    ));
    content_start..content_end
}
