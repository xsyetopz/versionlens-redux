use versionlens_model::Ecosystem::Docker;
use versionlens_model::{Dependency, update_level};
use versionlens_update::Suggestion;

use crate::presentation::statuses::{
    StatusBuildAvailable, StatusCurrent, StatusDirectory, StatusDirectoryNotFound, StatusError,
    StatusFixed, StatusInvalid, StatusInvalidRange, StatusNoMatch, StatusNotSupported,
    StatusRateLimited, StatusSatisfies, StatusSatisfiesLatest, StatusUpdateAvailable,
    SuggestionStatus,
};
use crate::presentation::view::LensView;
use crate::presentation::{
    code_lens_title, project_version_code_lens_view, suggested_code_lens_view,
    update_choice_code_lens_view,
};
use crate::project::project_version_code_lens_suggestions;
use crate::session::CacheScope;
use crate::{SuggestionIndicators, VersionLensSession};

type CodeLensViews = Vec<LensView>;

impl VersionLensSession {
    pub(crate) fn code_lenses_for_dependency(
        &self,
        dependency: &Dependency,
        scope: &CacheScope,
    ) -> CodeLensViews {
        let project_version_suggestions = project_version_code_lens_suggestions(dependency);
        if !project_version_suggestions.is_empty() {
            return ordered_code_lenses(
                project_version_suggestions
                    .into_iter()
                    .map(|suggestion| {
                        project_version_code_lens_view(
                            dependency,
                            &suggestion,
                            &self.config.suggestion_indicators,
                        )
                    })
                    .collect(),
            );
        }

        let Some(suggestion) = self.cached_suggestion(dependency, scope) else {
            return vec![];
        };

        if dependency.is_runtime_constraint() {
            return vec![runtime_constraint_lens(
                dependency,
                &suggestion,
                &self.config.suggestion_indicators,
            )];
        }

        let has_vulnerabilities = self.target_update_has_cached_vulnerabilities(Some(&suggestion));
        let title = code_lens_title(
            dependency,
            Some(&suggestion),
            &self.config.suggestion_indicators,
            has_vulnerabilities,
        );

        let mut lenses = status_only_code_lens(
            dependency,
            Some(&suggestion),
            &self.config.suggestion_indicators,
        )
        .into_iter()
        .collect::<Vec<_>>();

        let build_lens = build_choice_code_lens_view(
            dependency,
            &suggestion,
            &self.config.suggestion_indicators,
            build_has_cached_vulnerabilities(self, dependency, &suggestion),
        );

        if suggestion.status == StatusCurrent
            && dependency.ecosystem != Docker
            && suggestion.choices.is_empty()
        {
            lenses.extend(build_lens);
            return ordered_code_lenses(lenses);
        }

        if shows_update_choice_lenses(suggestion.status) && !suggestion.choices.is_empty() {
            lenses.extend(suggestion.choices.iter().map(|choice| {
                update_choice_code_lens_view(
                    dependency,
                    choice,
                    &self.config.suggestion_indicators,
                    choice_has_cached_vulnerabilities(self, dependency, choice.version.as_str()),
                )
            }));
            lenses.extend(build_lens);
            return ordered_code_lenses(lenses);
        }

        if let Some(build_lens) = build_lens {
            lenses.push(build_lens);
            return ordered_code_lenses(lenses);
        }

        if !lenses.is_empty() && uses_status_command(suggestion.status) {
            return ordered_code_lenses(lenses);
        }

        lenses.push(suggested_code_lens_view(
            dependency,
            Some(&suggestion),
            title,
        ));
        ordered_code_lenses(lenses)
    }
}

fn runtime_constraint_lens(
    dependency: &Dependency,
    suggestion: &Suggestion,
    indicators: &SuggestionIndicators,
) -> LensView {
    let title = if suggestion.status == StatusSatisfiesLatest {
        format!(
            "checked {}; compatible {}",
            dependency.requirement,
            suggestion.latest.as_deref().unwrap_or("unknown")
        )
    } else {
        code_lens_title(dependency, Some(suggestion), indicators, false)
    };
    LensView::new(dependency.range, title, None)
}

fn ordered_code_lenses(mut lenses: CodeLensViews) -> CodeLensViews {
    for (order, lens) in lenses.iter_mut().enumerate() {
        lens.order = u32::try_from(order).unwrap_or(u32::MAX);
    }
    lenses
}

fn build_has_cached_vulnerabilities(
    session: &VersionLensSession,
    dependency: &Dependency,
    suggestion: &Suggestion,
) -> bool {
    session.has_cached_vulnerabilities(dependency)
        || session.target_update_has_cached_vulnerabilities(Some(suggestion))
}

fn choice_has_cached_vulnerabilities(
    session: &VersionLensSession,
    dependency: &Dependency,
    version: &str,
) -> bool {
    let suggestion = status_suggestion(dependency, Some(version), StatusUpdateAvailable);
    session.target_update_has_cached_vulnerabilities(Some(&suggestion))
}

fn shows_update_choice_lenses(status: SuggestionStatus) -> bool {
    matches!(
        status,
        StatusCurrent
            | StatusUpdateAvailable
            | StatusInvalid
            | StatusInvalidRange
            | StatusNoMatch
            | StatusFixed
            | StatusSatisfies
            | StatusSatisfiesLatest
    )
}

fn uses_status_command(status: SuggestionStatus) -> bool {
    !matches!(
        status,
        StatusUpdateAvailable | StatusBuildAvailable | StatusDirectory
    )
}

fn build_choice_code_lens_view(
    dependency: &Dependency,
    suggestion: &Suggestion,
    indicators: &SuggestionIndicators,
    has_vulnerabilities: bool,
) -> Option<LensView> {
    if suggestion.builds.is_empty() {
        return None;
    }

    let build_suggestion = Suggestion {
        builds: suggestion.builds.clone(),
        ..status_suggestion(
            dependency,
            suggestion.latest.as_deref(),
            StatusBuildAvailable,
        )
    };
    let title = code_lens_title(
        dependency,
        Some(&build_suggestion),
        indicators,
        has_vulnerabilities,
    );
    Some(suggested_code_lens_view(
        dependency,
        Some(&build_suggestion),
        title,
    ))
}

