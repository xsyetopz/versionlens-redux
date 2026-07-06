import { expect, mock, test } from "bun:test";

let activeTextEditor: { document: unknown } | undefined;
let updateContextCount = 0;
let updateContextsResult = false;
let analyzeDocumentResult: { isSupportedManifest: boolean } | undefined;
const refreshedDocuments: unknown[] = [];
const textDocumentChangeListeners: ((event: {
	contentChanges?: unknown[];
	document: unknown;
	reason?: number;
}) => Promise<void> | void)[] = [];
const textDocumentCloseListeners: ((document: { uri: unknown }) => void)[] = [];
const activeEditorChangeListeners: ((
	editor: { document: unknown } | undefined,
) => Promise<void> | void)[] = [];
const createdWatcherPatterns: unknown[] = [];

mock.module("vscode", () => ({
	TextDocumentChangeReason: { Redo: 2, Undo: 1 },
	window: {
		get activeTextEditor() {
			return activeTextEditor;
		},
		onDidChangeActiveTextEditor(
			listener: (
				editor: { document: unknown } | undefined,
			) => Promise<void> | void,
		) {
			activeEditorChangeListeners.push(listener);
			return { dispose: () => undefined };
		},
	},
	workspace: {
		workspaceFolders: [{ uri: { fsPath: "/workspace" } }],
		createFileSystemWatcher(pattern: unknown) {
			createdWatcherPatterns.push(pattern);
			return {
				dispose: () => undefined,
				onDidChange: () => ({ dispose: () => undefined }),
				onDidCreate: () => ({ dispose: () => undefined }),
				onDidDelete: () => ({ dispose: () => undefined }),
			};
		},
		getWorkspaceFolder: () => ({ uri: { fsPath: "/workspace" } }),
		getConfiguration() {
			return {
				get(key: string, fallback?: unknown) {
					return key === "npm.files" ? "**/package.json" : fallback;
				},
			};
		},
		onDidChangeTextDocument(
			listener: (event: {
				contentChanges?: unknown[];
				document: unknown;
				reason?: number;
			}) => Promise<void> | void,
		) {
			textDocumentChangeListeners.push(listener);
			return { dispose: () => undefined };
		},
		onDidSaveTextDocument: () => ({ dispose: () => undefined }),
		onDidCloseTextDocument(listener: (document: { uri: unknown }) => void) {
			textDocumentCloseListeners.push(listener);
			return { dispose: () => undefined };
		},
	},
}));

mock.module("../../commands.ts", () => ({
	registerCommands: () => [],
	updateContexts: () => {
		updateContextCount += 1;
		return updateContextsResult;
	},
}));

mock.module("../../diagnostics.ts", () => ({
	analyzeDocument: () => analyzeDocumentResult,
	refreshDiagnostics: (_state: unknown, document: unknown) => {
		refreshedDocuments.push(document);
	},
}));

mock.module("../../tasks.ts", () => ({
	handleDidSaveTextDocument: () => undefined,
}));

mock.module("../../lifecycle/refresh-timer.ts", () => ({
	registerRefreshTimer: () => ({ dispose: () => undefined }),
}));

test("active document edits refresh diagnostics and toolbar contexts", async () => {
	const { registerExtensionSubscriptions } = await import(
		"../../lifecycle/subscriptions.ts"
	);
	const document = { uri: { toString: () => "file:///package.json" } };
	const context = { subscriptions: [] };
	textDocumentChangeListeners.length = 0;
	refreshedDocuments.length = 0;
	updateContextCount = 0;
	updateContextsResult = true;
	analyzeDocumentResult = { isSupportedManifest: true };
	activeTextEditor = { document };

	registerExtensionSubscriptions(
		{
			snapshots: {
				editedDependencies: new Map<string, string>(),
				savedDependencies: new Map<string, string>(),
			},
			ui: {
				diagnostics: { delete: () => undefined },
				outputChannel: {},
			},
		} as never,
		context as never,
	);
	await textDocumentChangeListeners[0]?.({
		contentChanges: [{ text: "changed" }],
		document,
	});

	expect(refreshedDocuments).toEqual([document]);
	expect(updateContextCount).toBe(1);
});

test("unsupported text document changes do not refresh diagnostics", async () => {
	const { registerExtensionSubscriptions } = await import(
		"../../lifecycle/subscriptions.ts"
	);
	const document = { uri: { toString: () => "file:///README.md" } };
	const context = { subscriptions: [] };
	textDocumentChangeListeners.length = 0;
	refreshedDocuments.length = 0;
	updateContextCount = 0;
	analyzeDocumentResult = { isSupportedManifest: false };
	activeTextEditor = { document };

	registerExtensionSubscriptions(
		{
			snapshots: {
				editedDependencies: new Map<string, string>(),
				savedDependencies: new Map<string, string>(),
			},
			ui: {
				diagnostics: { delete: () => undefined },
				outputChannel: {},
			},
		} as never,
		context as never,
	);
	await textDocumentChangeListeners[0]?.({
		contentChanges: [{ text: "changed" }],
		document,
	});

	expect(refreshedDocuments).toEqual([]);
	expect(updateContextCount).toBe(0);
});

