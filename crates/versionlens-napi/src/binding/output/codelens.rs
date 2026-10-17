use napi_derive::napi;
use versionlens_core::{Action, LensView};

use crate::binding::position::{NativeRange, native_range_from_core};

const UPDATE_DEPENDENCY_COMMAND: &str = "versionlens.suggestion.onUpdateDependency";
const CHOOSE_BUILD_COMMAND: &str = "versionlens.suggestion.onChooseBuild";
const FILE_LINK_COMMAND: &str = "versionlens.suggestion.onFileLink";

#[napi(object)]
pub struct NativeCodeLensPayload {
    pub range: NativeRange,
    pub title: String,
    pub command: String,
    pub arguments: Vec<String>,
}

impl NativeCodeLensPayload {
    pub(super) fn from_core(lens: LensView) -> Self {
        let (command, arguments) = lens.action.map_or_else(
            || (String::new(), Vec::new()),
            |action| command_and_arguments(action),
        );
        // VS Code sorts the lenses of a line by position, so each lens moves
        // `order` characters right to show in suggestion order.
        let mut range = lens.range;
        range.start.character = range
            .start
            .character
            .saturating_add(lens.order)
            .min(lens.range.end.character);
        range.end = range.start;
        Self {
            range: native_range_from_core(range),
            title: lens.title,
            command,
            arguments,
        }
    }
}

/// Maps a typed action to the VS Code command ID and positional arguments
/// that the extension's command handlers read.
fn command_and_arguments(action: Action) -> (String, Vec<String>) {
    match action {
        Action::UpdateDependency {
            name,
            selector,
            mode,
            version,
        } => {
            let mut arguments = vec![name, selector];
            arguments.extend(mode.map(|mode| mode.as_str().to_owned()));
            arguments.extend(version);
            (UPDATE_DEPENDENCY_COMMAND.to_owned(), arguments)
        }
        Action::ChooseBuild {
            selector,
            name,
            requirement,
            builds,
        } => {
            let mut arguments = vec![selector, name, requirement];
            arguments.extend(builds);
            (CHOOSE_BUILD_COMMAND.to_owned(), arguments)
        }
        Action::OpenFile { path } => (FILE_LINK_COMMAND.to_owned(), vec![path]),
    }
}

impl From<LensView> for NativeCodeLensPayload {
    fn from(value: LensView) -> Self {
        Self::from_core(value)
    }
}
