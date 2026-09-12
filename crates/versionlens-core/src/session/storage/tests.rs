use crate::cache::vulnerability_cache_key;
use crate::{ApplyCommandRequest, RegistryResponseInput, RegistryUrlConfig, VersionLensSession};
use std::time::Duration;
use versionlens_cache::PersistentRecord;
use versionlens_model::{CanonicalReference, DocumentInput, Ecosystem::Dotnet, Ecosystem::Npm};
use versionlens_suggestions::{Suggestion, SuggestionStatus};

fn session(directory: &std::path::Path, credential: &str) -> VersionLensSession {
    configured_session(directory, credential, "http://127.0.0.1:1", None)
}

fn configured_session(
    directory: &std::path::Path,
    credential: &str,
    registry_url: &str,
    cache_ttl_ms: Option<u64>,
) -> VersionLensSession {
    let mut config = crate::support::tests::session_config(crate::default(), false);
    config.providers.registry_urls = vec![RegistryUrlConfig {
        ecosystem: Npm,
        url: registry_url.to_owned(),
    }];
    if let Some(cache_ttl_ms) = cache_ttl_ms {
        config.cache_ttl_ms = cache_ttl_ms;
    }
    config.http.timeout_ms = 20;
    config.http.auth_headers = vec![versionlens_http::HttpHeader {
        name: "Authorization".to_owned(),
        value: credential.to_owned(),
        url: Some("http://127.0.0.1:1".to_owned()),
    }];
    VersionLensSession::new(config)
        .with_persistent_cache(directory)
        .unwrap()
}

fn dotnet_session(directory: &std::path::Path) -> VersionLensSession {
    let mut config = crate::support::tests::session_config(crate::default(), false);
    config.providers.registry_urls = vec![RegistryUrlConfig {
        ecosystem: Dotnet,
        url: "http://127.0.0.1:1".to_owned(),
    }];
    config.http.timeout_ms = 20;
    VersionLensSession::new(config)
        .with_persistent_cache(directory)
        .unwrap()
}

fn input() -> DocumentInput {
    DocumentInput::new(
        "file:///workspace/package.json",
        "json",
        r#"{"dependencies":{"example":"1.0.0"}}"#,
        None,
    )
}

fn populate(session: &VersionLensSession) {
    populate_input(session, input());
}

fn populate_input(session: &VersionLensSession, input: DocumentInput) {
    let output = session.resolve_document_with_responses(
        input,
        &[RegistryResponseInput::new(
            "example".to_owned(),
            Npm,
            r#"{"dist-tags":{"latest":"2.0.0"}}"#.to_owned(),
        )],
    );
    assert_eq!(output.edits[0].new_text, "2.0.0");
}

fn failed_suggestion(dependency: versionlens_model::Dependency) -> Suggestion {
    Suggestion {
        dependency,
        latest: None,
        resolved: Some("registry unavailable".to_owned()),
        status: SuggestionStatus::Error,
        builds: vec![],
        choices: vec![],
    }
}