test("empty text document changes without undo or redo do not refresh diagnostics", async () => {
	const { registerExtensionSubscriptions } = await import(
		"../../lifecycle/subscriptions.ts"
	);
	const document = { uri: { toString: () => "file:///package.json" } };
	const context = { subscriptions: [] };
	textDocumentChangeListeners.length = 0;
	refreshedDocuments.length = 0;
	updateContextCount = 0;
	analyzeDocumentResult = { isSupportedManifest: true };
	activeTextEditor = { document };

	registerExtensionSubscriptions(
		{
			snapshots: {
				editedDependencies: new Map<string, string>(),
				savedDependencies: new Map<string, string>(),
			},
			ui: {
				diagnostics: { delete: () => undefined },
				outputChannel: {},
			},
		} as never,
		context as never,
	);
	await textDocumentChangeListeners[0]?.({ contentChanges: [], document });

	expect(refreshedDocuments).toEqual([]);
	expect(updateContextCount).toBe(0);
});

test("undo and redo text document changes refresh diagnostics without content changes", async () => {
	const { registerExtensionSubscriptions } = await import(
		"../../lifecycle/subscriptions.ts"
	);
	const document = { uri: { toString: () => "file:///package.json" } };
	const context = { subscriptions: [] };
	textDocumentChangeListeners.length = 0;
	refreshedDocuments.length = 0;
	updateContextCount = 0;
	activeTextEditor = { document };

	registerExtensionSubscriptions(
		{
			snapshots: {
				editedDependencies: new Map<string, string>(),
				savedDependencies: new Map<string, string>(),
			},
			ui: {
				diagnostics: { delete: () => undefined },
				outputChannel: {},
			},
		} as never,
		context as never,
	);
	await textDocumentChangeListeners[0]?.({
		contentChanges: [],
		document,
		reason: 1,
	});
	await textDocumentChangeListeners[0]?.({
		contentChanges: [],
		document,
		reason: 2,
	});

	expect(refreshedDocuments).toEqual([document, document]);
	expect(updateContextCount).toBe(2);
});

test("supported file closes clear edited snapshots without touching diagnostics", async () => {
	const { registerExtensionSubscriptions } = await import(
		"../../lifecycle/subscriptions.ts"
	);
	const uri = { scheme: "file", toString: () => "file:///package.json" };
	const document = { uri };
	const context = { subscriptions: [] };
	const deletedUris: unknown[] = [];
	textDocumentCloseListeners.length = 0;
	analyzeDocumentResult = { isSupportedManifest: true };

	const state = {
		snapshots: {
			editedDependencies: new Map([[uri.toString(), "edited"]]),
			savedDependencies: new Map([[uri.toString(), "saved"]]),
		},
		ui: {
			diagnostics: {
				delete(uriToDelete: unknown) {
					deletedUris.push(uriToDelete);
				},
			},
			outputChannel: {},
		},
	};

	registerExtensionSubscriptions(state as never, context as never);
	textDocumentCloseListeners[0]?.(document);

	expect(state.snapshots.editedDependencies.has(uri.toString())).toBe(false);
	expect(state.snapshots.savedDependencies.get(uri.toString())).toBe("saved");
	expect(deletedUris).toEqual([]);
});

test("non-file closes preserve dependency snapshots", async () => {
	const { registerExtensionSubscriptions } = await import(
		"../../lifecycle/subscriptions.ts"
	);
	const uri = {
		scheme: "versionlens",
		toString: () => "versionlens:/schema.json",
	};
	const document = { uri };
	const context = { subscriptions: [] };
	textDocumentCloseListeners.length = 0;
	analyzeDocumentResult = { isSupportedManifest: true };

	const state = {
		snapshots: {
			editedDependencies: new Map([[uri.toString(), "edited"]]),
			savedDependencies: new Map([[uri.toString(), "saved"]]),
		},
		ui: { diagnostics: { delete: () => undefined }, outputChannel: {} },
	};

	registerExtensionSubscriptions(state as never, context as never);
	textDocumentCloseListeners[0]?.(document);

	expect(state.snapshots.editedDependencies.get(uri.toString())).toBe("edited");
	expect(state.snapshots.savedDependencies.get(uri.toString())).toBe("saved");
});

