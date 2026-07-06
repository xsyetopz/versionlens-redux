import { expect, mock, test } from "bun:test";
import { readFileSync } from "node:fs";

const createdWatchers: {
	pattern: unknown;
	disposeCount: number;
	created: ((uri: unknown) => Promise<void> | void)[];
	changed: ((uri: unknown) => Promise<void> | void)[];
	deleted: ((uri: unknown) => Promise<void> | void)[];
}[] = [];
const findFilesCalls: { exclude?: unknown; include: unknown }[] = [];
const openedDocuments: unknown[] = [];
const analyzedInputs: unknown[] = [];
let codeLensRefreshCount = 0;
const refreshedDocuments: unknown[] = [];
let activeTextEditor: { document: unknown } | undefined;
let workspaceFolders: { uri: { fsPath: string } }[] | undefined = [
	{ uri: { fsPath: "/workspace" } },
];
const filesConfig: Record<string, unknown> = {
	exclude: { "**/dist/**": true, "**/tmp/**": false },
};
const versionlensConfig: Record<string, unknown> = {
	"hex.files": "**/{mix.exs,rebar.config,gleam.toml}",
	"npm.files": "**/package.json",
};

function uri(value: string) {
	return {
		fsPath: value.replace("file://", ""),
		scheme: value.split(":")[0],
		toString: () => value,
	};
}

function document(
	value: string,
	text = packageFileFixture("package-left-pad.json"),
) {
	return {
		getText: () => text,
		isDirty: false,
		languageId: "json",
		uri: uri(value),
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
	commands: {
		executeCommand: () => undefined,
	},
	window: {
		get activeTextEditor() {
			return activeTextEditor;
		},
	},
	workspace: {
		createFileSystemWatcher(pattern: unknown) {
			const watcher = {
				disposeCount: 0,
				pattern,
				created: [] as ((uri: unknown) => Promise<void> | void)[],
				changed: [] as ((uri: unknown) => Promise<void> | void)[],
				deleted: [] as ((uri: unknown) => Promise<void> | void)[],
				dispose: () => {
					watcher.disposeCount += 1;
				},
				onDidChange(listener: (uri: unknown) => Promise<void> | void) {
					watcher.changed.push(listener);
					return { dispose: () => undefined };
				},
				onDidCreate(listener: (uri: unknown) => Promise<void> | void) {
					watcher.created.push(listener);
					return { dispose: () => undefined };
				},
				onDidDelete(listener: (uri: unknown) => Promise<void> | void) {
					watcher.deleted.push(listener);
					return { dispose: () => undefined };
				},
			};
			createdWatchers.push(watcher);
			return watcher;
		},
		findFiles(include: unknown, exclude?: unknown) {
			findFilesCalls.push({ exclude, include });
			return [uri("file:///workspace/package.json")];
		},
		getConfiguration(section?: string) {
			const config = section === "files" ? filesConfig : versionlensConfig;
			return {
				get(key: string, fallback?: unknown) {
					return Object.hasOwn(config, key) ? config[key] : fallback;
				},
			};
		},
		get workspaceFolders() {
			return workspaceFolders;
		},
		getWorkspaceFolder: (documentUri: { fsPath?: string }) =>
			documentUri.fsPath?.startsWith("/workspace/")
				? workspaceFolders?.[0]
				: undefined,
		openTextDocument(openedUri: unknown) {
			openedDocuments.push(openedUri);
			return document((openedUri as { toString: () => string }).toString());
		},
	},
}));

mock.module("../../commands/codelens.ts", () => ({
	refreshCodeLenses: () => {
		codeLensRefreshCount += 1;
	},
}));

mock.module("../../commands.ts", () => ({
	updateContexts: () => undefined,
}));

mock.module("../../diagnostics.ts", () => ({
	refreshDiagnostics: (_state: unknown, doc: unknown) => {
		refreshedDocuments.push(doc);
	},
}));

