import { expect, mock, test } from "bun:test";

let activeTextEditor: { document: TextDocumentStub } | undefined;
let warningChoice: string | undefined;
let reloadedResolveCount = 0;
const createdSessionConfigs: unknown[] = [];
const diagnosticsSets: { diagnostics: unknown[]; uri: unknown }[] = [];
const inputValues: (string | undefined)[] = [];
const inputPrompts: unknown[] = [];
const quickPickValues: unknown[] = [];
const secretValues: Record<string, string | undefined> = {};
const storedSecrets: { key: string; value: string }[] = [];
const updatedSettings: { key: string; target: boolean; value: unknown }[] = [];
const warningMessages: unknown[] = [];
const workspaceConfig: Record<string, unknown> = {};
const workspaceValues: Record<string, unknown> = {};
const registryUrl = "https://registry.example.test";
const authorizationSecret = `/workspace/.vscode__${registryUrl}`;

type TextDocumentStub = {
	getText: () => string;
	languageId: string;
	uri: {
		toString: () => string;
	};
};

mock.module("vscode", () => ({
	Diagnostic: class {
		message: string;
		range: unknown;
		severity: number;

		constructor(range: unknown, message: string, severity: number) {
			this.range = range;
			this.message = message;
			this.severity = severity;
		}
	},
	Range: class {
		values: number[];

		constructor(...values: number[]) {
			this.values = values;
		}
	},
	Uri: {
		parse: (value: string) => ({ scheme: value.split(":")[0], value }),
	},
	commands: {
		executeCommand: () => undefined,
	},
	window: {
		get activeTextEditor() {
			return activeTextEditor;
		},
		showInputBox: (options: unknown) => {
			inputPrompts.push(options);
			return inputValues.shift();
		},
		showQuickPick: () => quickPickValues.shift(),
		showWarningMessage: (...args: unknown[]) => {
			warningMessages.push(args);
			return warningChoice;
		},
	},
	workspace: {
		getConfiguration() {
			return {
				get(key: string, fallback?: unknown) {
					return Object.hasOwn(workspaceConfig, key)
						? workspaceConfig[key]
						: fallback;
				},
				inspect(key: string) {
					const value = workspaceConfig[key];
					return value === undefined ? undefined : { workspaceValue: value };
				},
				update(key: string, value: unknown, target: boolean) {
					workspaceConfig[key] = value;
					updatedSettings.push({ key, target, value });
				},
			};
		},
		getWorkspaceFolder: () => undefined,
	},
}));

mock.module("./native/module.ts", () => ({
	loadNative() {
		return {
			createSession(config: unknown) {
				createdSessionConfigs.push(config);
				return {
					analyzeDocument: () => outputFor("file:///workspace/package.json"),
					applyCommand: () => undefined,
					clearCache: () => undefined,
					disposeSession: () => undefined,
					resolveDocument: () => {
						reloadedResolveCount += 1;
						return {
							authorizationRequiredCount: 0,
							edits: [],
							suggestions: [],
							vulnerableUpdateCount: 0,
						};
					},
				};
			},
		};
	},
}));

test("document refresh renders diagnostics without status bar side effects", async () => {
	const { refreshDiagnostics } = await import("./diagnostics.ts");
	reset();
	const active = documentStub("file:///workspace/package.json");
	const background = documentStub("file:///workspace/other/package.json");
	activeTextEditor = { document: active };

	await refreshDiagnostics(state() as never, active as never);
	await refreshDiagnostics(state() as never, background as never);

	expect(diagnosticsSets.map((entry) => entry.uri)).toEqual([
		active.uri,
		background.uri,
	]);
});

test("dirty diagnostic refresh marks documents outdated when dependencies changed", async () => {
	const { refreshDiagnostics } = await import("./diagnostics.ts");
	reset();
	const document = {
		...documentStub("file:///workspace/package.json"),
		isDirty: true,
	};
	const currentState = state();
	currentState.snapshots.savedDependencies.set(
		document.uri.toString(),
		"previous-signature",
	);

	await refreshDiagnostics(currentState as never, document as never);

	expect(
		currentState.snapshots.editedDependencies.get(document.uri.toString()),
	).toBe(document.uri.toString());
	expect(currentState.flags.showOutdated).toBe(true);
});