test("unsupported file closes preserve dependency snapshots", async () => {
	const { registerExtensionSubscriptions } = await import(
		"../../lifecycle/subscriptions.ts"
	);
	const uri = { scheme: "file", toString: () => "file:///README.md" };
	const document = { uri };
	const context = { subscriptions: [] };
	textDocumentCloseListeners.length = 0;
	analyzeDocumentResult = { isSupportedManifest: false };

	const state = {
		snapshots: {
			editedDependencies: new Map([[uri.toString(), "edited"]]),
			savedDependencies: new Map([[uri.toString(), "saved"]]),
		},
		ui: { diagnostics: { delete: () => undefined }, outputChannel: {} },
	};

	registerExtensionSubscriptions(state as never, context as never);
	textDocumentCloseListeners[0]?.(document);

	expect(state.snapshots.editedDependencies.get(uri.toString())).toBe("edited");
	expect(state.snapshots.savedDependencies.get(uri.toString())).toBe("saved");
});

test("empty active editor changes update toolbar contexts without status UI", async () => {
	const { registerExtensionSubscriptions } = await import(
		"../../lifecycle/subscriptions.ts"
	);
	const context = { subscriptions: [] };
	activeEditorChangeListeners.length = 0;
	updateContextCount = 0;
	updateContextsResult = false;

	registerExtensionSubscriptions(
		{
			snapshots: {
				editedDependencies: new Map<string, string>(),
				savedDependencies: new Map<string, string>(),
			},
			ui: {
				diagnostics: { delete: () => undefined },
				outputChannel: {},
			},
		} as never,
		context as never,
	);
	await activeEditorChangeListeners[0]?.(undefined);

	expect(updateContextCount).toBe(1);
});

test("registers package file system watchers with extension subscriptions", async () => {
	const { registerExtensionSubscriptions } = await import(
		"../../lifecycle/subscriptions.ts"
	);
	const context = { subscriptions: [] };
	createdWatcherPatterns.length = 0;

	registerExtensionSubscriptions(
		{
			snapshots: {
				editedDependencies: new Map<string, string>(),
				savedDependencies: new Map<string, string>(),
			},
			ui: {
				diagnostics: { delete: () => undefined },
				outputChannel: {},
			},
		} as never,
		context as never,
	);

	expect(createdWatcherPatterns).toContain("**/package.json");
});

test("non-file active editor changes update contexts without refreshing diagnostics", async () => {
	const { registerExtensionSubscriptions } = await import(
		"../../lifecycle/subscriptions.ts"
	);
	const context = { subscriptions: [] };
	const document = {
		uri: { scheme: "versionlens", toString: () => "versionlens:/schema.json" },
	};
	activeEditorChangeListeners.length = 0;
	refreshedDocuments.length = 0;
	updateContextCount = 0;
	updateContextsResult = false;

	registerExtensionSubscriptions(
		{
			snapshots: {
				editedDependencies: new Map<string, string>(),
				savedDependencies: new Map<string, string>(),
			},
			ui: {
				diagnostics: { delete: () => undefined },
				outputChannel: {},
			},
		} as never,
		context as never,
	);
	await activeEditorChangeListeners[0]?.({ document });

	expect(updateContextCount).toBe(1);
	expect(refreshedDocuments).toEqual([]);
});

test("unsupported file active editor changes update contexts without refreshing diagnostics", async () => {
	const { registerExtensionSubscriptions } = await import(
		"../../lifecycle/subscriptions.ts"
	);
	const context = { subscriptions: [] };
	const document = {
		uri: { scheme: "file", toString: () => "file:///workspace/README.md" },
	};
	activeEditorChangeListeners.length = 0;
	refreshedDocuments.length = 0;
	updateContextCount = 0;
	updateContextsResult = false;

	registerExtensionSubscriptions(
		{
			snapshots: {
				editedDependencies: new Map<string, string>(),
				savedDependencies: new Map<string, string>(),
			},
			ui: {
				diagnostics: { delete: () => undefined },
				outputChannel: {},
			},
		} as never,
		context as never,
	);
	await activeEditorChangeListeners[0]?.({ document });

	expect(updateContextCount).toBe(1);
	expect(refreshedDocuments).toEqual([]);
});

test("supported workspace active editor changes update contexts without refreshing diagnostics", async () => {
	const { registerExtensionSubscriptions } = await import(
		"../../lifecycle/subscriptions.ts"
	);
	const context = { subscriptions: [] };
	const document = {
		uri: {
			scheme: "file",
			toString: () => "file:///workspace/package.json",
		},
	};
	activeEditorChangeListeners.length = 0;
	refreshedDocuments.length = 0;
	updateContextCount = 0;
	updateContextsResult = true;

	registerExtensionSubscriptions(
		{
			snapshots: {
				editedDependencies: new Map<string, string>(),
				savedDependencies: new Map<string, string>(),
			},
			ui: {
				diagnostics: { delete: () => undefined },
				outputChannel: {},
			},
		} as never,
		context as never,
	);
	await activeEditorChangeListeners[0]?.({ document });

	expect(updateContextCount).toBe(1);
	expect(refreshedDocuments).toEqual([]);
});
