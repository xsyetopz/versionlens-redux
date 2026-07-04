use super::super::RegistryErrorStatus;

type RegistryErrorBuilder = fn(String) -> RegistryErrorStatus;
type NpmErrorStatusSpec = (&'static str, RegistryErrorBuilder, &'static str);

const NPM_ERROR_STATUSES: &[NpmErrorStatusSpec] = &[
    (
        "ECONNREFUSED",
        RegistryErrorStatus::Error,
        "connection refused",
    ),
    ("ECONNRESET", RegistryErrorStatus::Error, "connection reset"),
    ("EUNSUPPORTEDPROTOCOL", not_supported, "not supported"),
    (
        "EINVALIDTAGNAME",
        RegistryErrorStatus::InvalidWithLatest,
        "invalid version",
    ),
    (
        "EINVALIDPACKAGENAME",
        RegistryErrorStatus::Invalid,
        "invalid version",
    ),
];

fn not_supported(_: String) -> RegistryErrorStatus {
    RegistryErrorStatus::NotSupported
}

pub(super) fn npm_known_error_status(status: &str) -> Option<RegistryErrorStatus> {
    NPM_ERROR_STATUSES
        .iter()
        .find_map(|(code, build, message)| (*code == status).then(|| build((*message).to_owned())))
}
