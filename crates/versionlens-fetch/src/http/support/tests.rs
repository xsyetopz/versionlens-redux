use std::io::{Error as IoError, ErrorKind as IoErrorKind};

pub(crate) fn io_error_from_kind(kind: IoErrorKind) -> IoError {
    kind.into()
}

#[test]
fn converts_io_error_kinds() {
    assert_eq!(
        io_error_from_kind(IoErrorKind::TimedOut).kind(),
        IoErrorKind::TimedOut
    );
}
