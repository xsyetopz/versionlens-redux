use std::io::{Error as IoError, ErrorKind as IoErrorKind};

pub(crate) fn io_error_from_kind(kind: IoErrorKind) -> IoError {
    kind.into()
}