function state() {
	return {
		flags: {
			providerBusy: 0,
			providerError: false,
			showVersionLenses: true,
		},
		session: {
			analyzeDocument(input: unknown) {
				analyzedInputs.push(input);
				return {
					codeLenses: [],
					dependencySignature: `signature-${analyzedInputs.length}`,
					diagnostics: [],
					isSupportedManifest: true,
				};
			},
		},
		lifecycle: {
			externalPackageFileWatchers: new Map<string, { dispose: () => void }[]>(),
			packageFileWatchers: [] as { dispose: () => void }[],
		},
		snapshots: {
			editedDependencies: new Map<string, string>(),
			savedDependencies: new Map<string, string>(),
		},
		ui: {
			codeLensRefresh: {
				fire: () => {
					codeLensRefreshCount += 1;
				},
			},
			diagnostics: { set: () => undefined },
			outputChannel: {},
		},
	};
}

function reset() {
	createdWatchers.length = 0;
	findFilesCalls.length = 0;
	openedDocuments.length = 0;
	analyzedInputs.length = 0;
	refreshedDocuments.length = 0;
	codeLensRefreshCount = 0;
	activeTextEditor = undefined;
	workspaceFolders = [{ uri: { fsPath: "/workspace" } }];
	delete versionlensConfig["enabledProviders"];
}

test("registers VS Code file watchers for configured package-file patterns", async () => {
	reset();
	const { registerPackageFileWatchers } = await import(
		"../../lifecycle/package-watchers.ts"
	);
	const subscriptions: unknown[] = [];

	registerPackageFileWatchers(state() as never, subscriptions as never[]);

	expect(
		createdWatchers.some((watcher) => watcher.pattern === "**/package.json"),
	).toBe(true);
	expect(
		createdWatchers.some(
			(watcher) => watcher.pattern === "**/{mix.exs,rebar.config,gleam.toml}",
		),
	).toBe(true);
	expect(createdWatchers.every((watcher) => watcher.created.length === 1)).toBe(
		true,
	);
	expect(createdWatchers.every((watcher) => watcher.changed.length === 1)).toBe(
		true,
	);
	expect(createdWatchers.every((watcher) => watcher.deleted.length === 1)).toBe(
		true,
	);
	expect(subscriptions.length).toBe(createdWatchers.length * 4);
});

test("registers package-file watchers only for enabledProviders like upstream", async () => {
	reset();
	versionlensConfig["enabledProviders"] = ["npm"];
	const { registerPackageFileWatchers } = await import(
		"../../lifecycle/package-watchers.ts"
	);

	registerPackageFileWatchers(state() as never);

	expect(createdWatchers.map((watcher) => watcher.pattern)).toEqual([
		"**/package.json",
	]);
});

test("initial workspace scan analyzes discovered package files through the native session", async () => {
	reset();
	const { scanWorkspacePackageFiles } = await import(
		"../../lifecycle/package-watchers.ts"
	);
	const currentState = state();

	await scanWorkspacePackageFiles(currentState as never);

	const npmFindCall = findFilesCalls.find(
		(call) => call.include === "**/package.json",
	);
	expect(npmFindCall).toBeDefined();
	expect(String(npmFindCall?.exclude)).toContain("**/dist/**");
	expect(
		openedDocuments.map((opened) =>
			(opened as { toString: () => string }).toString(),
		),
	).toContain("file:///workspace/package.json");
	expect(analyzedInputs).toContainEqual({
		languageId: "json",
		text: packageFileFixture("package-left-pad.json"),
		uri: "file:///workspace/package.json",
		workspaceRoot: "/workspace",
	});
	expect(
		currentState.snapshots.savedDependencies.get(
			"file:///workspace/package.json",
		),
	).toBe("signature-1");
});

test("watcher change refreshes package snapshot and active editor UI when dependencies changed", async () => {
	reset();
	const { registerPackageFileWatchers } = await import(
		"../../lifecycle/package-watchers.ts"
	);
	const currentState = state();
	const activeDocument = document("file:///workspace/package.json");
	activeTextEditor = { document: activeDocument };
	currentState.snapshots.savedDependencies.set(
		"file:///workspace/package.json",
		"old-signature",
	);

	registerPackageFileWatchers(currentState as never, [] as never[]);
	const npmWatcher = createdWatchers.find(
		(watcher) => watcher.pattern === "**/package.json",
	);
	await npmWatcher?.changed[0]?.(uri("file:///workspace/package.json"));

	expect(
		currentState.snapshots.savedDependencies.get(
			"file:///workspace/package.json",
		),
	).toBe("signature-1");
	expect(refreshedDocuments).toEqual([activeDocument]);
	expect(codeLensRefreshCount).toBe(1);
});

