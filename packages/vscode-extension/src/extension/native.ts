import { createRequire } from "node:module";
import { join } from "node:path";

export interface NativeSessionConfig {
  cacheDurationMinutes?: number;
  enabledProviders?: string[];
  http?: NativeHttpConfig;
  providers?: NativeProviderSettings;
  showPrereleases: boolean;
  showSuggestionStats?: boolean;
  showVulnerabilities?: boolean;
  suggestionIndicators?: NativeSuggestionIndicators;
}

export interface NativeProviderSettings {
  dependencyProperties?: NativeDependencyPropertyConfig[];
  filePatterns?: NativeFilePatternConfig[];
  prereleaseTagFilters?: NativePrereleaseTagFilter[];
  providerCache?: NativeProviderCacheConfig[];
  providerHttp?: NativeProviderHttpConfig[];
  registryUrls?: NativeRegistryUrl[];
}

export interface NativeDependencyPropertyConfig {
  ecosystem: string;
  properties: string[];
  provider?: string;
}

export interface NativeFilePatternConfig {
  ecosystem: string;
  pattern: string;
}

export interface NativeSuggestionIndicators {
  build?: string;
  downgradeable?: string;
  directory?: string;
  error?: string;
  latest?: string;
  matched?: string;
  noMatch?: string;
  satisfiesLatest?: string;
  updateable?: string;
  updateableVulnerable?: string;
}

export interface NativeRegistryUrl {
  ecosystem: string;
  url: string;
}

export interface NativePrereleaseTagFilter {
  ecosystem: string;
  tags: string[];
}

export interface NativeProviderHttpConfig {
  ecosystem: string;
  strictSsl?: boolean;
}

export interface NativeProviderCacheConfig {
  cacheDurationMinutes?: number;
  ecosystem: string;
}

export interface NativeHttpConfig {
  authHeaders?: NativeHttpHeader[];
  ca?: string;
  caFile?: string;
  cert?: string;
  certFile?: string;
  key?: string;
  keyFile?: string;
  proxy?: string;
  strictSsl?: boolean;
  timeoutMs?: number;
}

export interface NativeHttpHeader {
  name: string;
  url?: string;
  value: string;
}

export interface NativeDocumentInput {
  languageId: string;
  text: string;
  uri: string;
  workspaceRoot?: string;
  version?: number;
}

export interface NativeWorkspaceCheckingInput {
  documents: NativeDocumentInput[];
  exclusions: string[];
  providerExclusions?: NativeWorkspaceProviderExclusion[];
  roots: string[];
}

export interface NativeWorkspaceProviderExclusion {
  ecosystem: string;
  patterns: string[];
}

export interface NativeWorkspaceGeneration {
  generation: string;
}

export interface NativeWorkspaceCheckEvent {
  document?: NativeDocumentInput;
  generation: string;
  kind: string;
  message?: string;
  uri?: string;
}

export type NativeApplyCommand =
  | "sort"
  | "update"
  | "updateMajor"
  | "updateMinor"
  | "updatePatch"
  | "updateRelease"
  | "updatePrerelease";

export interface NativeApplyCommandInput {
  command?: NativeApplyCommand;
  dependencyName?: string;
  document: NativeDocumentInput;
  selectedVersion?: string;
}

export interface AnalyzeDocumentOutput {
  activeProviderName?: string;
  canSortDependencies: boolean;
  codeLenses: NativeCodeLensPayload[];
  dependencies: NativeDependency[];
  dependencySignature: string;
  diagnostics: NativeDiagnosticPayload[];
  installTaskConfigKey?: string;
  isSupportedManifest: boolean;
  status: NativeStatusPayload;
}

export interface NativeAuthorizationRequest {
  authUrl: string;
  requestUrl: string;
}

export interface ResolveDocumentOutput {
  authorizationRequiredCount: number;
  authorizationRequiredRequests: NativeAuthorizationRequest[];
  edits: NativeTextEdit[];
  editPlan?: NativeWorkspaceEditPlan;
  suggestions: NativeSuggestion[];
  vulnerableUpdateCount: number;
  vulnerableUpdatePackage?: string;
  vulnerableUpdateVersion?: string;
}

