use versionlens_model::Dependency;
use versionlens_versions::ProjectVersionBump::{Major, Minor, Patch, Prerelease, Release};
use versionlens_versions::{
    ProjectVersionBump, is_prerelease_project_version, next_project_version,
};

use crate::view::UpdateMode;

mod predicate;

pub(crate) use predicate::is_project_version_dependency;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ProjectVersionCodeLensSuggestion {
    pub label: &'static str,
    pub command: UpdateMode,
    pub latest: String,
}

#[derive(Clone, Copy)]
struct ProjectVersionCodeLensEntry {
    bump: ProjectVersionBump,
    label: &'static str,
    command: UpdateMode,
}

const STABLE_PROJECT_VERSION_CODE_LENSES: &[ProjectVersionCodeLensEntry] = &[
    ProjectVersionCodeLensEntry {
        bump: Major,
        label: "major",
        command: UpdateMode::UpdateMajor,
    },
    ProjectVersionCodeLensEntry {
        bump: Minor,
        label: "minor",
        command: UpdateMode::UpdateMinor,
    },
    ProjectVersionCodeLensEntry {
        bump: Patch,
        label: "patch",
        command: UpdateMode::UpdatePatch,
    },
];

const PRERELEASE_PROJECT_VERSION_CODE_LENSES: &[ProjectVersionCodeLensEntry] = &[
    ProjectVersionCodeLensEntry {
        bump: Release,
        label: "release",
        command: UpdateMode::UpdateRelease,
    },
    ProjectVersionCodeLensEntry {
        bump: Prerelease,
        label: "prerelease",
        command: UpdateMode::UpdatePrerelease,
    },
];

pub(crate) fn project_version_latest(
    dependency: &Dependency,
    bump: Option<ProjectVersionBump>,
) -> Option<String> {
    is_project_version_dependency(dependency)
        .then(|| next_project_version(&dependency.requirement, bump))
        .flatten()
}

pub(crate) fn project_version_code_lens_suggestions(
    dependency: &Dependency,
) -> Vec<ProjectVersionCodeLensSuggestion> {
    if !is_project_version_dependency(dependency) {
        return vec![];
    }

    let entries = if is_prerelease_project_version(&dependency.requirement) {
        PRERELEASE_PROJECT_VERSION_CODE_LENSES
    } else {
        STABLE_PROJECT_VERSION_CODE_LENSES
    };

    entries
        .iter()
        .filter_map(|entry| project_version_code_lens_suggestion(dependency, entry))
        .collect()
}

fn project_version_code_lens_suggestion(
    dependency: &Dependency,
    entry: &ProjectVersionCodeLensEntry,
) -> Option<ProjectVersionCodeLensSuggestion> {
    Some(ProjectVersionCodeLensSuggestion {
        label: entry.label,
        command: entry.command,
        latest: next_project_version(&dependency.requirement, Some(entry.bump))?,
    })
}