test("watcher delete removes cached package dependency snapshots", async () => {
	reset();
	const { registerPackageFileWatchers } = await import(
		"../../lifecycle/package-watchers.ts"
	);
	const currentState = state();
	currentState.snapshots.savedDependencies.set(
		"file:///workspace/package.json",
		"saved",
	);
	currentState.snapshots.editedDependencies.set(
		"file:///workspace/package.json",
		"edited",
	);

	registerPackageFileWatchers(currentState as never, [] as never[]);
	const npmWatcher = createdWatchers.find(
		(watcher) => watcher.pattern === "**/package.json",
	);
	npmWatcher?.deleted[0]?.(uri("file:///workspace/package.json"));

	expect(
		currentState.snapshots.savedDependencies.has(
			"file:///workspace/package.json",
		),
	).toBe(false);
	expect(
		currentState.snapshots.editedDependencies.has(
			"file:///workspace/package.json",
		),
	).toBe(false);
});

test("single-file activation analyzes the active file without workspace scanning", async () => {
	reset();
	const { initializePackageFileWatching } = await import(
		"../../lifecycle/package-watchers.ts"
	);
	const currentState = state();
	const activeDocument = document("file:///standalone/package.json");
	activeTextEditor = { document: activeDocument };
	workspaceFolders = undefined;

	await initializePackageFileWatching(currentState as never);

	expect(findFilesCalls).toEqual([]);
	expect(analyzedInputs).toContainEqual({
		languageId: "json",
		text: packageFileFixture("package-left-pad.json"),
		uri: "file:///standalone/package.json",
	});
	expect(
		currentState.snapshots.savedDependencies.get(
			"file:///standalone/package.json",
		),
	).toBe("signature-1");
});

test("single-file mode registers an out-of-workspace watcher for the active file", async () => {
	reset();
	const { registerPackageFileWatchers } = await import(
		"../../lifecycle/package-watchers.ts"
	);
	workspaceFolders = undefined;
	activeTextEditor = { document: document("file:///standalone/package.json") };

	registerPackageFileWatchers(state() as never, [] as never[]);

	expect(createdWatchers).toHaveLength(1);
	expect(createdWatchers[0]?.pattern).toMatchObject({
		base: "/standalone",
		pattern: "package.json",
	});
});

test("workspace mode watches activated package files outside the workspace", async () => {
	reset();
	const { watchActivePackageFileOutsideWorkspace } = await import(
		"../../lifecycle/package-watchers.ts"
	);
	const currentState = state();
	const activeDocument = document("file:///outside/package.json");
	activeTextEditor = { document: activeDocument };

	watchActivePackageFileOutsideWorkspace(
		currentState as never,
		activeDocument as never,
	);

	expect(analyzedInputs).toContainEqual({
		languageId: "json",
		text: packageFileFixture("package-left-pad.json"),
		uri: "file:///outside/package.json",
	});
	expect(
		currentState.snapshots.savedDependencies.get(
			"file:///outside/package.json",
		),
	).toBe("signature-1");
	expect(createdWatchers.at(-1)?.pattern).toMatchObject({
		base: "/outside",
		pattern: "package.json",
	});
});

test("disposing package file watchers clears workspace and external watcher state", async () => {
	reset();
	const {
		disposePackageFileWatchers,
		registerPackageFileWatchers,
		watchActivePackageFileOutsideWorkspace,
	} = await import("../../lifecycle/package-watchers.ts");
	const currentState = state();

	registerPackageFileWatchers(currentState as never, [] as never[]);
	const activeDocument = document("file:///outside/package.json");
	watchActivePackageFileOutsideWorkspace(
		currentState as never,
		activeDocument as never,
	);

	disposePackageFileWatchers(currentState as never);

	expect(createdWatchers.every((watcher) => watcher.disposeCount > 0)).toBe(
		true,
	);
	expect(currentState.lifecycle.packageFileWatchers).toEqual([]);
	expect(currentState.lifecycle.externalPackageFileWatchers.size).toBe(0);
});

function packageFileFixture(name: string): string {
	return readFileSync(
		`${process.cwd()}/tests/fixtures/vscode-extension/${name}`,
		"utf8",
	);
}
