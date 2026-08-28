pub(crate) use versionlens_suggestions::SuggestionStatus;
pub(crate) use versionlens_suggestions::SuggestionStatus::{
    BuildAvailable as StatusBuildAvailable, Current as StatusCurrent, Directory as StatusDirectory,
    DirectoryNotFound as StatusDirectoryNotFound, Error as StatusError, Fixed as StatusFixed,
    Invalid as StatusInvalid, InvalidRange as StatusInvalidRange, NoMatch as StatusNoMatch,
    NotSupported as StatusNotSupported, Satisfies as StatusSatisfies,
    SatisfiesLatest as StatusSatisfiesLatest, Unresolved as StatusUnresolved,
    UpdateAvailable as StatusUpdateAvailable,
};
