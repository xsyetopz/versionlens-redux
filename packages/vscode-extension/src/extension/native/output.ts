export type AnalyzeDocumentOutput = {
	activeProviderName?: string;
	canSortDependencies: boolean;
	codeLenses: NativeCodeLensPayload[];
	dependencies: NativeDependency[];
	dependencySignature: string;
	diagnostics: NativeDiagnosticPayload[];
	installTaskConfigKey?: string;
	isSupportedManifest: boolean;
	status: NativeStatusPayload;
};

export type NativeAuthorizationRequest = {
	authUrl: string;
	requestUrl: string;
};

export type ResolveDocumentOutput = {
	authorizationRequiredCount: number;
	authorizationRequiredRequests?: NativeAuthorizationRequest[];
	edits: NativeTextEdit[];
	suggestions: NativeSuggestion[];
	vulnerableUpdateCount: number;
	vulnerableUpdatePackage?: string;
	vulnerableUpdateVersion?: string;
};

export type NativeSuggestion = {
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
};

export type NativeDependency = {
	ecosystem: string;
	group: string;
	hostedName?: string;
	hostedUrl?: string;
	name: string;
	range: NativeRange;
	requirement: string;
	requirementRange: NativeRange;
};

export type NativeTextEdit = {
	newText: string;
	range: NativeRange;
};

export type NativeCodeLensPayload = {
	arguments: string[];
	command: string;
	range: NativeRange;
	title: string;
};

export type NativeDiagnosticPayload = {
	code?: string;
	codeDescriptionUrl?: string;
	message: string;
	range: NativeRange;
	severity: number;
	source?: string;
};

export type NativeStatusPayload = {
	dependencyCount: number;
	errorCount: number;
	noMatchCount: number;
	text: string;
	tooltip: string;
	updateCount: number;
	visible: boolean;
	vulnerabilityCount: number;
};

export type NativeRange = {
	end: NativePosition;
	start: NativePosition;
};

export type NativePosition = {
	character: number;
	line: number;
};
