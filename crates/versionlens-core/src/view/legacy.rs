//! Test-only flat view of a `LensView` action: its variant name and the positional
//! string arguments of the 0.4.4 `CodeLensPayload` shape. Tests and the golden capture
//! compare against these.

use super::{Action, LensView};

impl LensView {
    /// The `Action` variant name, or `""` for a display-only lens.
    pub(crate) fn action_kind(&self) -> &'static str {
        match &self.action {
            None => "",
            Some(Action::UpdateDependency { .. }) => "UpdateDependency",
            Some(Action::ChooseBuild { .. }) => "ChooseBuild",
            Some(Action::OpenFile { .. }) => "OpenFile",
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