export interface NativeDocumentEditPlan {
  document: NativeDocumentSnapshot;
  edits: NativeTextEdit[];
}

export interface NativeWorkspaceEditPlan {
  documents: NativeDocumentEditPlan[];
}

export interface NativeDocumentSnapshot {
  uri: string;
  version?: number;
  textHash: string;
}

export interface NativeSuggestion {
  builds: string[];
  dependency: NativeDependency;
  latest?: string;
  status:
    | "buildAvailable"
    | "current"
    | "directory"
    | "directoryNotFound"
    | "error"
    | "fixed"
    | "invalid"
    | "invalidRange"
    | "noMatch"
    | "notSupported"
    | "satisfies"
    | "satisfiesLatest"
    | "unresolved"
    | "updateAvailable";
}

export interface NativeDependency {
  ecosystem: string;
  group: string;
  hostedName?: string;
  hostedUrl?: string;
  name: string;
  range: NativeRange;
  requirement: string;
  requirementRange: NativeRange;
}

export interface NativeTextEdit {
  newText: string;
  range: NativeRange;
}

export interface NativeCodeLensPayload {
  arguments: string[];
  command: string;
  range: NativeRange;
  title: string;
}

export interface NativeDiagnosticPayload {
  code?: string;
  codeDescriptionUrl?: string;
  message: string;
  range: NativeRange;
  severity: number;
  source?: string;
}

export interface NativeStatusPayload {
  dependencyCount: number;
  errorCount: number;
  noMatchCount: number;
  text: string;
  tooltip: string;
  updateCount: number;
  visible: boolean;
  vulnerabilityCount: number;
}

export interface NativeRange {
  end: NativePosition;
  start: NativePosition;
}

export interface NativePosition {
  character: number;
  line: number;
}

interface DocumentSession {
  analyzeDocument: (input: NativeDocumentInput) => AnalyzeDocumentOutput;
  applyCommand: (
    input: NativeApplyCommandInput,
  ) => Promise<ResolveDocumentOutput>;
  documentIsFresh: (input: NativeDocumentInput) => boolean;
  resolveDocument: (
    input: NativeDocumentInput,
    background?: boolean,
  ) => Promise<ResolveDocumentOutput>;
}

interface WorkspaceSession {
  checkWorkspace: (
    input: NativeWorkspaceCheckingInput,
  ) => NativeWorkspaceGeneration;
  invalidateWorkspace: () => void;
  setWorkspaceDocuments: (documents: NativeDocumentInput[]) => boolean;
  takeWorkspaceEvents: () => NativeWorkspaceCheckEvent[];
  workspaceCheckingGeneration: () => NativeWorkspaceGeneration | null;
}

interface SessionLifecycle {
  clearCache: () => void;
  disposeSession: () => void;
}

export declare class NativeSession
  implements DocumentSession, WorkspaceSession, SessionLifecycle
{
  analyzeDocument(input: NativeDocumentInput): AnalyzeDocumentOutput;
  applyCommand(input: NativeApplyCommandInput): Promise<ResolveDocumentOutput>;
  checkWorkspace(
    input: NativeWorkspaceCheckingInput,
  ): NativeWorkspaceGeneration;
  clearCache(): void;
  documentIsFresh(input: NativeDocumentInput): boolean;
  disposeSession(): void;
  invalidateWorkspace(): void;
  resolveDocument(
    input: NativeDocumentInput,
    background?: boolean,
  ): Promise<ResolveDocumentOutput>;
  setWorkspaceDocuments(documents: NativeDocumentInput[]): boolean;
  takeWorkspaceEvents(): NativeWorkspaceCheckEvent[];
  workspaceCheckingGeneration(): NativeWorkspaceGeneration | null;
}

export interface NativeModule {
  createSession: (config: NativeSessionConfig) => NativeSession;
  createSessionWithStorage: (
    config: NativeSessionConfig,
    directory: string,
  ) => Promise<NativeSession>;
}

export function loadNative(extensionPath: string): NativeModule {
  return createRequire(join(extensionPath, "dist", "extension.js"))(
    join(extensionPath, "native", "versionlens_napi.node"),
  ) as NativeModule;
}