fn status_only_code_lens(
    dependency: &Dependency,
    suggestion: Option<&Suggestion>,
    indicators: &SuggestionIndicators,
) -> Option<LensView> {
    let status = status_only_suggestion(dependency, suggestion)?;
    let title = code_lens_title(dependency, Some(&status), indicators, false);
    Some(LensView::new(dependency.range, title, None))
}

fn status_only_suggestion(
    dependency: &Dependency,
    suggestion: Option<&Suggestion>,
) -> Option<Suggestion> {
    let suggestion = suggestion?;
    if suggestion.status == StatusNoMatch {
        return Some(status_suggestion(dependency, None, StatusNoMatch));
    }

    if suggestion.status == StatusNotSupported {
        return Some(status_suggestion(dependency, None, StatusNotSupported));
    }

    let latest = suggestion.latest.as_deref()?;
    if matches!(
        suggestion.status,
        StatusCurrent
            | StatusDirectoryNotFound
            | StatusInvalid
            | StatusInvalidRange
            | StatusSatisfiesLatest
            | StatusFixed
            | StatusError
            | StatusRateLimited
    ) {
        return Some(status_suggestion(
            dependency,
            Some(latest),
            suggestion.status,
        ));
    }

    if suggestion.status == StatusSatisfies {
        let version = bump_choice_version(suggestion).unwrap_or(latest);
        return Some(status_suggestion(
            dependency,
            Some(version),
            StatusSatisfies,
        ));
    }

    if suggestion.status != StatusUpdateAvailable
        || !is_fixed_version_requirement(&dependency.requirement, latest)
    {
        return None;
    }

    Some(status_suggestion(
        dependency,
        Some(dependency.requirement.as_str()),
        StatusFixed,
    ))
}

fn status_suggestion(
    dependency: &Dependency,
    latest: Option<&str>,
    status: SuggestionStatus,
) -> Suggestion {
    Suggestion {
        dependency: dependency.to_owned(),
        latest: latest.map(ToOwned::to_owned),
        resolved: None,
        status,
        builds: vec![],
        choices: vec![],
    }
}

fn bump_choice_version(suggestion: &Suggestion) -> Option<&str> {
    suggestion
        .choices
        .iter()
        .find(|choice| choice.label.as_str() == "bump")
        .map(|choice| choice.version.as_str())
}

fn is_fixed_version_requirement(requirement: &str, latest: &str) -> bool {
    let requirement = requirement.trim();
    !requirement.is_empty()
        && !requirement.contains([
            '^', '~', '>', '<', '=', '*', '|', ',', '[', ']', '(', ')', ' ',
        ])
        && update_level(latest, requirement).is_some()
}

#[cfg(test)]
mod tests {
    use crate::ProviderSettings;
    use crate::RegistryResponseInput;
    use versionlens_model::DocumentInput;

    use crate::{AnalyzeDocumentOutput, SessionConfig, SuggestionIndicators, VersionLensSession};
    use versionlens_model::Ecosystem::{Docker, Npm, Python};

    mod actions {
        use super::*;
        use crate::test_support::{file_uri, local_test_root};
        use std::fs::create_dir_all;
        use std::fs::remove_dir_all;
        use std::fs::write;

        #[test]
        fn package_version_gets_no_project_version_code_lenses() {
            let session = standard_session();
            let input = package_document("version-1.2.3.json");

            let output = session.analyze_document(input);

            assert_eq!(output.code_lenses.len(), 0);
        }

        #[test]
        fn project_version_code_lenses_offer_stable_bumps() {
            let session = standard_session();
            let input = jsr_version_document("1.2.3");

            let output = session.analyze_document(input);
            let titles = lens_titles(&output);
            let commands = crate::test_support::code_lens_commands(&output);

            assert_eq!(titles, ["U major 2.0.0", "U minor 1.3.0", "U patch 1.2.4"]);
            assert_eq!(commands, ["updateMajor", "updateMinor", "updatePatch"]);
        }

        #[test]
        fn project_version_code_lenses_offer_prerelease_bumps() {
            let session = standard_session();
            let input = jsr_version_document("1.2.3-beta.4");

            let output = session.analyze_document(input);
            let titles = lens_titles(&output);
            let commands = crate::test_support::code_lens_commands(&output);

            assert_eq!(titles, ["U release 1.2.3", "U prerelease 1.2.3-beta.5"]);
            assert_eq!(commands, ["updateRelease", "updatePrerelease"]);
        }

