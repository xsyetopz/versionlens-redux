use jsonc_parser::ast::Value;
use jsonc_parser::{CollectOptions, ParseOptions, parse_to_ast};

const PACKAGE_JSON_KEYS: &[&str] = &[
    "dependencies",
    "devDependencies",
    "peerDependencies",
    "optionalDependencies",
    "bundledDependencies",
    "bundleDependencies",
    "overrides",
    "packageManager",
    "jspm",
    "pnpm",
    "workspaces",
];

pub(super) fn looks_like_package_json(text: &str) -> bool {
    let Ok(parse_result) = parse_to_ast(text, &CollectOptions::default(), &ParseOptions::default())
    else {
        return false;
    };
    let Some(Value::Object(root)) = parse_result.value else {
        return false;
    };

    PACKAGE_JSON_KEYS.iter().any(|key| root.get(key).is_some())
}
