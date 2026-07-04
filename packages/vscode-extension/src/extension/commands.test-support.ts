import { mock } from "bun:test";

let activeTextEditor: { document: unknown } | undefined;
let analyzed:
	| {
			canSortDependencies: boolean;
			status?: { text: string; tooltip: string; visible: boolean };
			activeProviderName?: string;
			installTaskConfigKey?: string;
			isSupportedManifest: boolean;
	  }
	| undefined;
let dependencySnapshotValue = "";
let activeRefreshCount = 0;
let refreshCount = 0;
let codeLensRefreshCount = 0;
let warningChoice: string | undefined;
let resolveDiagnosticsHook:
	| ((state: unknown, document: unknown) => unknown)
	| undefined;
let dependencyFileType = 2;
const smokeTaskLabel = "smoke bun install";
const contexts: Record<string, unknown> = {};
const executedTasks: string[] = [];
const openedExternalUris: unknown[] = [];
const shownTextDocuments: unknown[] = [];
const quickPickItems: unknown[] = [];
const quickPickOptions: unknown[] = [];
const warningMessages: unknown[] = [];
const findFilesCalls: { exclude?: unknown; include: unknown }[] = [];
const fileSystemWatchers: { disposed: boolean; pattern: unknown }[] = [];
const workspaceConfig: Record<string, unknown> = {
	"npm.onSaveChanges": smokeTaskLabel,
};
const createdSessionConfigs: unknown[] = [];
const configurationChangeListeners: ((event: {
	affectsConfiguration: (section: string) => boolean;
}) => Promise<void> | void)[] = [];
const taskEndListeners: ((event: {
	execution: { task: { name: string } };
	exitCode: number | undefined;
}) => void)[] = [];
let taskCompletionMode: "auto" | "manual" = "auto";
const testGlobals = globalThis as typeof globalThis & {
	__versionLensAppliedEdits?: unknown[];
	__versionLensRegisteredCommands?: Record<
		string,
		(...args: unknown[]) => unknown
	>;
};
testGlobals.__versionLensRegisteredCommands ??= {};
testGlobals.__versionLensAppliedEdits ??= [];
const registeredCommands = testGlobals.__versionLensRegisteredCommands;
const appliedEdits = testGlobals.__versionLensAppliedEdits;

function clearRegisteredCommands() {
	for (const command of Object.keys(registeredCommands)) {
		delete registeredCommands[command];
	}
}