        fn jsr_version_document(version: &str) -> DocumentInput {
            DocumentInput::new(
                "file:///jsr.json".to_owned(),
                "json".to_owned(),
                format!(r#"{{"name":"@scope/pkg","version":"{version}"}}"#),
                None,
            )
        }

        #[tokio::test]
        async fn build_code_lens_chooses_available_build_versions() {
            let output = build_code_lens_output(
                "left-pad-1.0.0-build.1.json",
                "1.0.0+build.2",
                &[
                    "1.0.0",
                    "1.0.0+build.1",
                    "1.0.0+build.2",
                    "1.0.0+build.3",
                    "1.1.0",
                ],
            )
            .await;
            let titles = lens_titles(&output);
            let commands = lens_commands(&output);

            assert_eq!(titles, ["L latest 1.0.0+build.1", "B change build"]);
            assert_eq!(commands, ["", "ChooseBuild"]);
            assert_eq!(
                &output.code_lenses[1].arguments()[1..],
                [
                    "left-pad",
                    "1.0.0+build.1",
                    "1.0.0",
                    "1.0.0+build.1",
                    "1.0.0+build.2",
                    "1.0.0+build.3"
                ]
            );
        }

        #[tokio::test]
        async fn build_code_lens_keeps_latest_status_when_current_has_build_versions() {
            let output = build_code_lens_output(
                "left-pad-3.0.0.json",
                "3.0.0",
                &["1.0.0", "2.0.0", "2.1.0", "3.0.0", "3.0.0+b1", "3.0.0+b2"],
            )
            .await;
            let titles = lens_titles(&output);
            let commands = lens_commands(&output);

            assert_eq!(titles, ["L latest 3.0.0", "B change build"]);
            assert_eq!(commands, ["", "ChooseBuild"]);
            assert_eq!(
                &output.code_lenses[1].arguments()[1..],
                ["left-pad", "3.0.0", "3.0.0", "3.0.0+b1", "3.0.0+b2"]
            );
        }

        #[tokio::test]
        async fn build_code_lens_keeps_latest_status_when_current_build_differs_from_latest_build()
        {
            let output = build_code_lens_output(
                "left-pad-3.0.0-b1.json",
                "3.0.0+b2",
                &["1.0.0", "2.0.0", "2.1.0", "3.0.0", "3.0.0+b1", "3.0.0+b2"],
            )
            .await;
            let titles = lens_titles(&output);
            let commands = lens_commands(&output);

            assert_eq!(titles, ["L latest 3.0.0+b1", "B change build"]);
            assert_eq!(commands, ["", "ChooseBuild"]);
            assert_eq!(
                &output.code_lenses[1].arguments()[1..],
                ["left-pad", "3.0.0+b1", "3.0.0", "3.0.0+b1", "3.0.0+b2"]
            );
        }

        #[tokio::test]
        async fn build_code_lens_uses_latest_build_when_variant_list_is_missing() {
            let session = standard_session();
            let input = package_document("left-pad-1.0.0-build.1.json");

            let output = crate::test_support::analyze_with_responses(
                &session,
                &input,
                &[crate::test_support::npm_latest_response(
                    "left-pad",
                    "1.0.0+build.2",
                )],
            )
            .await;

            assert_eq!(output.code_lenses[0].title, "B change build");
            assert_eq!(output.code_lenses[0].action_kind(), "ChooseBuild");
            assert_eq!(
                &output.code_lenses[0].arguments()[1..],
                ["left-pad", "1.0.0+build.1", "1.0.0+build.2"]
            );
        }

        #[tokio::test]
        async fn directory_code_lens_opens_local_dependency_path() {
            let session = standard_session();
            let root = local_test_root("directory-codelens");
            let app = root.join("app");
            let local = root.join("local");
            create_dir_all(&app).unwrap();
            create_dir_all(&local).unwrap();
            let input = DocumentInput::new(
                file_uri(&app.join("package.json")),
                "json".to_owned(),
                package_file_fixture("local-file-dependency.json"),
                None,
            );

            session.resolve_document(input.clone()).await;
            let output = session.analyze_document(input);

            let local_path = local.to_string_lossy();
            assert_eq!(output.code_lenses[0].title, "D file://../local");
            assert_eq!(output.code_lenses[0].action_kind(), "OpenFile");
            assert_eq!(output.code_lenses[0].arguments(), [local_path.as_ref()]);
            remove_dir_all(root).unwrap();
        }

        #[tokio::test]
        async fn npm_link_code_lens_opens_package_json_target_path() {
            let session = standard_session();
            let root = local_test_root("npm-link-codelens");
            let app = root.join("app");
            let local = root.join("local");
            create_dir_all(&app).unwrap();
            create_dir_all(&local).unwrap();
            write(
                local.join("package.json"),
                package_file_fixture("empty-package.json"),
            )
            .unwrap();
            let input = DocumentInput::new(
                file_uri(&app.join("package.json")),
                "json".to_owned(),
                package_file_fixture("local-link-dependency.json"),
                None,
            );

            session.resolve_document(input.clone()).await;
            let output = session.analyze_document(input);

            let target_path = local.join("package.json");
            let target_path = target_path.to_string_lossy();
            assert_eq!(
                output.code_lenses[0].title,
                "D file://../local/package.json"
            );
            assert_eq!(output.code_lenses[0].action_kind(), "OpenFile");
            assert_eq!(output.code_lenses[0].arguments(), [target_path.as_ref()]);
            remove_dir_all(root).unwrap();
        }

        #[tokio::test]
        async fn missing_directory_code_lens_is_disabled_not_found_status() {
            let session = standard_session();
            let input = DocumentInput::new(
                "file:///repo/app/package.json".to_owned(),
                "json".to_owned(),
                package_file_fixture("local-file-dependency.json"),
                None,
            );

            session.resolve_document(input.clone()).await;
            let output = session.analyze_document(input);

            assert_eq!(output.code_lenses[0].title, "E not found ../local");
            assert_eq!(output.code_lenses[0].action_kind(), "");
            assert_eq!(output.code_lenses[0].arguments().len(), 0);
        }
    }
    mod docker {
        use super::*;

        #[tokio::test]
        async fn docker_argument_image_name_uses_not_supported_status() {
            let lenses =
                docker_code_lenses(package_file_fixture("Dockerfile-arg-image").as_str()).await;

            assert_eq!(lenses, [("N not supported".to_owned(), String::new())]);
        }

        #[tokio::test]
        async fn docker_argument_image_version_uses_not_supported_status() {
            let lenses =
                docker_code_lenses(package_file_fixture("Dockerfile-arg-version").as_str()).await;

            assert_eq!(lenses, [("N not supported".to_owned(), String::new())]);
        }

        const NODE_IMAGE_RESPONSE: &str = r#"{"results":[{"name":"latest","tag_status":"active","digest":"sha256-23"},{"name":"current-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"current","tag_status":"active","digest":"sha256-23"},{"name":"bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23.11.0-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23.11.0","tag_status":"active","digest":"sha256-23"},{"name":"23.11-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23.11","tag_status":"active","digest":"sha256-23"},{"name":"23-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23","tag_status":"active","digest":"sha256-23"},{"name":"22.4.3","tag_status":"active","digest":"sha256-22"},{"name":"22.4","tag_status":"active","digest":"sha256-22"},{"name":"22-bookworm","tag_status":"active","digest":"sha256-22"},{"name":"22","tag_status":"active","digest":"sha256-22"},{"name":"21.0.0","tag_status":"active","digest":"sha256-21"},{"name":"21.0","tag_status":"active","digest":"sha256-21"}]}"#;

        async fn docker_output(fixture: &str) -> AnalyzeDocumentOutput {
            let session = standard_session();
            let input = DocumentInput::new(
                "file:///Dockerfile".to_owned(),
                "dockerfile".to_owned(),
                package_file_fixture(fixture),
                None,
            );
            crate::test_support::analyze_with_responses(
                &session,
                &input,
                &[RegistryResponseInput::new(
                    "node".to_owned(),
                    Docker,
                    NODE_IMAGE_RESPONSE.to_owned(),
                )],
            )
            .await
        }

