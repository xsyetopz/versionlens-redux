use super::*;

async fn resolve_floating_action(
    responses: Vec<(u16, String)>,
) -> (ResolveDocumentOutput, Vec<String>) {
    let (base_url, server) = github_api_server(responses);
    let output = github_session(base_url)
        .resolve_document(DocumentInput::new(
            "file:///work/.github/workflows/ci.yml",
            "yaml",
            package_file_fixture("floating-action-ref.yaml"),
            None,
        ))
        .await;
    (output, server.join().unwrap())
}

fn assert_current_floating_action(output: &ResolveDocumentOutput) {
    assert_eq!(output.suggestions[0].status, "current");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("stable"));
    assert!(output.edits.is_empty());
}

fn assert_local_reference_statuses(output: &ResolveDocumentOutput) {
    assert_eq!(
        output
            .suggestions
            .iter()
            .map(|suggestion| suggestion.status.as_str())
            .collect::<Vec<_>>(),
        ["fixed", "fixed", "error", "error"]
    );
    assert!(output.edits.is_empty());
}

#[tokio::test]
async fn floating_action_tags_take_precedence_and_preserve_the_reference() {
    let sha = "a".repeat(40);
    let (output, paths) = resolve_floating_action(vec![
        (200, r#"[{"name":"v1.0.0"}]"#.to_owned()),
        (
            200,
            serde_json::json!({
                "ref": "refs/tags/stable",
                "object": {"type": "tag", "sha": sha}
            })
            .to_string(),
        ),
    ])
    .await;
    assert_eq!(
        paths,
        [
            "/repos/acme/action/tags",
            "/repos/acme/action/git/ref/tags/stable"
        ]
    );
    assert_current_floating_action(&output);
}

#[tokio::test]
async fn floating_action_branches_are_checked_after_an_absent_tag() {
    let sha = "b".repeat(40);
    let (output, paths) = resolve_floating_action(vec![
        (200, r#"[{"name":"v1.0.0"}]"#.to_owned()),
        (404, r#"{"message":"Not Found"}"#.to_owned()),
        (
            200,
            serde_json::json!({
                "ref": "refs/heads/stable",
                "object": {"type": "commit", "sha": sha}
            })
            .to_string(),
        ),
    ])
    .await;
    assert_eq!(
        paths,
        [
            "/repos/acme/action/tags",
            "/repos/acme/action/git/ref/tags/stable",
            "/repos/acme/action/git/ref/heads/stable"
        ]
    );
    assert_current_floating_action(&output);
}

#[tokio::test]
async fn floating_action_ref_failures_are_explicit() {
    let cases = [
        (
            vec![
                (200, r#"[]"#.to_owned()),
                (404, r#"{"message":"Not Found"}"#.to_owned()),
                (404, r#"{"message":"Not Found"}"#.to_owned()),
            ],
            "unavailable",
        ),
        (
            vec![
                (200, r#"[]"#.to_owned()),
                (
                    200,
                    serde_json::json!([
                        {"ref":"refs/tags/stable","object":{"type":"commit","sha":"a".repeat(40)}},
                        {"ref":"refs/tags/stable/old","object":{"type":"commit","sha":"b".repeat(40)}}
                    ])
                    .to_string(),
                ),
            ],
            "ambiguous",
        ),
        (
            vec![
                (200, r#"[]"#.to_owned()),
                (
                    200,
                    r#"{"ref":"refs/tags/stable","object":{"type":"commit","sha":"abcdef0"}}"#
                        .to_owned(),
                ),
            ],
            "invalid",
        ),
    ];
    for (responses, message) in cases {
        let (output, _) = resolve_floating_action(responses).await;

        assert_eq!(output.suggestions[0].status, "error");
        assert!(
            output.suggestions[0]
                .latest
                .as_deref()
                .is_some_and(|value| value.contains(message)),
            "{output:?}"
        );
        assert!(output.edits.is_empty());
    }
}

#[tokio::test]
async fn local_action_references_use_the_declared_workspace_and_expressions_fail_explicitly() {
    let root = temp_dir().join(format!("versionlens-github-local-{}", id()));
    create_dir_all(root.join(".github/workflows")).unwrap();
    create_dir_all(root.join("actions/build")).unwrap();
    write(
        root.join(".github/workflows/release.yml"),
        "on: workflow_call\n",
    )
    .unwrap();
    write(
        root.join("actions/build/action.yml"),
        "runs: {using: composite, steps: []}\n",
    )
    .unwrap();
    let input = DocumentInput::new(
        format!("file://{}/.github/workflows/ci.yml", root.display()),
        "yaml",
        package_file_fixture("local-action-reference-kinds.yaml"),
        Some(root.to_string_lossy().into_owned()),
    );

    let output = session_without_vulnerabilities()
        .resolve_document(input)
        .await;

    assert_local_reference_statuses(&output);
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("./.github/workflows/release.yml")
    );
    assert_eq!(
        output.suggestions[1].latest.as_deref(),
        Some("./actions/build")
    );
    assert!(
        output.suggestions[2]
            .latest
            .as_deref()
            .is_some_and(|value| value.contains("expression"))
    );
    assert_eq!(
        output.suggestions[3].latest.as_deref(),
        Some("invalid GitHub action reference")
    );
    remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn unsaved_local_targets_are_verified_from_one_workspace_snapshot_without_network() {
    let root = temp_dir().join(format!("versionlens-github-unsaved-{}", id()));
    create_dir_all(&root).unwrap();
    let workspace_root = root.to_string_lossy().into_owned();
    let document = |relative: &str, text: &str, version| {
        DocumentInput::new(
            format!("file://{}/{relative}", root.display()),
            "yaml",
            text,
            Some(workspace_root.clone()),
        )
        .with_version(version)
    };
    let action = document(
        "actions/build/action.yml",
        "name: build\nruns: {using: composite, steps: []}\n",
        1,
    );
    let workflow = document(
        ".github/workflows/release.yml",
        "on:\n  workflow_call:\n",
        1,
    );
    let input = document(
        ".github/workflows/ci.yml",
        &package_file_fixture("local-action-reference-kinds.yaml"),
        1,
    );
    let (base_url, server) = github_api_server(Vec::<(u16, String)>::new());
    let session = github_session(base_url);
    assert!(session.set_workspace_documents(vec![action, workflow]));

    let output = session.resolve_document(input).await;

    assert_local_reference_statuses(&output);
    assert!(server.join().unwrap().is_empty());
    remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn edited_unsaved_workflow_invalidates_a_cached_local_result() {
    let root = temp_dir().join(format!("versionlens-github-edit-{}", id()));
    create_dir_all(root.join(".github/workflows")).unwrap();
    write(root.join(".github/workflows/release.yml"), "on: push\n").unwrap();
    let workspace_root = root.to_string_lossy().into_owned();
    let target = |text: &str, version| {
        DocumentInput::new(
            format!("file://{}/.github/workflows/release.yml", root.display()),
            "yaml",
            text,
            Some(workspace_root.clone()),
        )
        .with_version(version)
    };
    let input = DocumentInput::new(
        format!("file://{}/.github/workflows/ci.yml", root.display()),
        "yaml",
        "jobs:\n  shared:\n    uses: ./.github/workflows/release.yml\n",
        Some(workspace_root.clone()),
    );
    let session = session_without_vulnerabilities();
    assert!(session.set_workspace_documents(vec![target("on:\n  workflow_call:\n", 1,)]));
    let valid = session.resolve_document(input.clone()).await;
    assert_eq!(valid.suggestions[0].status, "fixed");

    assert!(session.set_workspace_documents(vec![target("on: push\n", 2)]));
    let invalid = session.resolve_document(input).await;

    assert_eq!(invalid.suggestions[0].status, "error");
    assert!(
        invalid.suggestions[0]
            .latest
            .as_deref()
            .is_some_and(|value| value.contains("invalid"))
    );
    remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn local_action_references_report_unavailable_roots_and_ambiguous_metadata() {
    let without_root = session_without_vulnerabilities()
        .resolve_document(DocumentInput::new(
            "file:///work/.github/workflows/ci.yml",
            "yaml",
            "steps:\n  - uses: ./actions/build\n",
            None,
        ))
        .await;
    assert_eq!(without_root.suggestions[0].status, "error");
    assert!(
        without_root.suggestions[0]
            .latest
            .as_deref()
            .is_some_and(|value| value.contains("workspace root unavailable"))
    );

    let root = temp_dir().join(format!("versionlens-github-contained-{}", id()));
    create_dir_all(&root).unwrap();
    let escaped = session_without_vulnerabilities()
        .resolve_document(DocumentInput::new(
            format!("file://{}/.github/workflows/ci.yml", root.display()),
            "yaml",
            "steps:\n  - uses: ./../outside\n",
            Some(root.to_string_lossy().into_owned()),
        ))
        .await;
    assert_eq!(escaped.suggestions[0].status, "error");
    assert_eq!(
        escaped.suggestions[0].latest.as_deref(),
        Some("invalid local GitHub action path")
    );
    remove_dir_all(root).unwrap();

    let root = temp_dir().join(format!("versionlens-github-ambiguous-{}", id()));
    create_dir_all(root.join("actions/build")).unwrap();
    write(root.join("actions/build/action.yml"), "name: first\n").unwrap();
    write(root.join("actions/build/action.yaml"), "name: second\n").unwrap();
    let output = session_without_vulnerabilities()
        .resolve_document(DocumentInput::new(
            format!("file://{}/.github/workflows/ci.yml", root.display()),
            "yaml",
            "steps:\n  - uses: ./actions/build\n",
            Some(root.to_string_lossy().into_owned()),
        ))
        .await;
    assert_eq!(output.suggestions[0].status, "error");
    assert!(
        output.suggestions[0]
            .latest
            .as_deref()
            .is_some_and(|value| value.contains("ambiguous"))
    );
    remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn escaped_workflow_references_preserve_surrounding_yaml_on_update() {
    let source = r#"steps: [{uses: "acme\u002faction\u0040v\u0031.0.0", name: keep}] # retained"#;
    let output = session_without_vulnerabilities()
        .resolve_document_with_responses(
            DocumentInput::new(
                "file:///work/.github/workflows/ci.yml",
                "yaml",
                source,
                None,
            ),
            &[RegistryResponseInput::new(
                "acme/action",
                GitHub,
                r#"[{"name":"v1.0.0"},{"name":"v2.0.0"}]"#,
            )],
        )
        .await;
    assert_eq!(output.edits.len(), 1, "{output:?}");
    let edit = &output.edits[0];
    let mut updated = source.to_owned();
    updated.replace_range(
        edit.range.start.character as usize..edit.range.end.character as usize,
        &edit.new_text,
    );
    assert_eq!(
        updated,
        r#"steps: [{uses: "acme\u002faction\u0040v2.0.0", name: keep}] # retained"#
    );
}

#[tokio::test]
async fn cached_action_results_are_isolated_by_tag_family_and_commit() {
    let session = session_without_vulnerabilities();
    let input = |reference: &str| {
        DocumentInput::new(
            "file:///work/.github/workflows/ci.yml",
            "yaml",
            format!("steps:\n  - uses: acme/action@{reference}\n"),
            None,
        )
    };
    let first = input("v1.0.0");
    session
        .resolve_document_with_responses(
            first,
            &[RegistryResponseInput::new(
                "acme/action".to_owned(),
                GitHub,
                r#"[{"name":"v1.0.0"},{"name":"v2.0.0"}]"#.to_owned(),
            )],
        )
        .await;
    for reference in [
        "release-v1.0.0",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa # v1.0.0",
    ] {
        let dependency = parse_document(&input(reference)).remove(0);
        assert!(
            session
                .cached_resolved_suggestion(&dependency, &cache_scope(&session, &input(reference)))
                .is_none()
        );
    }
}

#[tokio::test]
async fn github_tag_resolution_reads_subsequent_pages() {
    let page = serde_json::Value::Array(
        (0..30)
            .map(|index| serde_json::json!({"name": format!("unrelated-v1.0.{index}")}))
            .collect(),
    )
    .to_string();
    let (base_url, server) = github_api_server(vec![
        (200, page),
        (200, r#"[{"name":"v1.0.0"},{"name":"v2.0.0"}]"#.to_owned()),
    ]);
    let session = github_session(base_url);
    let output = session
        .resolve_document(DocumentInput::new(
            "file:///work/.github/workflows/ci.yml",
            "yaml",
            "steps:\n  - uses: acme/action@v1.0.0\n",
            None,
        ))
        .await;
    assert_eq!(
        server.join().unwrap(),
        [
            "/repos/acme/action/tags",
            "/repos/acme/action/tags?per_page=30&page=2"
        ]
    );
    assert_eq!(output.edits[0].new_text, "v2.0.0");
}

#[tokio::test]
async fn cached_action_suggestions_use_the_current_document_ranges() {
    let session = session_without_vulnerabilities();
    let document = |text: &str| {
        DocumentInput::new("file:///work/.github/workflows/ci.yml", "yaml", text, None)
    };
    session
        .resolve_document_with_responses(
            document("steps:\n  - uses: acme/action@v1.0.0\n"),
            &[RegistryResponseInput::new(
                "acme/action".to_owned(),
                GitHub,
                r#"[{"name":"v1.0.0"},{"name":"v2.0.0"}]"#.to_owned(),
            )],
        )
        .await;
    let moved = document("# moved\nsteps:\n  - uses: acme/action@v1.0.0\n");
    let dependency = parse_document(&moved).remove(0);
    let cached = session
        .cached_resolved_suggestion(&dependency, &cache_scope(&session, &moved))
        .unwrap();
    assert_eq!(
        cached.dependency.requirement_range,
        dependency.requirement_range
    );
    assert_eq!(cached.dependency.requirement_range.start.line, 2);
}

fn cache_scope(session: &VersionLensSession, input: &DocumentInput) -> String {
    let context = session.registry_context(input, session.classify_document(input));
    session.document_cache_scope(&context, input)
}

#[tokio::test]
async fn nested_annotated_tags_resolve_to_the_pinned_commit() {
    let commit = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let outer = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let inner = "cccccccccccccccccccccccccccccccccccccccc";
    let mut responses = annotated_tag_start(outer);
    responses.extend([
        (
            200,
            serde_json::json!({"sha":outer,"object":{"type":"tag","sha":inner}}).to_string(),
        ),
        (
            200,
            serde_json::json!({"sha":inner,"object":{"type":"commit","sha":commit}}).to_string(),
        ),
    ]);
    let (base_url, server) = github_api_server(responses);
    let output = github_session(base_url)
        .resolve_document(DocumentInput::new(
            "file:///work/.github/workflows/ci.yml",
            "yaml",
            format!("steps:\n  - uses: acme/action@{commit} # v2\n"),
            None,
        ))
        .await;
    assert_eq!(
        server.join().unwrap(),
        [
            "/repos/acme/action/tags".to_owned(),
            "/repos/acme/action/git/ref/tags/v2".to_owned(),
            format!("/repos/acme/action/git/tags/{outer}"),
            format!("/repos/acme/action/git/tags/{inner}"),
        ]
    );
    assert_eq!(output.suggestions[0].status, "current");
}

#[tokio::test]
async fn cyclic_annotated_tags_finish_with_a_specific_failure() {
    let tag = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let mut responses = annotated_tag_start(tag);
    responses.push((
        200,
        serde_json::json!({"sha":tag,"object":{"type":"tag","sha":tag}}).to_string(),
    ));
    let (base_url, server) = github_api_server(responses);
    let output = github_session(base_url)
        .resolve_document(DocumentInput::new(
            "file:///work/.github/workflows/release.yml",
            "yaml",
            "steps:\n  - uses: acme/action@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa # v2\n",
            None,
        ))
        .await;
    assert_eq!(server.join().unwrap().len(), 3);
    assert!(output.edits.is_empty());
    assert_eq!(output.suggestions[0].status, "error");
    assert!(
        output.suggestions[0]
            .latest
            .as_deref()
            .unwrap()
            .contains("cyclic GitHub tag reference")
    );
}

#[tokio::test]
async fn commit_pins_resolve_release_families_and_keep_pin_format() {
    let current = "a".repeat(40);
    let latest = "b".repeat(40);
    let body = commit_tags(&current, &latest);
    let responses = [RegistryResponseInput::new("acme/action", GitHub, &body)];
    let session = session_without_vulnerabilities();
    for (pin, expected_status) in [(&current, "updateAvailable"), (&latest, "current")] {
        let input = DocumentInput::new(
            "file:///work/.github/workflows/ci.yml",
            "yaml",
            format!("steps: [{{uses: 'acme/action@{pin}', name: keep}}] # preserved\n"),
            None,
        );
        let output = session
            .resolve_document_with_responses(input, &responses)
            .await;
        assert_eq!(output.suggestions.len(), 1, "{output:?}");
        assert_eq!(output.suggestions[0].status, expected_status, "{output:?}");
        if pin == &current {
            assert_eq!(output.edits.len(), 1);
            assert_eq!(output.edits[0].new_text, latest);
            assert_eq!(
                output.edits[0].range.end.character - output.edits[0].range.start.character,
                40
            );
        } else {
            assert!(output.edits.is_empty());
        }
    }
}

#[tokio::test]
async fn commit_pins_with_multiple_tag_families_have_a_checked_outcome() {
    let commit = "a".repeat(40);
    let body = serde_json::json!([
        {"name":"v1.0.0","commit":{"sha":commit}},
        {"name":"release-v3.0.0","commit":{"sha":commit}}
    ])
    .to_string();
    let session = session_without_vulnerabilities();
    let input = DocumentInput::new(
        "file:///work/.github/workflows/ci.yml",
        "yaml",
        format!("steps:\n  - uses: acme/action@{commit}\n"),
        None,
    );
    let output = session
        .resolve_document_with_responses(
            input.clone(),
            &[RegistryResponseInput::new("acme/action", GitHub, &body)],
        )
        .await;
    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "error");
    assert!(output.edits.is_empty());
    assert!(session.document_is_fresh(&input));
}

#[tokio::test]
async fn abbreviated_action_commit_is_checked_through_the_repository_endpoint() {
    let current = "a".repeat(40);
    let latest = "b".repeat(40);
    let body = commit_tags(&current, &latest);
    let (base_url, server) = github_api_server(vec![(200, body)]);
    let output = github_session(base_url)
        .resolve_document(DocumentInput::new(
            "file:///work/.github/workflows/release.yml",
            "yaml",
            "steps:\n  - uses: acme/action@aaaaaaa # reviewed\n",
            None,
        ))
        .await;
    assert_eq!(server.join().unwrap(), ["/repos/acme/action/tags"]);
    assert_eq!(output.edits.len(), 1, "{output:?}");
    assert_eq!(output.edits[0].new_text, latest);
}

#[tokio::test]
async fn action_commit_selections_require_a_verified_release_replacement() {
    let current = "a".repeat(40);
    let body = serde_json::json!([
        {"name":"v1.0.0","commit":{"sha":"c".repeat(40)}},
        {"name":"v2.0.0","commit":{"sha":current}},
        {"name":"v3.0.0","commit":{"sha":"b".repeat(40)}}
    ])
    .to_string();
    let responses = [RegistryResponseInput::new("acme/action", GitHub, &body)];
    let input = DocumentInput::new(
        "file:///work/.github/workflows/ci.yml",
        "yaml",
        format!("steps:\n  - uses: acme/action@{current}\n"),
        None,
    );
    for version in ["1.0.0", "99.0.0"] {
        let output = session_without_vulnerabilities()
            .apply_command_with_selected_version(crate::ApplyCommandRequest {
                input: input.clone(),
                command: Some("update"),
                dependency_name: Some("acme/action"),
                selected_version: Some(version),
                responses: &responses,
            })
            .await;
        assert!(output.edits.is_empty(), "{output:?}");
    }
}

fn annotated_tag_start(tag: &str) -> Vec<(u16, String)> {
    vec![
        (200, r#"[{"name":"v2"}]"#.to_owned()),
        (
            200,
            serde_json::json!({"ref":"refs/tags/v2","object":{"type":"tag","sha":tag}}).to_string(),
        ),
    ]
}

fn commit_tags(current: &str, latest: &str) -> String {
    serde_json::json!([
        {"name":"v1.0.0","commit":{"sha":current}},
        {"name":"v2.0.0","commit":{"sha":latest}}
    ])
    .to_string()
}