function commandState(session: unknown, extra: Record<string, unknown> = {}) {
	return {
		flags: {
			providerBusy: 0,
			providerError: false,
			showOutdated: false,
			showPrereleases: false,
			showSuggestionStats: false,
			showVersionLenses: true,
			codeLensReplace: true,
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

mock.module("vscode", () => ({
	RelativePattern: class {
		base: unknown;
		pattern: string;

		constructor(base: unknown, pattern: string) {
			this.base = base;
			this.pattern = pattern;
		}
	},
	CodeLens: class {},
	EventEmitter: class {
		listeners: (() => void)[] = [];
		event = (listener: () => void) => {
			this.listeners.push(listener);
			return {
				dispose: () => {
					const index = this.listeners.indexOf(listener);
					if (index >= 0) {
						this.listeners.splice(index, 1);
					}
				},
			};
		};
		fire() {
			codeLensRefreshCount += 1;
			for (const listener of [...this.listeners]) {
				listener();
			}
		}
		dispose() {
			this.listeners = [];
		}
	},
	FileType: {
		Directory: 2,
		File: 1,
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
		executeCommand(command: string, key: string, value: unknown) {
			if (command === "setContext") {
				contexts[key] = value;
			}
		},
		registerCommand(
			command: string,
			callback: (...args: unknown[]) => unknown,
		) {
			registeredCommands[command] = callback;
			return { dispose: () => undefined };
		},
	},
	languages: {
		registerCodeLensProvider: () => ({ dispose: () => undefined }),
	},
	env: {
		openExternal(uri: unknown) {
			openedExternalUris.push(uri);
		},
	},
	tasks: {
		executeTask(task: { name: string }) {
			executedTasks.push(task.name);
			if (taskCompletionMode === "auto") {
				queueMicrotask(() => completeTask(task.name, 0));
			}
		},
		fetchTasks() {
			return [{ name: smokeTaskLabel }];
		},
		onDidEndTaskProcess(
			listener: (event: {
				execution: { task: { name: string } };
				exitCode: number | undefined;
			}) => void,
		) {
			taskEndListeners.push(listener);
			return {
				dispose() {
					const index = taskEndListeners.indexOf(listener);
					if (index >= 0) {
						taskEndListeners.splice(index, 1);
					}
				},
			};
		},
	},
	window: {
		get activeTextEditor() {
			return activeTextEditor;
		},
		onDidChangeActiveTextEditor: () => ({ dispose: () => undefined }),
		showWarningMessage: (...args: unknown[]) => {
			warningMessages.push(args);
			return warningChoice;
		},
		showInformationMessage: (...args: unknown[]) => {
			warningMessages.push(args);
		},
		showInputBox: () => undefined,
		showQuickPick: (items: unknown[], options?: unknown) => {
			quickPickItems.push(...items);
			quickPickOptions.push(options);
			return items[0];
		},
		showTextDocument: (uri: unknown) => {
			shownTextDocuments.push(uri);
		},
	},
	workspace: {
		createFileSystemWatcher(pattern: unknown) {
			const watcher = { disposed: false, pattern };
			fileSystemWatchers.push(watcher);
			return {
				dispose() {
					watcher.disposed = true;
				},
				onDidChange: () => ({ dispose: () => undefined }),
				onDidCreate: () => ({ dispose: () => undefined }),
				onDidDelete: () => ({ dispose: () => undefined }),
			};
		},
		asRelativePath(uri: { toString: () => string }) {
			return uri.toString().replace("file:///workspace/", "");
		},
		applyEdit(edit: { edits: unknown[] }) {
			appliedEdits.push(...edit.edits);
		},
		findFiles(include: unknown, exclude?: unknown) {
			findFilesCalls.push({ exclude, include });
			return [
				{
					fsPath: "/workspace/package.json",
					toString: () => "file:///workspace/package.json",
				},
			];
		},
		fs: {
			stat: () => ({ type: dependencyFileType }),
		},
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
				update(key: string, value: unknown, _target: boolean) {
					workspaceConfig[key] = value;
				},
			};
		},
		workspaceFolders: [{ uri: { fsPath: "/workspace" } }],
		getWorkspaceFolder: () => ({ uri: { fsPath: "/workspace" } }),
		openTextDocument(uri: unknown) {
			return {
				getText: () => '{"dependencies":{"left-pad":"1.0.0"}}',
				languageId: "json",
				uri,
			};
		},
		onDidChangeConfiguration(
			listener: (event: {
				affectsConfiguration: (section: string) => boolean;
			}) => Promise<void> | void,
		) {
			configurationChangeListeners.push(listener);
			return { dispose: () => undefined };
		},
		onDidChangeTextDocument: () => ({ dispose: () => undefined }),
		onDidSaveTextDocument: () => ({ dispose: () => undefined }),
		onDidCloseTextDocument: () => ({ dispose: () => undefined }),
	},
}));

mock.module("./native/module.ts", () => ({
	loadNative() {
		return {
			createSession(config: unknown) {
				createdSessionConfigs.push(config);
				return {
					analyzeDocument: () => analyzed,
					applyCommand: () => undefined,
					clearCache: () => undefined,
					disposeSession: () => undefined,
					resolveDocument: () => undefined,
				};
			},
		};
	},
}));

function completeTask(name: string, exitCode: number | undefined) {
	for (const listener of [...taskEndListeners]) {
		listener({ execution: { task: { name } }, exitCode });
	}
}

mock.module("./diagnostics.ts", () => ({
	analyzeDocument: () => analyzed,
	dependencySnapshot: () => dependencySnapshotValue,
	refreshActiveDiagnostics: () => {
		activeRefreshCount += 1;
	},
	refreshDiagnostics: () => {
		refreshCount += 1;
	},
	setProviderState: () => undefined,
}));

mock.module("./diagnostics/resolve.ts", () => ({
	resolveDocumentForDiagnostics: (
		state: {
			session?: { resolveDocument?: (input: unknown) => unknown };
		},
		document: unknown,
	) => {
		if (resolveDiagnosticsHook) {
			return resolveDiagnosticsHook(state, document);
		}
		return state.session?.resolveDocument?.(document);
	},
}));

export const testState = {
	get activeTextEditor() {
		return activeTextEditor;
	},
	set activeTextEditor(value) {
		activeTextEditor = value;
	},
	get activeRefreshCount() {
		return activeRefreshCount;
	},
	set activeRefreshCount(value) {
		activeRefreshCount = value;
	},
	get analyzed() {
		return analyzed;
	},
	set analyzed(value) {
		analyzed = value;
	},
	get codeLensRefreshCount() {
		return codeLensRefreshCount;
	},
	set codeLensRefreshCount(value) {
		codeLensRefreshCount = value;
	},
	get dependencyFileType() {
		return dependencyFileType;
	},
	set dependencyFileType(value) {
		dependencyFileType = value;
	},
	get dependencySnapshotValue() {
		return dependencySnapshotValue;
	},
	set dependencySnapshotValue(value) {
		dependencySnapshotValue = value;
	},
	get refreshCount() {
		return refreshCount;
	},
	set refreshCount(value) {
		refreshCount = value;
	},
	get resolveDiagnosticsHook() {
		return resolveDiagnosticsHook;
	},
	set resolveDiagnosticsHook(value) {
		resolveDiagnosticsHook = value;
	},
	get taskCompletionMode() {
		return taskCompletionMode;
	},
	set taskCompletionMode(value) {
		taskCompletionMode = value;
	},
	get warningChoice() {
		return warningChoice;
	},
	set warningChoice(value) {
		warningChoice = value;
	},
};

export {
	appliedEdits,
	clearRegisteredCommands,
	commandState,
	completeTask,
	configurationChangeListeners,
	contexts,
	createdSessionConfigs,
	executedTasks,
	fileSystemWatchers,
	findFilesCalls,
	openedExternalUris,
	quickPickItems,
	quickPickOptions,
	registeredCommands,
	shownTextDocuments,
	smokeTaskLabel,
	warningMessages,
	workspaceConfig,
};