        async fn docker_code_lenses(text: &str) -> Vec<(String, String)> {
            let session = standard_session();
            let input = DocumentInput::new(
                "file:///Dockerfile".to_owned(),
                "dockerfile".to_owned(),
                text.to_owned(),
                None,
            );

            session
                .resolve_document_with_responses(input.clone(), &[])
                .await;
            session
                .analyze_document(input)
                .code_lenses
                .into_iter()
                .map(|lens| {
                    let command = lens.action_kind().to_owned();
                    (lens.title, command)
                })
                .collect()
        }

        #[tokio::test]
        async fn docker_code_lenses_offer_same_suffix_update_choices() {
            let session = standard_session();
            let input = DocumentInput::new(
                "file:///Dockerfile".to_owned(),
                "dockerfile".to_owned(),
                package_file_fixture("Dockerfile-node-20-bookworm"),
                None,
            );

            let output = crate::test_support::analyze_with_responses(
        &session,
        &input,
        &[RegistryResponseInput::new("node".to_owned(), Docker, r#"{"results":[{"name":"20.19.1-bookworm","tag_status":"active","digest":"sha256-20-bookworm"},{"name":"21.0.0-alpine","tag_status":"active","digest":"sha256-21-alpine"},{"name":"23.11.0-bookworm","tag_status":"active","digest":"sha256-23-bookworm"}]}"#
                .to_owned())],
    ).await;
            let arguments = crate::test_support::update_code_lens_arguments(&output);

            assert_eq!(arguments, [vec!["update", "23.11.0-bookworm"]]);
        }

        #[tokio::test]
        async fn docker_code_lenses_map_latest_update_choice_to_matching_tag_shape() {
            let output = docker_output("Dockerfile-node-22-bookworm").await;
            let arguments = crate::test_support::update_code_lens_arguments(&output);
            assert_eq!(arguments, [vec!["update", "23-bookworm"]]);
            assert_eq!(
                output
                    .code_lenses
                    .iter()
                    .find(|lens| lens.action_kind() == "ChooseBuild")
                    .map(|lens| lens.arguments().into_iter().skip(1).collect::<Vec<_>>())
                    .expect("choose build lens"),
                ["node", "22-bookworm", "22", "22-bookworm", "22.4", "22.4.3"]
            );
        }

        #[tokio::test]
        async fn docker_code_lenses_offer_update_choices_for_missing_numeric_tag() {
            let output = docker_output("Dockerfile-node-21").await;
            let titles = lens_titles(&output);
            let arguments = crate::test_support::code_lens_arguments(&output);

            assert_eq!(titles, ["N no match", "U latest 23", "U major 22"]);
            assert_eq!(arguments, [vec!["update", "23"], vec!["updateMajor", "22"]]);
        }

        #[tokio::test]
        async fn docker_code_lenses_offer_latest_for_untagged_non_version_latest() {
            let session = standard_session();
            let input = DocumentInput::new(
                "file:///Dockerfile".to_owned(),
                "dockerfile".to_owned(),
                package_file_fixture("Dockerfile-mssql-server"),
                None,
            );

            let output = crate::test_support::analyze_with_responses(
        &session,
        &input,
        &[RegistryResponseInput::new("mssql/server".to_owned(), Docker, r#"{"results":[{"name":"2022-RTM-CU2-ubuntu-20.04","tag_status":"active","digest":"sha256-a"},{"name":"2022-RTM-GDR1-ubuntu-20.04","tag_status":"active","digest":"sha256-b"},{"name":"2022-RTM-ubuntu-20.04","tag_status":"active","digest":"sha256-c"},{"name":"2022-latest","tag_status":"active","digest":"sha256-latest"},{"name":"2022-preview-ubuntu-22.04","tag_status":"active","digest":"sha256-d"},{"name":"latest","tag_status":"active","digest":"sha256-latest"},{"name":"latest-ubuntu","tag_status":"active","digest":"sha256-e"}]}"#
                .to_owned())],
    ).await;
            let titles = lens_titles(&output);
            let arguments = crate::test_support::code_lens_arguments(&output);

            assert_eq!(titles, ["N no match", "U latest latest"]);
            assert_eq!(arguments, [vec!["update", "latest"]]);
        }
    }
    mod npm {
        use super::*;

        #[tokio::test]
        async fn code_lenses_label_latest_dist_tag_prerelease_for_missing_fixed_versions() {
            let session = standard_session();
            let input = package_document("left-pad-4.0.0.json");

            let output = crate::test_support::analyze_with_responses(
                &session,
                &input,
                &[RegistryResponseInput::new(
                    "left-pad".to_owned(),
                    Npm,
                    r#"{
              "dist-tags": { "latest": "4.0.0-next" },
              "versions": {
                "0.0.5": {},
                "0.0.6": {},
                "1.1.0-alpha.1": {},
                "4.0.0-next": {}
              }
            }"#
                    .to_owned(),
                )],
            )
            .await;
            let titles = lens_titles(&output);
            let arguments = crate::test_support::code_lens_arguments(&output);

            assert_eq!(titles, ["N no match",]);
            assert_eq!(arguments.len(), 0);
        }

        #[tokio::test]
        async fn npm_invalid_tag_name_error_offers_latest_dist_tag_update() {
            let session = standard_session();
            let input = package_document("left-pad-bad-tag.json");

            let resolved = session
                .resolve_document_with_responses(
                    input.clone(),
                    &[RegistryResponseInput::new(
                        "left-pad".to_owned(),
                        Npm,
                        r#"{"status":"EINVALIDTAGNAME"}"#.to_owned(),
                    )],
                )
                .await;
            let output = session.analyze_document(input);
            let titles = lens_titles(&output);
            let commands = lens_commands(&output);
            let arguments = crate::test_support::code_lens_arguments(&output);

            assert_eq!(resolved.suggestions[0].status, "invalid");
            assert_eq!(titles, ["E invalid version", "U latest latest"]);
            assert_eq!(commands, ["", "UpdateDependency"]);
            assert_eq!(arguments, [vec!["update", "latest"]]);
        }

