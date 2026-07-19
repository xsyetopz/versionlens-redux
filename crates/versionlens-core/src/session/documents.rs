use crate::selection::matches_dependency;
use std::collections::HashSet;
use versionlens_edits::can_sort_dependencies;
use versionlens_edits::update_edits;

use versionlens_model::{DocumentInput, ManifestKind, ecosystem_for_manifest};
use versionlens_suggestions::Suggestion;
use versionlens_versions::ProjectVersionBump;
use versionlens_vscode_model::DiagnosticPayload;

use super::operation::OperationContext;
use super::resolution::ResolutionRequest;
use crate::VersionLensSession;
use crate::command::install_task_config_key_for_manifest;
use crate::contract::{AnalyzeDocumentOutput, RegistryResponseInput, ResolveDocumentOutput};
use crate::dependency::into_dependency_payloads;
use crate::registry;
use crate::schema::schema_output;
use crate::snapshot::dependency_signature;
use crate::status::{status_payload, to_u32};
use crate::suggestion::into_suggestion_payloads;
use versionlens_model::ManifestKind::{
    AnsibleGalaxyRequirementsYaml, BazelModule, BazelWorkspace, Cabal, CabalProject, CargoToml,
    ClojureDepsEdn, Cmake, CocoaPodsPodfile, ComposerJson, ConanfilePy, ConanfileTxt, Cpanfile,
    DenoImportMapJson, DenoJson, DockerComposeYaml, Dockerfile, DotnetProjectJson, DotnetXml,
    DubJson, DubSdl, DuneProject, Gemfile, GleamToml, GoMod, GradleBuild, GradleSettings,
    GradleVersionCatalogToml, HaxelibJson, HelmChartYaml, JsrJson, JuliaManifestToml,
    JuliaProjectToml, KustomizationYaml, LeiningenProjectClj, LuaRockspec, MavenPomXml, MesonWrap,
    MixExs, Nimble, NixFlake, NpmPackageJson, NpmPackageJson5, NpmPackageYaml, Opam,
    PaketDependencies, PaketReferences, PnpmYaml, PubspecOverridesYaml, PubspecYaml, PythonPipfile,
    PythonPyprojectToml, PythonRequirementsTxt, RDescription, RebarConfig, RenvLock, RubyGemspec,
    SbtBuild, StackYaml, SwiftPackage, TerraformTf, UnityProjectManifestJson, Unknown, VcpkgJson,
    VersionLensMultiRegistries, XmakeLua, ZigBuildZon,
};

pub(super) struct DependencySuggestionsRequest<'a> {
    pub(super) input: DocumentInput,
    pub(super) selector: &'a str,
    pub(super) responses: &'a [RegistryResponseInput],
    pub(super) project_bump: Option<ProjectVersionBump>,
    pub(super) operation: &'a OperationContext,
}

impl VersionLensSession {
    pub fn analyze_document(&self, input: DocumentInput) -> AnalyzeDocumentOutput {
        self.analyze_document_with_responses(input, &[])
    }

    pub fn analyze_document_with_responses(
        &self,
        input: DocumentInput,
        responses: &[RegistryResponseInput],
    ) -> AnalyzeDocumentOutput {
        let manifest_kind = self.classify_document(&input);
        let is_supported_manifest = self.manifest_enabled(manifest_kind);
        let active_provider_name = is_supported_manifest
            .then(|| active_provider_name_for_manifest(manifest_kind))
            .flatten()
            .map(ToOwned::to_owned);
        if manifest_kind == VersionLensMultiRegistries {
            return schema_output(&input);
        }

        let dependencies = self.dependencies(&input);
        let suggestions = dependencies
            .iter()
            .map(|dependency| self.cached_suggestion(dependency))
            .collect::<Vec<_>>();
        let code_lenses = dependencies
            .iter()
            .flat_map(|dependency| self.code_lenses_for_dependency(dependency))
            .collect();
        let mut diagnostic_ranges: HashSet<(u32, u32, u32, u32)> = crate::default();
        let diagnostics: Vec<DiagnosticPayload> = dependencies
            .iter()
            .filter(|dependency| {
                let range = dependency.requirement_range;
                diagnostic_ranges.insert((
                    range.start.line,
                    range.start.character,
                    range.end.line,
                    range.end.character,
                ))
            })
            .flat_map(|dependency| self.diagnostics_for_dependency(dependency, responses))
            .collect();

        let status = status_payload(
            dependencies.len(),
            &diagnostics,
            &suggestions,
            self.config.show_suggestion_stats,
        );
        let install_task_config_key = (!dependencies.is_empty())
            .then(|| install_task_config_key_for_manifest(manifest_kind))
            .flatten();
        let dependency_signature = dependency_signature(&dependencies);
        let can_sort_dependencies = can_sort_dependencies(&dependencies);
        let dependency_payloads = into_dependency_payloads(dependencies);

        AnalyzeDocumentOutput {
            can_sort_dependencies,
            is_supported_manifest,
            active_provider_name,
            install_task_config_key,
            dependency_signature,
            dependencies: dependency_payloads,
            code_lenses,
            diagnostics,
            status,
        }
    }

    fn manifest_enabled(&self, manifest_kind: ManifestKind) -> bool {
        match ecosystem_for_manifest(manifest_kind) {
            Some(ecosystem) => self.provider_enabled_for_manifest(manifest_kind, ecosystem),
            None => manifest_kind == VersionLensMultiRegistries,
        }
    }

