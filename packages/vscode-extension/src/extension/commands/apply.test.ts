import { expect, mock, test } from "bun:test";

let activeTextEditor: { document: unknown } | undefined;
let applyEditBlocker: Promise<void> | undefined;
let releaseApplyEdit: (() => void) | undefined;
let warningChoice: string | undefined;
let codeLensRefreshCount = 0;
const appliedEdits: unknown[] = [];
const createdSessionConfigs: unknown[] = [];
const createdNativeSessions: unknown[] = [];
const inputValues: string[] = [];
const inputPrompts: unknown[] = [];
const outputLines: string[] = [];
const quickPickValues: unknown[] = [];
const registeredCommands: Record<string, (...args: unknown[]) => unknown> = {};
const storedSecrets: { key: string; value: string }[] = [];
const updatedConfig: { key: string; target: boolean; value: unknown }[] = [];
const warningMessages: unknown[] = [];
const workspaceConfig: Record<string, unknown> = {};
const workspaceValues: Record<string, unknown> = {};
const secretValues: Record<string, string | undefined> = {};
const registryUrl = "https://registry.example.test";
const authorizationSecret = `/workspace/.vscode__${registryUrl}`;

mock.module("vscode", () => ({
	EventEmitter: class {
		event = () => ({ dispose: () => undefined });
		dispose() {
			return undefined;
		}
		fire() {
			codeLensRefreshCount += 1;
		}
	},
	Range: class {},
	Uri: {
		file: (path: string) => ({ path, scheme: "file" }),
	},
	WorkspaceEdit: class {
		edits: unknown[] = [];

		replace(uri: unknown, range: unknown, newText: string) {
			this.edits.push({ newText, range, uri });
		}
	},
	commands: {
		executeCommand: () => undefined,
		registerCommand(
			command: string,
			callback: (...args: unknown[]) => unknown,
		) {
			registeredCommands[command] = callback;
			return { dispose: () => undefined };
		},
	},
	env: {
		openExternal: () => undefined,
	},
	languages: {
		registerCodeLensProvider: () => ({ dispose: () => undefined }),
	},
	tasks: {
		executeTask: () => undefined,
		fetchTasks: () => [],
	},
	window: {
		get activeTextEditor() {
			return activeTextEditor;
		},
		onDidChangeActiveTextEditor: () => ({ dispose: () => undefined }),
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
		applyEdit(edit: { edits: unknown[] }) {
			appliedEdits.push(...edit.edits);
			return applyEditBlocker;
		},
		asRelativePath: (uri: { toString: () => string }) => uri.toString(),
		findFiles: () => [],
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
					updatedConfig.push({ key, target, value });
				},
			};
		},
		getWorkspaceFolder: () => undefined,
		onDidChangeConfiguration: () => ({ dispose: () => undefined }),
		onDidChangeTextDocument: () => ({ dispose: () => undefined }),
		onDidCloseTextDocument: () => ({ dispose: () => undefined }),
		onDidSaveTextDocument: () => ({ dispose: () => undefined }),
		openTextDocument: () => undefined,
	},
}));

mock.module("../diagnostics.ts", () => ({
	analyzeDocument: () => undefined,
	dependencySnapshot: () => "",
	refreshActiveDiagnostics: () => undefined,
	refreshDiagnostics: () => undefined,
	setProviderState: () => undefined,
}));

mock.module("../native/module.ts", () => ({
	loadNative() {
		return {
			createSession(config: unknown) {
				createdSessionConfigs.push(config);
				return (
					createdNativeSessions.shift() ?? {
						analyzeDocument: () => undefined,
						applyCommand: () => undefined,
						clearCache: () => undefined,
						disposeSession: () => undefined,
						resolveDocument: () => undefined,
					}
				);
			},
		};
	},
}));