        #[tokio::test]
        async fn npm_unsupported_protocol_error_uses_not_supported_status() {
            let session = standard_session();
            let input = package_document("left-pad-1.0.0.json");

            let resolved = session
                .resolve_document_with_responses(
                    input.clone(),
                    &[RegistryResponseInput::new(
                        "left-pad".to_owned(),
                        Npm,
                        r#"{"status":"EUNSUPPORTEDPROTOCOL"}"#.to_owned(),
                    )],
                )
                .await;
            let output = session.analyze_document(input);
            let titles = lens_titles(&output);
            let commands = lens_commands(&output);

            assert_eq!(resolved.suggestions[0].status, "notSupported");
            assert_eq!(titles, ["N not supported"]);
            assert_eq!(commands, [""]);
        }
    }
    mod python {
        use super::*;

        fn assert_python_code_lenses(output: &AnalyzeDocumentOutput, expected: &[(&str, &str)]) {
            assert_eq!(
                output
                    .code_lenses
                    .iter()
                    .map(|lens| (lens.title.as_str(), lens.action_kind()))
                    .collect::<Vec<_>>(),
                expected
            );
        }

        #[tokio::test]
        async fn pyproject_ranges_that_admit_latest_offer_updates_from_canonical_pypi_releases() {
            let session = standard_session();
            let input = DocumentInput::new(
                "file:///pyproject.toml".to_owned(),
                "toml".to_owned(),
                r#"
[project]
dependencies = ["httpx>=0.27,<1"]

[project.optional-dependencies]
test = ["httpcore>=0.27,<1"]
"#
                .to_owned(),
                None,
            );
            let body = r#"{
      "info": { "version": "0.28.1" },
      "releases": {
        "0.27.0": [],
        "0.28.1": [{ "yanked": false }]
      }
    }"#
            .to_owned();

            let output = session
                .resolve_document_with_responses(
                    input.clone(),
                    &[
                        RegistryResponseInput::new("httpx".to_owned(), Python, body.clone()),
                        RegistryResponseInput {
                            package: "httpcore".to_owned(),
                            ecosystem: Python,
                            body,
                        },
                    ],
                )
                .await;

            assert_eq!(
                output
                    .suggestions
                    .iter()
                    .map(|suggestion| suggestion.status.as_str())
                    .collect::<Vec<_>>(),
                ["satisfiesLatest", "satisfiesLatest"]
            );

            let lenses = session.analyze_document(input);
            assert_python_code_lenses(
                &lenses,
                &[
                    ("S satisfies latest 0.28.1", ""),
                    ("S satisfies latest 0.28.1", ""),
                ],
            );
        }

