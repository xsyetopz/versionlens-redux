use super::super::{RegistryErrorStatus, http_status_message_from_code};

pub(super) fn npm_status_error(status: &str) -> RegistryErrorStatus {
    if status == "128" {
        return RegistryErrorStatus::Error("not found".to_owned());
    }

    if let Some(message) = status
        .strip_prefix('E')
        .unwrap_or(status)
        .parse::<u16>()
        .ok()
        .and_then(http_status_message_from_code)
    {
        return RegistryErrorStatus::Error(message.to_owned());
    }

    RegistryErrorStatus::Error(status.to_owned())
}