    pub fn resolve_document(&self, input: DocumentInput) -> ResolveDocumentOutput {
        self.resolve_document_with_responses(input, &[])
    }

    pub fn resolve_document_with_responses(
        &self,
        input: DocumentInput,
        responses: &[RegistryResponseInput],
    ) -> ResolveDocumentOutput {
        let operation = OperationContext::with_timeout(crate::duration_from_millis(
            self.config.http.timeout_ms,
        ));
        let manifest_kind = self.classify_document(&input);
        let suggestions = self.resolve_suggestions(input, responses, None, &operation);
        let edits = update_edits(&suggestions);
        let authorization_required_count = Self::authorization_required_count(&suggestions);
        let vulnerable_update_count =
            self.vulnerable_update_count(&suggestions, responses, Some(manifest_kind), &operation);
        let authorization_required_requests = operation.take_authorization_requests();
        let authorization_required_count =
            authorization_required_count.max(to_u32(authorization_required_requests.len()));
        let suggestion_payloads = into_suggestion_payloads(suggestions);
        ResolveDocumentOutput {
            suggestions: suggestion_payloads,
            edits,
            authorization_required_count,
            authorization_required_requests,
            vulnerable_update_count,
            vulnerable_update_package: None,
            vulnerable_update_version: None,
        }
    }

    pub(crate) fn resolve_suggestions(
        &self,
        input: DocumentInput,
        responses: &[RegistryResponseInput],
        project_bump: Option<ProjectVersionBump>,
        operation: &OperationContext,
    ) -> Vec<Suggestion> {
        let manifest_kind = self.classify_document(&input);
        let context = registry::registry_context_from_document_kind(&input, manifest_kind);
        let suggestions = self.resolve_dependencies(ResolutionRequest {
            dependencies: self.dependencies(&input),
            document_uri: &input.uri,
            responses,
            project_bump,
            context: &context,
            operation,
        });
        if project_bump.is_none() {
            self.cache_resolved_suggestions(&suggestions, context.manifest_kind());
        }
        suggestions
    }

    pub(super) fn resolve_dependency_suggestions(
        &self,
        request: DependencySuggestionsRequest<'_>,
    ) -> Vec<Suggestion> {
        let DependencySuggestionsRequest {
            input,
            selector,
            responses,
            project_bump,
            operation,
        } = request;
        let manifest_kind = self.classify_document(&input);
        let context = registry::registry_context_from_document_kind(&input, manifest_kind);
        let dependencies = self
            .dependencies(&input)
            .into_iter()
            .filter(|dependency| matches_dependency(dependency, selector))
            .collect();
        self.resolve_dependencies(ResolutionRequest {
            dependencies,
            document_uri: &input.uri,
            responses,
            project_bump,
            context: &context,
            operation,
        })
    }
}

#[cfg(test)]
mod tests;

fn active_provider_name_for_manifest(manifest_kind: ManifestKind) -> Option<&'static str> {
    match manifest_kind {
        CargoToml => Some("cargo"),
        ComposerJson => Some("composer"),
        DenoJson | DenoImportMapJson | JsrJson => Some("deno"),
        DotnetProjectJson | DotnetXml | PaketDependencies | PaketReferences => Some("dotnet"),
        DockerComposeYaml | Dockerfile => Some("docker"),
        KustomizationYaml => Some("kustomize"),
        DubJson | DubSdl => Some("dub"),
        Gemfile | RubyGemspec => Some("ruby"),
        GoMod => Some("golang"),
        MavenPomXml
        | GradleBuild
        | GradleSettings
        | GradleVersionCatalogToml
        | SbtBuild
        | ClojureDepsEdn
        | LeiningenProjectClj => Some("maven"),
        MixExs | RebarConfig | GleamToml => Some("hex"),
        Opam | DuneProject => Some("opam"),
        Cabal | CabalProject | StackYaml => Some("hackage"),
        JuliaProjectToml | JuliaManifestToml => Some("julia"),
        RDescription | RenvLock => Some("cran"),
        ConanfileTxt | ConanfilePy => Some("conan"),
        VcpkgJson => Some("vcpkg"),
        Cmake | XmakeLua | MesonWrap | BazelWorkspace => Some("cpp"),
        SwiftPackage => Some("swift"),
        ZigBuildZon => Some("zig"),
        Nimble => Some("nim"),
        LuaRockspec => Some("luarocks"),
        Cpanfile => Some("cpan"),
        HaxelibJson => Some("haxelib"),
        TerraformTf => Some("terraform"),
        HelmChartYaml => Some("helm"),
        AnsibleGalaxyRequirementsYaml => Some("ansible"),
        BazelModule => Some("bazel"),
        NixFlake => Some("nix"),
        UnityProjectManifestJson => Some("unity"),
        CocoaPodsPodfile => Some("cocoapods"),
        NpmPackageJson | NpmPackageJson5 | NpmPackageYaml => Some("npm"),
        PnpmYaml => Some("pnpm"),
        PubspecOverridesYaml | PubspecYaml => Some("pub"),
        PythonPipfile | PythonPyprojectToml | PythonRequirementsTxt => Some("pypi"),
        Unknown | VersionLensMultiRegistries => None,
    }
}