        #[tokio::test]
        async fn requirements_ranges_offer_updates_from_canonical_pypi_releases() {
            let session = standard_session();
            let input = DocumentInput::new(
                "file:///requirements.txt".to_owned(),
                "pip-requirements".to_owned(),
                "httpx>=0.27,<1\n".to_owned(),
                None,
            );

            let output = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput::new(
            "httpx".to_owned(),
            Python,
            r#"{"info":{"version":"0.28.1"},"releases":{"0.27.0":[],"0.28.1":[{"yanked":false}]}}"#
                .to_owned(),
        )],
    ).await;

            assert_eq!(output.suggestions[0].status, "satisfiesLatest");
            let lenses = session.analyze_document(input);
            assert_python_code_lenses(&lenses, &[("S satisfies latest 0.28.1", "")]);
        }
    }
    mod ranges {
        use super::*;

        #[tokio::test]
        async fn code_lenses_offer_minor_update_choices_for_tilde_ranges() {
            let session = standard_session();
            let input = package_document("left-pad-tilde-1.1.json");

            let output = crate::test_support::analyze_with_responses(
                &session,
                &input,
                &[RegistryResponseInput::new(
                    "left-pad".to_owned(),
                    Npm,
                    r#"{
              "dist-tags": { "latest": "2.2.2" },
              "versions": {
                "1.1.0": {},
                "1.1.1": {},
                "1.1.2": {},
                "1.2.0": {},
                "1.2.2": {},
                "2.0.0": {},
                "2.2.2": {}
              }
            }"#
                    .to_owned(),
                )],
            )
            .await;
            let orders = output
                .code_lenses
                .iter()
                .map(|lens| lens.order)
                .collect::<Vec<_>>();
            assert_eq!(orders, [0, 1, 2, 3]);
            assert!(
                output
                    .code_lenses
                    .iter()
                    .all(|lens| lens.range == output.code_lenses[0].range),
                "lenses of one dependency share its range: {:?}",
                output.code_lenses
            );
            let titles = lens_titles(&output);
            let arguments = crate::test_support::all_code_lens_arguments(&output);

            assert_eq!(
                titles,
                [
                    "M satisfies 1.1.2",
                    "U bump 1.1.2",
                    "U minor 1.2.2",
                    "U latest 2.2.2"
                ]
            );
            assert_eq!(
                arguments,
                [
                    Vec::<&str>::new(),
                    vec!["update", "1.1.2"],
                    vec!["updateMinor", "1.2.2"],
                    vec!["update", "2.2.2"]
                ]
            );
        }
    }
    mod vulnerabilities {
        use super::*;

        async fn vulnerable_output(
            session: VersionLensSession,
            fixture: &str,
            response: RegistryResponseInput,
        ) -> AnalyzeDocumentOutput {
            let input = package_document(fixture);
            session
                .resolve_document_with_responses(input.clone(), &[response])
                .await;
            session.analyze_document(input)
        }

        #[tokio::test]
        async fn code_lens_title_marks_vulnerable_update_targets() {
            let output = vulnerable_output(
                standard_session(),
                "left-pad-1.0.0.json",
                npm_vulnerability_response(
                    Some("1.1.0"),
                    &[],
                    "OSV-1",
                    "target issue",
                    Some(("1.1.0", "2.0.0")),
                ),
            )
            .await;

            assert_eq!(output.code_lenses[1].title, "V latest 1.1.0");
        }

        #[tokio::test]
        async fn code_lens_title_does_not_mark_update_that_fixes_current_vulnerability() {
            let output = vulnerable_output(
                standard_session(),
                "left-pad-1.0.0.json",
                npm_vulnerability_response(
                    Some("1.1.0"),
                    &[],
                    "OSV-1",
                    "current issue",
                    Some(("0", "1.1.0")),
                ),
            )
            .await;

            assert_eq!(output.code_lenses[1].title, "U latest 1.1.0");
        }

        #[tokio::test]
        async fn vulnerable_update_indicator_falls_back_to_warning_when_configured_indicator_is_empty()
         {
            let output = vulnerable_output(
                session_with_empty_vulnerable_indicator(),
                "left-pad-1.1.1.json",
                npm_vulnerability_response(
                    None,
                    &["1.1.1", "1.1.2", "1.2.2", "2.2.2"],
                    "OSV-MINOR",
                    "minor target issue",
                    Some(("1.2.2", "1.2.3")),
                ),
            )
            .await;
            let titles = lens_titles(&output);

            assert!(titles.contains(&"⚠️ minor 1.2.2"));
            assert!(!titles.contains(&"U minor 1.2.2"));
        }

        #[tokio::test]
        async fn vulnerable_build_code_lens_uses_vulnerable_update_indicator_fallback() {
            let output = vulnerable_output(
                session_with_empty_vulnerable_indicator(),
                "left-pad-1.0.0-b1.json",
                npm_vulnerability_response(
                    Some("1.0.0+b2"),
                    &["1.0.0+b1", "1.0.0+b2"],
                    "OSV-BUILD",
                    "build target issue",
                    None,
                ),
            )
            .await;
            let titles = lens_titles(&output);

            assert!(titles.contains(&"⚠️ change build"));
            assert!(!titles.contains(&"B change build"));
        }

        #[tokio::test]
        async fn update_choice_code_lens_marks_vulnerable_non_latest_targets() {
            let output = vulnerable_output(
                standard_session(),
                "left-pad-1.1.1.json",
                npm_vulnerability_response(
                    None,
                    &["1.1.1", "1.1.2", "1.2.2", "2.2.2"],
                    "OSV-MINOR",
                    "minor target issue",
                    Some(("1.2.2", "1.2.3")),
                ),
            )
            .await;
            let titles = lens_titles(&output);

            assert_eq!(
                titles,
                [
                    "M fixed 1.1.1",
                    "U patch 1.1.2",
                    "V minor 1.2.2",
                    "U latest 2.2.2"
                ]
            );
        }
    }

    #[tokio::test]
    async fn code_lens_title_uses_configured_indicators() {
        let session = standard_session();
        let output = analyze_npm_fixture_with_response(
            &session,
            "left-pad-1.0.0.json",
            r#"{"dist-tags":{"latest":"1.1.0"}}"#,
        )
        .await;

        let titles = lens_titles(&output);
        let commands = lens_commands(&output);

        assert_eq!(titles, ["M fixed 1.0.0", "U latest 1.1.0"]);
        assert_eq!(commands, ["", "UpdateDependency"]);
    }

    #[tokio::test]
    async fn direct_blank_indicators_use_standard_glyphs_for_status_and_update_lenses() {
        let session = crate::version_lens_session(SessionConfig {
            update_severity: Some(crate::Severity::Information),
            cache_ttl_ms: 300_000,
            enabled_providers: vec![],
            providers: ProviderSettings::default(),
            suggestion_indicators: SuggestionIndicators {
                latest: String::new(),
                satisfies_latest: " \t\n\u{2003} ".to_owned(),
                directory: String::new(),
                error: String::new(),
                no_match: String::new(),
                matched: String::new(),
                updateable: String::new(),
                updateable_vulnerable: String::new(),
                build: String::new(),
            },
            show_vulnerabilities: true,
            show_suggestion_stats: false,
            show_prereleases: false,
            http: versionlens_fetch::standard_http_config(),
        });

        assert_eq!(
            session.config.suggestion_indicators,
            crate::standard_suggestion_indicators()
        );
        let output = analyze_npm_fixture_with_response(
            &session,
            "left-pad-1.0.0.json",
            r#"{"dist-tags":{"latest":"1.1.0"}}"#,
        )
        .await;

        assert_eq!(
            output
                .code_lenses
                .iter()
                .map(|lens| lens.title.as_str())
                .collect::<Vec<_>>(),
            ["🟡 fixed 1.0.0", "↑  latest 1.1.0"]
        );
        assert_eq!(output.code_lenses[1].action_kind(), "UpdateDependency");
    }

    #[tokio::test]
    async fn code_lenses_offer_release_update_choices_for_fixed_versions() {
        let session = standard_session();
        let input = package_document("left-pad-1.0.0.json");

        let output = crate::test_support::analyze_with_responses(
            &session,
            &input,
            &[npm_versions_response(
                "2.1.0",
                &["1.0.0", "1.0.1", "1.1.0", "2.0.0", "2.1.0"],
            )],
        )
        .await;

        let titles = lens_titles(&output);
        let arguments = crate::test_support::code_lens_arguments(&output);

        assert_eq!(
            titles,
            [
                "M fixed 1.0.0",
                "U patch 1.0.1",
                "U minor 1.1.0",
                "U latest 2.1.0"
            ]
        );
        assert_eq!(
            arguments,
            [
                vec!["updatePatch", "1.0.1"],
                vec!["updateMinor", "1.1.0"],
                vec!["update", "2.1.0"]
            ]
        );
    }

    #[tokio::test]
    async fn code_lenses_carry_their_order_and_the_dependency_range() {
        let session = session_with_indicators(test_indicators(), false);
        let input = package_document("left-pad-1.0.0.json");

        let output = crate::test_support::analyze_with_responses(
            &session,
            &input,
            &[npm_versions_response(
                "2.1.0",
                &["1.0.0", "1.0.1", "1.1.0", "2.1.0"],
            )],
        )
        .await;
        let dependency_range = output.dependencies[0].range;
        let orders = output
            .code_lenses
            .iter()
            .map(|lens| lens.order)
            .collect::<Vec<_>>();
        let expected_orders = (0..orders.len())
            .map(|order| u32::try_from(order).expect("code lens order fits u32"))
            .collect::<Vec<_>>();

        assert!(orders.len() > 1, "{:?}", output.code_lenses);
        assert_eq!(orders, expected_orders);
        assert!(
            output
                .code_lenses
                .iter()
                .all(|lens| lens.range == dependency_range)
        );
    }

    #[tokio::test]
    async fn multiline_package_json_code_lenses_stay_on_dependency_lines() {
        let session = session_with_indicators(test_indicators(), false);
        let input = package_document("dev-dependencies.json");

        let output = crate::test_support::analyze_with_responses(
            &session,
            &input,
            &[
                npm_response("@biomejs/biome", "2.5.2"),
                npm_response("@types/bun", "1.3.14"),
                npm_response("@types/node", "26.0.1"),
                npm_response("@types/vscode", "1.125.0"),
                npm_response("@vscode/vsce", "3.9.2"),
                npm_response("typescript", "6.0.3"),
            ],
        )
        .await;
        let lenses = output
            .code_lenses
            .iter()
            .map(|lens| {
                (
                    lens.range.start.line,
                    lens.title.as_str(),
                    lens.action_kind(),
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            lenses,
            [
                (2, "L latest 2.5.2", ""),
                (3, "L latest 1.3.14", ""),
                (4, "L latest 26.0.1", ""),
                (5, "S satisfies latest 1.125.0", ""),
                (5, "U latest 1.125.0", "UpdateDependency"),
                (6, "L latest 3.9.2", ""),
                (7, "L latest 6.0.3", "")
            ]
        );
    }
    #[tokio::test]
    async fn code_lenses_offer_bump_update_choices_for_ranges() {
        let output = build_code_lens_output(
            "left-pad-4.1.0-range.json",
            "5.4.5",
            &[
                "2.1.2", "3.0.0", "3.1.0", "4.0.0", "4.0.1", "4.1.10", "5.1.1", "5.2.0", "5.3.3",
                "5.4.5",
            ],
        )
        .await;
        let titles = lens_titles(&output);
        let arguments = crate::test_support::all_code_lens_arguments(&output);

        assert_eq!(
            titles,
            ["M satisfies 4.1.10", "U bump 4.1.10", "U latest 5.4.5",]
        );
        assert_eq!(
            arguments,
            [
                Vec::<&str>::new(),
                vec!["update", "4.1.10"],
                vec!["update", "5.4.5"]
            ]
        );
    }

    #[tokio::test]
    async fn code_lenses_keep_latest_update_choice_for_invalid_ranges() {
        let output = build_code_lens_output(
            "left-pad-invalid-range.json",
            "5.0.0",
            &["1.0.0", "2.0.0", "5.0.0"],
        )
        .await;
        let titles = lens_titles(&output);
        let arguments = crate::test_support::all_code_lens_arguments(&output);

        assert_eq!(titles, ["E invalid version range", "U latest 5.0.0"]);
        assert_eq!(arguments, [Vec::<&str>::new(), vec!["update", "5.0.0"]]);
    }

    #[tokio::test]
    async fn code_lenses_omit_latest_update_choice_for_ranges_satisfying_latest() {
        let output = build_code_lens_output(
            "left-pad-gte-2.json",
            "3.0.0",
            &["1.0.0", "2.0.0", "2.1.0", "3.0.0"],
        )
        .await;
        assert_eq!(lens_titles(&output), ["S satisfies latest 3.0.0",]);
        assert_eq!(
            crate::test_support::all_code_lens_arguments(&output),
            [Vec::<&str>::new()]
        );
    }

    #[tokio::test]
    async fn code_lenses_keep_satisfies_status_for_ranges_with_in_range_updates() {
        let output = build_code_lens_output(
            "left-pad-gte-2-lt-3.json",
            "3.0.0",
            &["1.0.0", "2.0.0", "2.1.0", "3.0.0"],
        )
        .await;
        let titles = lens_titles(&output);
        let arguments = crate::test_support::all_code_lens_arguments(&output);

        assert_eq!(
            titles,
            ["M satisfies 2.1.0", "U bump 2.1.0", "U latest 3.0.0"]
        );
        assert_eq!(
            arguments,
            [
                Vec::<&str>::new(),
                vec!["update", "2.1.0"],
                vec!["update", "3.0.0"]
            ]
        );
    }

    #[tokio::test]
    async fn github_current_release_omits_noop_latest_and_older_choices() {
        let session = standard_session();
        let input = DocumentInput::new(
            "file:///work/.github/workflows/ci.yml".to_owned(),
            "yaml".to_owned(),
            "steps:\n  - uses: actions/checkout@v7.0.1\n".to_owned(),
            None,
        );

        let responses = [RegistryResponseInput::new(
            "actions/checkout".to_owned(),
            versionlens_model::Ecosystem::GitHub,
            r#"[{"name":"v7.0.1"},{"name":"v7.0.0"},{"name":"v6.0.0"}]"#.to_owned(),
        )];
        let resolved = session
            .resolve_document_with_responses(input.clone(), &responses)
            .await;
        let output = session.analyze_document(input);

        assert_eq!(
            (
                resolved.suggestions[0].status.as_str(),
                resolved.suggestions[0].latest.as_deref()
            ),
            ("current", Some("7.0.1"))
        );

        assert_eq!(lens_titles(&output), ["L latest v7.0.1",]);
        assert_eq!(
            crate::test_support::all_code_lens_arguments(&output),
            [Vec::<&str>::new()]
        );
    }

    #[tokio::test]
    async fn code_lenses_offer_prerelease_update_choices_by_tag() {
        let session = standard_session();
        let input = package_document("left-pad-prerelease-range.json");

        let output = crate::test_support::analyze_with_responses(
            &session,
            &input,
            &[RegistryResponseInput::new(
                "left-pad".to_owned(),
                Npm,
                r#"{
              "versions": {
                "1.0.0-alpha": {},
                "1.0.1-alpha": {},
                "1.2.0-alpha": {},
                "1.2.0-dev": {},
                "1.2.0-beta": {}
              }
            }"#
                .to_owned(),
            )],
        )
        .await;
        let titles = lens_titles(&output);

        assert_eq!(
            titles,
            [
                "U dev 1.2.0-dev",
                "U beta 1.2.0-beta",
                "U alpha 1.2.0-alpha"
            ]
        );
    }
    #[tokio::test]
    async fn code_lens_title_uses_satisfies_latest_indicator() {
        let session = standard_session();
        let input = package_document("left-pad-caret-1.0.0.json");

        let output = crate::test_support::analyze_with_responses(
            &session,
            &input,
            &[RegistryResponseInput::new(
                "left-pad".to_owned(),
                Npm,
                r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
            )],
        )
        .await;

        assert_eq!(output.code_lenses[0].title, "S satisfies latest 1.1.0");
    }

    #[tokio::test]
    async fn code_lens_title_uses_latest_indicator_for_current_dependencies() {
        let session = standard_session();
        let input = package_document("typescript-latest.json");

        let output = crate::test_support::analyze_with_responses(
            &session,
            &input,
            &[RegistryResponseInput::new(
                "typescript".to_owned(),
                Npm,
                r#"{"dist-tags":{"latest":"6.0.3"}}"#.to_owned(),
            )],
        )
        .await;

        assert_eq!(output.code_lenses[0].title, "L latest 6.0.3");
    }

    #[tokio::test]
    async fn code_lens_title_shows_fixed_git_dependencies() {
        let session = standard_session();
        let input = DocumentInput::new(
            "file:///Cargo.toml".to_owned(),
            "toml".to_owned(),
            package_file_fixture("Cargo-git-dependency.toml"),
            None,
        );

        session.resolve_document(input.clone()).await;
        let output = session.analyze_document(input);

        assert_eq!(output.code_lenses[0].title, "M fixed git repository");
    }

    #[test]
    fn missing_suggestion_code_lens_is_omitted_like_upstream() {
        let session = standard_session();
        let input = package_document("left-pad-1.0.0.json");

        let output = session.analyze_document(input);

        assert_eq!(output.code_lenses.len(), 0);
    }

    #[tokio::test]
    async fn code_lens_title_preserves_configured_indicator_spacing_like_non_windows_upstream() {
        let mut indicators = test_indicators();
        indicators.updateable = "U ".to_owned();
        let session = session_with_indicators(indicators, true);
        let output = analyze_npm_fixture_with_response(
            &session,
            "left-pad-1.0.0.json",
            r#"{"dist-tags":{"latest":"1.1.0"}}"#,
        )
        .await;

        assert_eq!(output.code_lenses[1].title, "U  latest 1.1.0");
    }

    fn npm_response(package: &str, latest: &str) -> RegistryResponseInput {
        RegistryResponseInput::new(
            package.to_owned(),
            Npm,
            format!(r#"{{"dist-tags":{{"latest":"{latest}"}}}}"#),
        )
    }

    fn npm_versions_response(latest: &str, versions: &[&str]) -> RegistryResponseInput {
        let versions = versions
            .iter()
            .map(|version| (*version, serde_json::json!({})))
            .collect::<std::collections::HashMap<_, _>>();
        RegistryResponseInput::new(
            "left-pad".to_owned(),
            Npm,
            serde_json::json!({"dist-tags": {"latest": latest}, "versions": versions}).to_string(),
        )
    }

    async fn build_code_lens_output(
        fixture: &str,
        latest: &str,
        versions: &[&str],
    ) -> AnalyzeDocumentOutput {
        let session = standard_session();
        let input = package_document(fixture);
        session
            .resolve_document_with_responses(
                input.clone(),
                &[npm_versions_response(latest, versions)],
            )
            .await;
        session.analyze_document(input)
    }

    fn npm_vulnerability_response(
        latest: Option<&str>,
        versions: &[&str],
        id: &str,
        summary: &str,
        affected_range: Option<(&str, &str)>,
    ) -> RegistryResponseInput {
        let mut affected = serde_json::json!({"package": {"name": "left-pad"}});
        if let Some((introduced, fixed)) = affected_range {
            affected["ranges"] = serde_json::json!([{
                "events": [{"introduced": introduced}, {"fixed": fixed}]
            }]);
        } else {
            affected["versions"] = serde_json::json!(versions);
        }

        let mut body = serde_json::json!({
            "vulns": [{
                "id": id,
                "summary": summary,
                "affected": [affected]
            }]
        });
        if let Some(latest) = latest {
            body["dist-tags"] = serde_json::json!({"latest": latest});
        }
        if !versions.is_empty() {
            body["versions"] = serde_json::json!(
                versions
                    .iter()
                    .map(|version| (*version, serde_json::json!({})))
                    .collect::<std::collections::HashMap<_, _>>()
            );
        }
        RegistryResponseInput::new("left-pad".to_owned(), Npm, body.to_string())
    }

    fn session_with_empty_vulnerable_indicator() -> VersionLensSession {
        let mut indicators = test_indicators();
        indicators.updateable_vulnerable.clear();
        session_with_indicators(indicators, true)
    }

    fn session_with_indicators(
        indicators: SuggestionIndicators,
        show_vulnerabilities: bool,
    ) -> VersionLensSession {
        crate::version_lens_session(SessionConfig {
            update_severity: Some(crate::Severity::Information),
            cache_ttl_ms: 300_000,
            enabled_providers: vec![],
            providers: ProviderSettings::default(),
            suggestion_indicators: indicators,
            show_vulnerabilities,
            show_suggestion_stats: false,
            show_prereleases: false,
            http: versionlens_fetch::standard_http_config(),
        })
    }

    fn test_indicators() -> SuggestionIndicators {
        SuggestionIndicators {
            latest: "L".to_owned(),
            satisfies_latest: "S".to_owned(),
            directory: "D".to_owned(),
            error: "E".to_owned(),
            no_match: "N".to_owned(),
            matched: "M".to_owned(),
            updateable: "U".to_owned(),
            updateable_vulnerable: "V".to_owned(),
            build: "B".to_owned(),
        }
    }

    fn standard_session() -> VersionLensSession {
        session_with_indicators(test_indicators(), true)
    }

    fn package_document(fixture: &str) -> DocumentInput {
        DocumentInput::new(
            "file:///package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture(fixture),
            None,
        )
    }

    async fn analyze_npm_fixture_with_response(
        session: &VersionLensSession,
        fixture: &str,
        response: &str,
    ) -> AnalyzeDocumentOutput {
        let input = package_document(fixture);
        crate::test_support::analyze_with_responses(
            session,
            &input,
            &[RegistryResponseInput::new(
                "left-pad".to_owned(),
                Npm,
                response.to_owned(),
            )],
        )
        .await
    }

    fn lens_titles(output: &AnalyzeDocumentOutput) -> Vec<&str> {
        output
            .code_lenses
            .iter()
            .map(|lens| lens.title.as_str())
            .collect()
    }

    fn lens_commands(output: &AnalyzeDocumentOutput) -> Vec<&str> {
        output
            .code_lenses
            .iter()
            .map(crate::presentation::view::LensView::action_kind)
            .collect()
    }

    fn package_file_fixture(name: &str) -> String {
        crate::test_support::fixture("tests/fixtures/session/presentation/codelens", name)
    }
}