#[test]
fn session_restart_reuses_checked_versions_without_a_registry_connection() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    populate(&session(&directory, "credential-a"));
    let restarted = session(&directory, "credential-a");
    let output = restarted.resolve_document(input());
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "2.0.0");
    let disk = std::fs::read_to_string(directory.join("cache.json")).unwrap();
    assert!(!disk.contains("credential-a"));
    assert!(!disk.contains("127.0.0.1"));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn session_restart_preserves_dotnet_fixed_response_semantics() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    let input = DocumentInput::new(
        "file:///coverage/app.csproj",
        "xml",
        "<Project><ItemGroup><PackageReference Include=\"Foo\" Version=\"1.0.0\" /></ItemGroup></Project>\n",
        None,
    );
    let first = session(&directory, "credential-a").resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput::new(
            "Foo",
            Dotnet,
            r#"{"versions":["1.0.0","2.0.0"]}"#,
        )],
    );
    assert_eq!(first.suggestions[0].status, "fixed");
    assert_eq!(first.suggestions[0].latest.as_deref(), Some("1.0.0"));

    let restarted = session(&directory, "credential-a");
    let cached = restarted.resolve_document(input.clone());
    assert_eq!(cached.suggestions[0].status, first.suggestions[0].status);
    assert_eq!(cached.suggestions[0].latest, first.suggestions[0].latest);
    assert!(cached.edits.is_empty());

    let analysis = restarted.analyze_document(input.clone());
    assert!(
        analysis
            .code_lenses
            .iter()
            .any(|lens| { lens.arguments.iter().any(|argument| argument == "2.0.0") })
    );

    let update = restarted.apply_command_with_selected_version(ApplyCommandRequest {
        input: input.clone(),
        command: Some("update"),
        dependency_name: Some("Foo"),
        selected_version: Some("2.0.0"),
        responses: &[],
    });
    assert_eq!(update.edits.len(), 1);
    assert_eq!(update.edits[0].new_text, "2.0.0");
    let requirement_start = u32::try_from(input.text.find("1.0.0").unwrap()).unwrap();
    assert_eq!(update.edits[0].range.start.line, 0);
    assert_eq!(update.edits[0].range.start.character, requirement_start);
    assert_eq!(update.edits[0].range.end.character, requirement_start + 5);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn fixed_pin_revalidates_cached_latest_record_without_proof_field() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    let input = DocumentInput::new(
        "file:///coverage/app.csproj",
        "xml",
        "<Project><ItemGroup><PackageReference Include=\"Foo\" Version=\"1.0.0\" /></ItemGroup></Project>\n",
        None,
    );
    let first = dotnet_session(&directory);
    let dependency = first.dependencies(&input).remove(0);
    let manifest_kind = first.classify_document(&input);
    let context = first.registry_context(&input, manifest_kind);
    let key = first.persistent_latest_key(&dependency, &context).unwrap();
    let cache = first.storage_state.persistent_cache.as_ref().unwrap();
    let epoch = cache.epoch().unwrap();
    let now = super::timestamp_ms();
    cache
        .insert(
            epoch,
            key,
            PersistentRecord {
                value: serde_json::json!({
                    "latest": "2.0.0",
                    "builds": [],
                    "choices": []
                }),
                attempted_at_ms: now,
                succeeded_at_ms: Some(now),
                expires_at_ms: now + 60_000,
                retry_at_ms: None,
                accessed_at_ms: now,
            },
            now,
        )
        .unwrap();
    drop(first);

    let revalidated = dotnet_session(&directory).resolve_document(input);
    assert_eq!(revalidated.suggestions[0].status, "error");
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn restart_restores_semantic_freshness_with_current_document_ranges() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    let initial = DocumentInput::new(
        "file:///workspace/package.json5",
        "json5",
        "{'dependencies':{'example':'1.0.0'}}",
        None,
    );
    populate_input(&session(&directory, "credential-a"), initial);

    let moved = DocumentInput::new(
        "file:///workspace/package.json5",
        "json5",
        "{\n  \"dependencies\": {\n    \"example\": \"1.0.0\"\n  }\n}",
        None,
    );
    let restarted = session(&directory, "credential-a");
    assert!(restarted.document_is_fresh(&moved));
    let analyzed = restarted.analyze_document(moved);
    assert!(!analyzed.code_lenses.is_empty());
    assert!(
        analyzed
            .code_lenses
            .iter()
            .all(|lens| lens.range.start.line == 2)
    );

    let disk = std::fs::read_to_string(directory.join("cache.json")).unwrap();
    assert!(!disk.contains("credential-a"));
    assert!(!disk.contains("requirementRange"));
    assert!(!disk.contains("requirementPrefix"));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn failed_outcomes_survive_restart_only_until_their_retry_deadline() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    let first = session(&directory, "credential-a");
    let failed = first.resolve_document(input());
    assert_eq!(failed.suggestions[0].status, "error");

    let restarted = session(&directory, "credential-a");
    assert!(restarted.document_is_fresh(&input()));

    let dependency = restarted.dependencies(&input()).remove(0);
    let kind = restarted.classify_document(&input());
    let context = restarted.registry_context(&input(), kind);
    let scope = restarted.document_cache_scope(&context, &input());
    let operation = restarted.operation_context();
    let key = restarted.persistent_suggestion_key(&dependency, &scope);
    restarted.store_persistent_suggestions(
        std::iter::once((key, &failed_suggestion(dependency), Duration::ZERO)),
        &operation,
    );

    let expired = session(&directory, "credential-a");
    assert!(!expired.document_is_fresh(&input()));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn local_vulnerability_status_is_available_after_restart_without_disk_records() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    let first = session(&directory, "credential-a");
    let local = DocumentInput::new(
        "file:///workspace/package.json",
        "json",
        r#"{"dependencies":{"example":"file:../example"}}"#,
        None,
    );
    let dependency = first.dependencies(&local).remove(0);
    let operation = first.operation_context();
    first.store_persistent_vulnerability(
        &dependency,
        &crate::vulnerability::VulnerabilityCheck::Unsupported,
        Duration::from_secs(60),
        &operation,
    );
    let cache = first.storage_state.persistent_cache.as_ref().unwrap();
    assert!(
        cache
            .get(
                &first.persistent_vulnerability_key(&dependency),
                super::timestamp_ms()
            )
            .unwrap()
            .is_none()
    );
    let restarted = session(&directory, "credential-a");
    assert!(matches!(
        restarted.load_persistent_vulnerability(&dependency),
        Some((crate::vulnerability::VulnerabilityCheck::Unsupported, ttl)) if !ttl.is_zero()
    ));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn restarted_session_rechecks_changed_local_action_manifests() {
    let directory =
        versionlens_test_support::temporary_directory("versionlens-local-action").unwrap();
    let root = directory.join("workspace");
    let action = root.join("tool/action.yml");
    std::fs::create_dir_all(action.parent().unwrap()).unwrap();
    std::fs::write(&action, "runs: {using: composite, steps: []}\n").unwrap();
    let input = DocumentInput::new(
        crate::workspace_file_uri(&root.join(".github/workflows/ci.yml")).unwrap(),
        "yaml",
        "jobs: {check: {steps: [{uses: './tool'}]}}\n",
        Some(root.to_string_lossy().into_owned()),
    );
    let storage = directory.join("cache");
    let first = session(&storage, "credential-a").resolve_document(input.clone());
    assert_eq!(first.suggestions[0].status, "fixed");

    std::fs::write(&action, "runs: {}\n").unwrap();
    let restarted = session(&storage, "credential-a");
    assert!(!restarted.document_is_fresh(&input));
    let output = restarted.resolve_document(input);
    assert_eq!(output.suggestions[0].status, "error");
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn vulnerability_empty_and_failed_outcomes_remain_distinct_after_restart() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    let mut first = session(&directory, "credential-a");
    first.config.show_vulnerabilities = true;
    let dependency = first.dependencies(&input()).remove(0);
    let operation = first.operation_context();
    first.store_persistent_vulnerability(
        &dependency,
        &crate::vulnerability::VulnerabilityCheck::Checked(vec![]),
        Duration::from_secs(60),
        &operation,
    );
    let mut restarted = session(&directory, "credential-a");
    restarted.config.show_vulnerabilities = true;
    restarted.analyze_document(input());
    assert!(matches!(
        restarted.vulnerability_cache().get(&vulnerability_cache_key(&dependency)),
        Some(crate::vulnerability::VulnerabilityCheck::Checked(advisories))
            if advisories.is_empty()
    ));

    let operation = restarted.operation_context();
    restarted.store_persistent_vulnerability(
        &dependency,
        &crate::vulnerability::VulnerabilityCheck::Failed("service unavailable".to_owned()),
        Duration::from_secs(60),
        &operation,
    );
    let mut failed = session(&directory, "credential-a");
    failed.config.show_vulnerabilities = true;
    failed.analyze_document(input());
    assert!(matches!(
        failed.vulnerability_cache().get(&vulnerability_cache_key(&dependency)),
        Some(crate::vulnerability::VulnerabilityCheck::Failed(message))
            if message == "service unavailable"
    ));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn persistent_semantic_keys_ignore_formatting_but_isolate_canonical_identity() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    let first = session(&directory, "credential-a");
    let original = first.dependencies(&input()).remove(0);
    let kind = first.classify_document(&input());
    let context = first.registry_context(&input(), kind);
    let scope = first.document_cache_scope(&context, &input());
    let original_key = first.persistent_suggestion_key(&original, &scope);

    let mut formatted = original;
    formatted.range.start.line = 99;
    formatted.requirement_range.start.character = 42;
    formatted.requirement_prefix = "'".to_owned();
    formatted.requirement_suffix = "'".to_owned();
    assert_eq!(
        original_key,
        first.persistent_suggestion_key(&formatted, &scope)
    );

    formatted.canonical_reference = Some(CanonicalReference::GitHubActionTag {
        tag: "v1".to_owned(),
    });
    assert_ne!(
        original_key,
        first.persistent_suggestion_key(&formatted, &scope)
    );
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn persistent_semantic_keys_isolate_authentication_source_and_policy() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    let input = input();
    let key = |session: &VersionLensSession| {
        let dependency = session.dependencies(&input).remove(0);
        let kind = session.classify_document(&input);
        let context = session.registry_context(&input, kind);
        let scope = session.document_cache_scope(&context, &input);
        session.persistent_suggestion_key(&dependency, &scope)
    };
    let baseline = configured_session(&directory, "credential-a", "http://127.0.0.1:1", None);
    let different_auth = configured_session(&directory, "credential-b", "http://127.0.0.1:1", None);
    let different_source =
        configured_session(&directory, "credential-a", "http://127.0.0.1:2", None);
    let different_policy = configured_session(
        &directory,
        "credential-a",
        "http://127.0.0.1:1",
        Some(123_456),
    );
    let baseline = key(&baseline);
    assert_ne!(baseline, key(&different_auth));
    assert_ne!(baseline, key(&different_source));
    assert_ne!(baseline, key(&different_policy));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn persistent_outcomes_record_attempt_success_expiry_and_retry_separately() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    let session = session(&directory, "credential-a");
    let dependency = session.dependencies(&input()).remove(0);
    let kind = session.classify_document(&input());
    let context = session.registry_context(&input(), kind);
    let scope = session.document_cache_scope(&context, &input());
    let key = session.persistent_suggestion_key(&dependency, &scope);
    let operation = session.operation_context();
    session.store_persistent_suggestions(
        std::iter::once((
            key.clone(),
            &failed_suggestion(dependency.clone()),
            Duration::from_secs(60),
        )),
        &operation,
    );
    let cache = session.storage_state.persistent_cache.as_ref().unwrap();
    let failure = cache.get(&key, super::timestamp_ms()).unwrap().unwrap();
    assert_eq!(failure.attempted_at_ms, operation.attempted_at_ms);
    assert_eq!(failure.succeeded_at_ms, None);
    assert_eq!(failure.retry_at_ms, Some(failure.expires_at_ms));

    let success = Suggestion {
        dependency,
        latest: Some("2.0.0".to_owned()),
        resolved: None,
        status: SuggestionStatus::UpdateAvailable,
        builds: vec![],
        choices: vec![],
    };
    let operation = session.operation_context();
    session.store_persistent_suggestions(
        std::iter::once((key.clone(), &success, Duration::from_secs(60))),
        &operation,
    );
    let success = cache.get(&key, super::timestamp_ms()).unwrap().unwrap();
    assert_eq!(success.attempted_at_ms, operation.attempted_at_ms);
    assert!(success.succeeded_at_ms.is_some());
    assert_eq!(success.retry_at_ms, None);
    assert!(success.expires_at_ms > success.succeeded_at_ms.unwrap());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn clear_epoch_rejects_a_late_semantic_outcome_write() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    let first = session(&directory, "credential-a");
    let dependency = first.dependencies(&input()).remove(0);
    let kind = first.classify_document(&input());
    let context = first.registry_context(&input(), kind);
    let scope = first.document_cache_scope(&context, &input());
    let key = first.persistent_suggestion_key(&dependency, &scope);
    let operation = first.operation_context();
    first.try_clear_cache().unwrap();
    first.store_persistent_suggestions(
        std::iter::once((key, &failed_suggestion(dependency), Duration::from_secs(60))),
        &operation,
    );

    assert!(!session(&directory, "credential-a").document_is_fresh(&input()));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn authentication_partitions_require_independent_checks() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    populate(&session(&directory, "credential-a"));
    let other = session(&directory, "credential-b");
    assert_check_failed(&other);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn clearing_persistent_results_requires_a_new_successful_check() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    let first = session(&directory, "credential-a");
    populate(&first);
    first.clear_cache();
    let restarted = session(&directory, "credential-a");
    assert_check_failed(&restarted);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn a_clear_from_another_session_invalidates_memory_results() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    let first = session(&directory, "credential-a");
    populate(&first);
    assert!(!first.analyze_document(input()).code_lenses.is_empty());
    session(&directory, "credential-a")
        .try_clear_cache()
        .unwrap();
    assert!(first.analyze_document(input()).code_lenses.is_empty());
    assert_check_failed(&first);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn a_disk_generation_change_invalidates_pending_publication() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    let first = session(&directory, "credential-a");
    let operation = first.operation_context();
    assert!(operation.can_publish());
    session(&directory, "credential-a")
        .try_clear_cache()
        .unwrap();
    assert!(!operation.can_publish());
    std::fs::remove_dir_all(directory).unwrap();
}

fn assert_check_failed(session: &VersionLensSession) {
    let output = session.resolve_document(input());
    assert!(output.edits.is_empty());
    assert_eq!(output.suggestions[0].status, "error");
}