test("dirty diagnostic refresh without saved baseline marks non-empty dependencies outdated", async () => {
	const { refreshDiagnostics } = await import("./diagnostics.ts");
	reset();
	const document = {
		...documentStub("file:///workspace/package.json"),
		isDirty: true,
	};
	const currentState = state();

	await refreshDiagnostics(currentState as never, document as never);

	expect(
		currentState.snapshots.editedDependencies.get(document.uri.toString()),
	).toBe(document.uri.toString());
	expect(currentState.flags.showOutdated).toBe(true);
});

test("diagnostic refresh is gated by visible version lenses", async () => {
	const { refreshDiagnostics } = await import("./diagnostics.ts");
	reset();
	const document = documentStub("file:///workspace/package.json");
	let analyzeCount = 0;

	await refreshDiagnostics(
		state({
			flags: {
				codeLensReplace: true,
				providerBusy: 0,
				providerError: false,
				showPrereleases: false,
				showSuggestionStats: false,
				showVersionLenses: false,
			},
			session: {
				analyzeDocument() {
					analyzeCount += 1;
					return outputFor(document.uri.toString());
				},
				resolveDocument: () => ({
					authorizationRequiredCount: 0,
					edits: [],
					suggestions: [],
					vulnerableUpdateCount: 0,
				}),
			},
		}) as never,
		document as never,
	);

	expect(analyzeCount).toBe(0);
	expect(diagnosticsSets).toEqual([]);
});

test("diagnostic refresh suppresses repeated auth prompts after cancellation", async () => {
	const { refreshDiagnostics } = await import("./diagnostics.ts");
	reset();
	workspaceValues["UrlAuthenticationStore"] = {
		[registryUrl]: {
			protocol: "https:",
			scheme: "NotSet",
			status: "User cancelled",
			url: registryUrl,
		},
	};
	const document = documentStub("file:///workspace/package.json");
	let resolveCount = 0;
	await refreshDiagnostics(
		state({
			context: authContext(),
			session: {
				analyzeDocument: () => outputFor(document.uri.toString()),
				resolveDocument: () => {
					resolveCount += 1;
					return {
						authorizationRequiredCount: 1,
						authorizationRequiredRequests: [
							{
								authUrl: registryUrl,
								requestUrl: `${registryUrl}/private-package`,
							},
						],
						edits: [],
						suggestions: [],
						vulnerableUpdateCount: 0,
					};
				},
			},
		}) as never,
		document as never,
	);

	expect(resolveCount).toBe(1);
	expect(warningMessages).toEqual([]);
	expect(inputPrompts).toEqual([]);
	expect(updatedSettings).toEqual([]);
});

test("diagnostic refresh offers authentication when registry auth is required", async () => {
	const { refreshDiagnostics } = await import("./diagnostics.ts");
	reset();
	warningChoice = "Add Authentication";
	inputValues.push(registryUrl, "Bearer token");
	quickPickValues.push({ label: "Custom Value", providerScheme: "Custom" });
	let initialResolveCount = 0;
	const document = documentStub("file:///workspace/package.json");
	await refreshDiagnostics(
		state({
			context: authContext(),
			session: {
				analyzeDocument: () => outputFor(document.uri.toString()),
				disposeSession: () => undefined,
				resolveDocument: () => {
					initialResolveCount += 1;
					return {
						authorizationRequiredCount: 1,
						authorizationRequiredRequests: [
							{
								authUrl: registryUrl,
								requestUrl: `${registryUrl}/private-package`,
							},
						],
						edits: [],
						suggestions: [],
						vulnerableUpdateCount: 0,
					};
				},
			},
		}) as never,
		document as never,
	);

	expect(warningMessages).toHaveLength(1);
	expect(inputPrompts[0]).toMatchObject({ value: registryUrl });
	expect(storedSecrets).toEqual([
		{
			key: authorizationSecret,
			value: "Bearer token",
		},
	]);
	expect(updatedSettings[0]).toMatchObject({
		key: "UrlAuthenticationStore",
		target: false,
		value: {
			[registryUrl]: {
				label: "Custom Value",
				protocol: "https:",
				scheme: "Custom",
				status: "NoStatus",
				url: registryUrl,
			},
		},
	});
	expect(createdSessionConfigs[0]).toMatchObject({
		http: {
			authHeaders: [
				{
					name: "Authorization",
					url: registryUrl,
					value: "Bearer token",
				},
			],
		},
	});
	expect(initialResolveCount).toBe(1);
	expect(reloadedResolveCount).toBe(1);
});

