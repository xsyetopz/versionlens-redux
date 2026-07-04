use serde_json::Value;

pub(super) fn npm_response_status(body: &str) -> Option<String> {
    let value = serde_json::from_str::<Value>(body).ok()?;
    let status = value
        .get("status")
        .or_else(|| value.get("code"))
        .or_else(|| value.pointer("/error/code"))?;

    status
        .as_str()
        .map(str::to_owned)
        .or_else(|| status.as_u64().map(|status| status.to_string()))
}
