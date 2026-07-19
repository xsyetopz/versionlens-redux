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
  suggestions: NativeSuggestion[];
  vulnerableUpdateCount: number;
  vulnerableUpdatePackage?: string;
  vulnerableUpdateVersion?: string;
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
