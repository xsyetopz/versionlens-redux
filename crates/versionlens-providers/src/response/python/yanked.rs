use serde_json::Value;

pub(super) fn python_release_is_yanked(value: &Value, version: &str) -> bool {
    value
        .get("releases")
        .and_then(Value::as_object)
        .and_then(|releases| releases.get(version))
        .and_then(Value::as_array)
        .is_some_and(|files| {
            !files.is_empty()
                && files
                    .iter()
                    .all(|file| file.get("yanked").and_then(Value::as_bool).unwrap_or(false))
        })
}
