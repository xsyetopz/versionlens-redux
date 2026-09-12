use std::{fmt, io::Error};

use super::{
    WorkspaceDiscoveryFailure, WorkspaceDiscoveryFailureKind as Kind,
    WorkspaceDiscoveryIoOperation as Operation,
};

impl fmt::Display for WorkspaceDiscoveryFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(path) = &self.path {
            write!(formatter, "{}: ", path.display())?;
        }
        match &self.kind {
            Kind::Io {
                operation,
                error_kind,
            } => {
                let action = match operation {
                    Operation::ResolvePath => "resolve path",
                    Operation::ReadDirectory => "read directory",
                    Operation::ReadMetadata => "read file metadata",
                    Operation::ReadFile => "read file",
                };
                write!(
                    formatter,
                    "could not {action}: {}",
                    Error::from(*error_kind)
                )
            }
            Kind::InvalidOverlayUri | Kind::InvalidFileUri => {
                formatter.write_str("invalid file URI")
            }
            Kind::OutsideWorkspace => {
                formatter.write_str("document is outside the configured workspace")
            }
            Kind::EscapesWorkspace => formatter.write_str("path resolves outside the workspace"),
            Kind::InvalidUtf8 => formatter.write_str("file is not valid UTF-8"),
            Kind::FileTooLarge { size, limit } => write!(
                formatter,
                "file has {size} bytes; checking allows at most {limit} bytes"
            ),
            Kind::DepthLimitExceeded { limit } => write!(
                formatter,
                "workspace exceeds the directory depth limit of {limit}"
            ),
            Kind::FileLimitExceeded { limit } => write!(
                formatter,
                "workspace exceeds the checked file limit of {limit}"
            ),
            Kind::EntryLimitExceeded { limit } => write!(
                formatter,
                "workspace exceeds the visited entry limit of {limit}"
            ),
            Kind::Cancelled => formatter.write_str("workspace checking cancelled"),
        }
    }
}
