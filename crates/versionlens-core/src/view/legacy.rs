//! Test-only view of a `LensView` in the 0.4.4 `CodeLensPayload` shape: a VS Code
//! command ID and positional string arguments. Tests and the golden capture compare
//! against that shape.

use super::{Action, LensView};

impl LensView {
    pub(crate) fn command(&self) -> &'static str {
        match &self.action {
            None => "",
            Some(Action::UpdateDependency { .. }) => "versionlens.suggestion.onUpdateDependency",
            Some(Action::ChooseBuild { .. }) => "versionlens.suggestion.onChooseBuild",
            Some(Action::OpenFile { .. }) => "versionlens.suggestion.onFileLink",
        }
    }

    pub(crate) fn arguments(&self) -> Vec<String> {
        match &self.action {
            None => Vec::new(),
            Some(Action::UpdateDependency {
                name,
                selector,
                mode,
                version,
            }) => {
                let mut arguments = vec![name.clone(), selector.clone()];
                arguments.extend(mode.map(|mode| mode.as_str().to_owned()));
                arguments.extend(version.iter().cloned());
                arguments
            }
            Some(Action::ChooseBuild {
                selector,
                name,
                requirement,
                builds,
            }) => {
                let mut arguments = vec![selector.clone(), name.clone(), requirement.clone()];
                arguments.extend(builds.iter().cloned());
                arguments
            }
            Some(Action::OpenFile { path }) => vec![path.clone()],
        }
    }
}