test("resolve command offers authentication when registry auth is required", async () => {
	const { registerCommands } = await import("../commands.ts");
	reset();
	warningChoice = "Add Authentication";
	inputValues.push(registryUrl, "Bearer token");
	quickPickValues.push({ label: "Custom Value", providerScheme: "Custom" });
	const document = documentStub("private-package");
	const session = {
		disposeSession: () => undefined,
		applyCommand: () => ({
			authorizationRequiredCount: 1,
			authorizationRequiredRequests: [
				{
					authUrl: registryUrl,
					requestUrl: `${registryUrl}/private-package`,
				},
			],
			edits: [],
			vulnerableUpdateCount: 0,
		}),
	};

	activeTextEditor = { document };
	registerCommands(
		commandState(session, {
			context: authContext(),
		}) as never,
	);
	await registeredCommands["versionlens.suggestion.onUpdateDependency"]?.(
		"private-package",
	);

	expect(warningMessages).toHaveLength(1);
	expect(inputPrompts[0]).toMatchObject({ value: registryUrl });
	expect(storedSecrets).toEqual([
		{
			key: authorizationSecret,
			value: "Bearer token",
		},
	]);
	expect(updatedConfig[0]).toMatchObject({
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
	expect(appliedEdits).toEqual([]);
});

test("resolve command retries and applies edits after adding authentication", async () => {
	const { registerCommands } = await import("../commands.ts");
	reset();
	warningChoice = "Add Authentication";
	inputValues.push(registryUrl, "Bearer token");
	quickPickValues.push({ label: "Custom Value", providerScheme: "Custom" });
	const document = documentStub("private-package");
	const applyInputs: unknown[] = [];
	const session = {
		applyCommand: (input: unknown) => {
			applyInputs.push(input);
			return {
				authorizationRequiredCount: 1,
				edits: [],
				vulnerableUpdateCount: 0,
			};
		},
		disposeSession: () => undefined,
	};
	createdNativeSessions.push({
		analyzeDocument: () => undefined,
		applyCommand: (input: unknown) => {
			applyInputs.push(input);
			return {
				authorizationRequiredCount: 0,
				edits: [
					{
						newText: "1.1.0",
						range: {
							end: { character: 42, line: 0 },
							start: { character: 37, line: 0 },
						},
					},
				],
				vulnerableUpdateCount: 0,
			};
		},
		clearCache: () => undefined,
		disposeSession: () => undefined,
		resolveDocument: () => undefined,
	});

	activeTextEditor = { document };
	registerCommands(
		commandState(session, {
			context: authContext(),
		}) as never,
	);
	await registeredCommands["versionlens.suggestion.onUpdateDependency"]?.(
		"private-package",
	);

	expect(applyInputs).toHaveLength(2);
	expect(applyInputs[1]).toEqual(applyInputs[0]);
	expect(appliedEdits).toEqual([expect.objectContaining({ newText: "1.1.0" })]);
});

test("add auth command reloads the native session with stored headers", async () => {
	const { registerCommands } = await import("../commands.ts");
	reset();
	inputValues.push(registryUrl, "secret-token");
	quickPickValues.push({ label: "Custom Value", providerScheme: "Custom" });
	let clearCacheCount = 0;

	registerCommands(
		commandState(
			{
				clearCache: () => (clearCacheCount += 1),
				disposeSession: () => undefined,
			},
			{
				context: authContext(),
			},
		) as never,
	);
	await registeredCommands["versionlens.editor.onAddUrlAuthentication"]?.();

	expect(storedSecrets).toEqual([
		{
			key: authorizationSecret,
			value: "secret-token",
		},
	]);
	expect(createdSessionConfigs[0]).toMatchObject({
		http: {
			authHeaders: [
				{
					name: "Authorization",
					url: registryUrl,
					value: "secret-token",
				},
			],
		},
	});
	expect(clearCacheCount).toBe(1);
	expect(codeLensRefreshCount).toBe(1);
});

test("sort command bypasses CodeLens replacement gate like upstream", async () => {
	const { registerCommands } = await import("../commands.ts");
	reset();
	const applyInputs: unknown[] = [];
	const session = {
		applyCommand: (input: unknown) => {
			applyInputs.push(input);
			return {
				authorizationRequiredCount: 0,
				edits: [
					{
						newText: '"a":"1.0.0",\n"b":"1.0.0"',
						range: {
							end: { character: 41, line: 0 },
							start: { character: 17, line: 0 },
						},
					},
				],
				vulnerableUpdateCount: 0,
			};
		},
	};

	const state = commandState(session, {
		flags: {
			providerBusy: 0,
			providerError: false,
			codeLensReplace: false,
			showPrereleases: false,
			showSuggestionStats: false,
			showVersionLenses: true,
		},
	});
	activeTextEditor = { document: documentStub("b") };
	registerCommands(state as never);
	await registeredCommands["versionlens.editor.onSortDependencies"]?.();

	expect(applyInputs[0]).toMatchObject({ command: "sort" });
	expect(appliedEdits).toEqual([
		expect.objectContaining({ newText: expect.stringContaining('"a"') }),
	]);
	expect(state.flags.codeLensReplace).toBe(false);
});

test("single update leaves CodeLens replacement disabled after applying like upstream", async () => {
	const { registerCommands } = await import("../commands.ts");
	reset();
	const document = documentStub("left-pad");
	const state = commandState({
		applyCommand: () => ({
			authorizationRequiredCount: 0,
			edits: [
				{
					newText: "1.1.0",
					range: {
						end: { character: 35, line: 0 },
						start: { character: 30, line: 0 },
					},
				},
			],
			vulnerableUpdateCount: 0,
		}),
	});

	activeTextEditor = { document };
	registerCommands(state as never);
	await registeredCommands["versionlens.suggestion.onUpdateDependency"]?.(
		"left-pad",
	);

	expect(appliedEdits).toHaveLength(1);
	expect(state.flags.codeLensReplace).toBe(false);
});

test("bulk update leaves CodeLens replacement disabled when applyEdit rejects like upstream", async () => {
	const { registerCommands } = await import("../commands.ts");
	reset();
	const document = documentStub("left-pad");
	applyEditBlocker = Promise.reject(new Error("apply failed"));
	const state = commandState({
		applyCommand: () => ({
			authorizationRequiredCount: 0,
			edits: [
				{
					newText: "1.1.0",
					range: {
						end: { character: 35, line: 0 },
						start: { character: 30, line: 0 },
					},
				},
			],
			vulnerableUpdateCount: 0,
		}),
	});

	activeTextEditor = { document };
	registerCommands(state as never);
	await expect(
		registeredCommands["versionlens.editor.onUpdateDependenciesLatest"]?.(),
	).rejects.toThrow("apply failed");

	expect(state.flags.codeLensReplace).toBe(false);
});

test("resolve command ignores reentry while an edit is pending", async () => {
	const { registerCommands } = await import("../commands.ts");
	reset();
	const document = documentStub("left-pad");
	const applyInputs: unknown[] = [];
	applyEditBlocker = new Promise((resolve) => {
		releaseApplyEdit = resolve;
	});
	const session = {
		applyCommand: (input: unknown) => {
			applyInputs.push(input);
			return {
				authorizationRequiredCount: 0,
				edits: [
					{
						newText: "1.1.0",
						range: {
							end: { character: 35, line: 0 },
							start: { character: 30, line: 0 },
						},
					},
				],
				vulnerableUpdateCount: 0,
			};
		},
	};

	activeTextEditor = { document };
	registerCommands(commandState(session) as never);
	const first =
		registeredCommands["versionlens.suggestion.onUpdateDependency"]?.(
			"left-pad",
		);
	await Promise.resolve();
	const second =
		registeredCommands["versionlens.suggestion.onUpdateDependency"]?.(
			"left-pad",
		);
	await Promise.resolve();

	expect(applyInputs).toHaveLength(1);

	releaseApplyEdit?.();
	await first;
	await second;
});

test("resolve command logs native failures without applying edits", async () => {
	const { registerCommands } = await import("../commands.ts");
	reset();
	const consoleError = console.error;
	console.error = () => undefined;
	const document = documentStub("left-pad");
	const session = {
		applyCommand: () => {
			throw new Error("native exploded");
		},
	};

	activeTextEditor = { document };
	registerCommands(
		commandState(session, {
			ui: {
				outputChannel: {
					appendLine(value: string) {
						outputLines.push(value);
					},
				},
			},
		}) as never,
	);
	try {
		await registeredCommands["versionlens.suggestion.onUpdateDependency"]?.(
			"left-pad",
		);

		expect(outputLines[0]).toContain("native exploded");
		expect(appliedEdits).toEqual([]);
	} finally {
		console.error = consoleError;
	}
});

test("resolve command forwards CodeLens-selected update commands", async () => {
	const { registerCommands } = await import("../commands.ts");
	reset();
	const applyInputs: unknown[] = [];
	const selector = "1.2.3\u001f0:12,0:17";
	const session = {
		applyCommand: (input: unknown) => {
			applyInputs.push(input);
			return {
				authorizationRequiredCount: 0,
				edits: [],
				vulnerableUpdateCount: 0,
			};
		},
	};

	activeTextEditor = { document: documentStub("version") };
	registerCommands(commandState(session) as never);
	await registeredCommands["versionlens.suggestion.onUpdateDependency"]?.(
		"1.2.3",
		selector,
		"updateMajor",
		"2.0.0",
	);

	expect(applyInputs[0]).toMatchObject({
		command: "updateMajor",
		dependencyName: selector,
		selectedVersion: "2.0.0",
	});
});

test("bulk update native failure decrements provider busy once like upstream", async () => {
	const { registerCommands } = await import("../commands.ts");
	reset();
	const document = documentStub("left-pad");
	const state = commandState(
		{
			applyCommand() {
				throw new Error("native exploded");
			},
		},
		{
			flags: {
				providerBusy: 2,
				providerError: false,
				codeLensReplace: true,
				showPrereleases: false,
				showSuggestionStats: false,
				showVersionLenses: true,
			},
			ui: {
				codeLensRefresh: undefined,
				diagnostics: undefined,
				outputChannel: { appendLine: () => undefined },
			},
		},
	);

	activeTextEditor = { document };
	registerCommands(state as never);
	await registeredCommands["versionlens.editor.onUpdateDependenciesLatest"]?.();

	expect(state.flags.providerBusy).toBe(2);
	expect(state.flags.providerError).toBe(true);
});

function authContext() {
	return {
		extensionPath: "/test/extension",
		secrets: {
			get: async (key: string) => secretValues[key],
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
				updatedConfig.push({ key, target: false, value });
			},
		},
	};
}

function reset() {
	activeTextEditor = undefined;
	applyEditBlocker = undefined;
	appliedEdits.length = 0;
	codeLensRefreshCount = 0;
	createdSessionConfigs.length = 0;
	createdNativeSessions.length = 0;
	inputValues.length = 0;
	inputPrompts.length = 0;
	outputLines.length = 0;
	quickPickValues.length = 0;
	releaseApplyEdit = undefined;
	storedSecrets.length = 0;
	updatedConfig.length = 0;
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
	for (const command of Object.keys(registeredCommands)) {
		delete registeredCommands[command];
	}
}

function commandState(session: unknown, extra: Record<string, unknown> = {}) {
	return {
		flags: {
			providerBusy: 0,
			providerError: false,
			codeLensReplace: true,
			showPrereleases: false,
			showSuggestionStats: false,
			showVersionLenses: true,
		},
		session,
		ui: {
			codeLensRefresh: undefined,
			diagnostics: undefined,
			outputChannel: undefined,
		},
		...extra,
	};
}

function documentStub(packageName: string) {
	return {
		getText: () => `{"dependencies":{"${packageName}":"1.0.0"}}`,
		languageId: "json",
		uri: { toString: () => "file:///package.json" },
	};
}