test("analyze failure decrements only its provider busy operation", async () => {
	const { analyzeDocument } = await import("./diagnostics/analyze.ts");
	reset();
	const document = documentStub("file:///workspace/package.json");
	const currentState = state({
		flags: {
			codeLensReplace: true,
			providerBusy: 2,
			providerError: false,
			showOutdated: false,
			showPrereleases: false,
			showSuggestionStats: false,
			showVersionLenses: true,
		},
		session: {
			analyzeDocument() {
				throw new Error("provider failed");
			},
		},
	});

	analyzeDocument(currentState as never, document as never);

	expect(currentState.flags.providerBusy).toBe(2);
	expect(currentState.flags.providerError).toBe(true);
});

test("resolve failure decrements only its provider busy operation", async () => {
	const { resolveDocumentForDiagnostics } = await import(
		"./diagnostics/resolve.ts"
	);
	reset();
	const document = documentStub("file:///workspace/package.json");
	const currentState = state({
		flags: {
			codeLensReplace: true,
			providerBusy: 2,
			providerError: false,
			showOutdated: false,
			showPrereleases: false,
			showSuggestionStats: false,
			showVersionLenses: true,
		},
		session: {
			resolveDocument() {
				throw new Error("provider failed");
			},
		},
	});

	await resolveDocumentForDiagnostics(currentState as never, document as never);

	expect(currentState.flags.providerBusy).toBe(2);
	expect(currentState.flags.providerError).toBe(true);
});

function authContext() {
	return {
		extensionPath: "/test/extension",
		secrets: {
			get: (key: string) => secretValues[key],
			store(key: string, value: string) {
				secretValues[key] = value;
				storedSecrets.push({ key, value });
			},
		},
		storageUri: { path: "/workspace/.vscode" },
		workspaceState: {
			get: (key: string, fallback: unknown) => workspaceValues[key] ?? fallback,
			update: (key: string, value: unknown) => {
				workspaceValues[key] = value;
				updatedSettings.push({ key, target: false, value });
			},
		},
	};
}

function state(extra: Record<string, unknown> = {}) {
	return {
		context: undefined,
		flags: {
			codeLensReplace: true,
			providerBusy: 0,
			providerError: false,
			showOutdated: false,
			showPrereleases: false,
			showSuggestionStats: false,
			showVersionLenses: true,
		},
		session: {
			analyzeDocument(input: { uri: string }) {
				return outputFor(input.uri);
			},
			resolveDocument: () => ({
				authorizationRequiredCount: 0,
				edits: [],
				suggestions: [],
				vulnerableUpdateCount: 0,
			}),
		},
		snapshots: {
			editedDependencies: new Map<string, string>(),
			savedDependencies: new Map<string, string>(),
		},
		ui: {
			diagnostics: {
				set(uri: unknown, diagnostics: unknown[]) {
					diagnosticsSets.push({ diagnostics, uri });
				},
			},
			outputChannel: { appendLine: () => undefined },
		},
		...extra,
	};
}

function documentStub(uri: string): TextDocumentStub {
	return {
		getText: () => "{}",
		languageId: "json",
		uri: {
			toString: () => uri,
		},
	};
}

function reset() {
	activeTextEditor = undefined;
	createdSessionConfigs.length = 0;
	diagnosticsSets.length = 0;
	inputValues.length = 0;
	inputPrompts.length = 0;
	quickPickValues.length = 0;
	reloadedResolveCount = 0;
	storedSecrets.length = 0;
	updatedSettings.length = 0;
	warningChoice = undefined;
	warningMessages.length = 0;
	for (const key of Object.keys(workspaceConfig)) {
		delete workspaceConfig[key];
	}
	for (const key of Object.keys(workspaceValues)) {
		delete workspaceValues[key];
	}
	for (const key of Object.keys(secretValues)) {
		delete secretValues[key];
	}
}

function outputFor(uri: string) {
	const isActive = uri === "file:///workspace/package.json";
	return {
		canSortDependencies: false,
		codeLenses: [],
		dependencies: [],
		dependencySignature: uri,
		diagnostics: [],
		installTaskConfigKey: undefined,
		isSupportedManifest: true,
		status: {
			dependencyCount: 1,
			errorCount: 0,
			noMatchCount: 0,
			text: isActive ? "active status" : "background status",
			tooltip: isActive ? "active tooltip" : "background tooltip",
			updateCount: 1,
			visible: true,
			vulnerabilityCount: 0,
		},
	};
}
