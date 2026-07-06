use versionlens_suggestions::Suggestion;
use versionlens_versions::{ProjectVersionBump, UpdateLevel, update_level};

use crate::project::is_project_version_dependency;
use versionlens_versions::ProjectVersionBump::{
    Major as ProjectMajor, Minor as ProjectMinor, Patch as ProjectPatch, Prerelease, Release,
};
use versionlens_versions::UpdateLevel::{
    Major as LevelMajor, Minor as LevelMinor, Patch as LevelPatch,
};

struct UpdateCommand {
    name: &'static str,
    level: UpdateLevel,
    project_bump: ProjectVersionBump,
}

struct ProjectUpdateCommand {
    name: &'static str,
    project_bump: ProjectVersionBump,
}

const UPDATE_COMMANDS: &[UpdateCommand] = &[
    UpdateCommand {
        name: "updateMajor",
        level: LevelMajor,
        project_bump: ProjectMajor,
    },
    UpdateCommand {
        name: "updateMinor",
        level: LevelMinor,
        project_bump: ProjectMinor,
    },
    UpdateCommand {
        name: "updatePatch",
        level: LevelPatch,
        project_bump: ProjectPatch,
    },
];

const PROJECT_UPDATE_COMMANDS: &[ProjectUpdateCommand] = &[
    ProjectUpdateCommand {
        name: "updateRelease",
        project_bump: Release,
    },
    ProjectUpdateCommand {
        name: "updatePrerelease",
        project_bump: Prerelease,
    },
];

pub(crate) fn filter_update_command(
    suggestions: &mut Vec<Suggestion>,
    command: Option<&str>,
    keep_selected_version: bool,
) {
    if project_update_command(command).is_some() {
        suggestions.retain(|suggestion| is_project_version_dependency(&suggestion.dependency));
        return;
    }

    if !keep_selected_version {
        select_update_choice_versions(suggestions, command);
    }

    let Some(level) = update_command_level(command) else {
        return;
    };

    suggestions.retain(|suggestion| {
        choice_version_for_command(suggestion, command).is_some()
            || suggestion
                .latest
                .as_ref()
                .and_then(|latest| update_level(latest, &suggestion.dependency.requirement))
                == Some(level)
    });
}

fn select_update_choice_versions(suggestions: &mut [Suggestion], command: Option<&str>) {
    let Some(command) = command else {
        return;
    };

    for suggestion in suggestions {
        if let Some(version) = choice_version_for_command(suggestion, Some(command)) {
            suggestion.latest = Some(version.to_owned());
        }
    }
}

fn choice_version_for_command<'a>(
    suggestion: &'a Suggestion,
    command: Option<&str>,
) -> Option<&'a str> {
    let command = command?;
    suggestion
        .choices
        .iter()
        .find(|choice| choice.command == command)
        .map(|choice| choice.version.as_str())
}

pub(crate) fn project_version_bump(
    command: Option<&str>,
    dependency_name: Option<&str>,
) -> Option<ProjectVersionBump> {
    if let Some(command) = project_update_command(command) {
        return Some(command.project_bump);
    }

    dependency_name?;
    update_command(command).map(|command| command.project_bump)
}

fn update_command_level(command: Option<&str>) -> Option<UpdateLevel> {
    update_command(command).map(|command| command.level)
}

fn update_command(command: Option<&str>) -> Option<&'static UpdateCommand> {
    let command = command?;
    UPDATE_COMMANDS.iter().find(|known| known.name == command)
}

fn project_update_command(command: Option<&str>) -> Option<&'static ProjectUpdateCommand> {
    let command = command?;
    PROJECT_UPDATE_COMMANDS
        .iter()
        .find(|known| known.name == command)
}
