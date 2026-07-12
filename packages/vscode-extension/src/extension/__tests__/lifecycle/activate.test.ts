import { expect, mock, test } from "bun:test";

const outputLines: string[] = [];
let editorCodeLens = true;
let updateContextCount = 0;
let recreateSessionCount = 0;
let refreshDiagnosticsCount = 0;
let packageWatchingCount = 0;
let subscriptionCount = 0;
let uiInitCount = 0;
let recreateSessionError: Error | undefined;
let originalVersionLensInstalled = false;
let warningSelection: string | undefined;
const warningMessages: string[] = [];
const errorMessages: string[] = [];
const executedCommands: unknown[][] = [];

mock.module("vscode", () => ({
	commands: {
		executeCommand(...args: unknown[]) {
			executedCommands.push(args);
		},
	},
	extensions: {
		getExtension(id: string) {
			return originalVersionLensInstalled &&
				id === "pflannery.vscode-versionlens"
				? { id }
				: undefined;
		},
	},
	window: {
		showErrorMessage(message: string) {
			errorMessages.push(message);
		},
		showWarningMessage(message: string) {
			warningMessages.push(message);
			return warningSelection;
		},
	},
	workspace: {
		getConfiguration() {
			return {
				get(key: string, fallback?: unknown) {
					return key === "editor.codeLens" ? editorCodeLens : fallback;
				},
			};
		},
	},
}));

mock.module("../../commands.ts", () => ({
	updateContexts: () => {
		updateContextCount += 1;
	},
}));

mock.module("../../diagnostics.ts", () => ({
	refreshActiveDiagnostics: () => {
		refreshDiagnosticsCount += 1;
	},
}));

mock.module("../../session.ts", () => ({
	recreateSession: () => {
		recreateSessionCount += 1;
		if (recreateSessionError) {
			throw recreateSessionError;
		}
	},
	reloadConfigurationState: () => undefined,
}));

mock.module("../../lifecycle/package-watchers.ts", () => ({
	initializePackageFileWatching: () => {
		packageWatchingCount += 1;
	},
}));

mock.module("../../lifecycle/subscriptions.ts", () => ({
	registerExtensionSubscriptions: () => {
		subscriptionCount += 1;
	},
}));

mock.module("../../lifecycle/ui.ts", () => ({
	initializeUi: (state: {
		ui: { outputChannel?: { appendLine(value: string): void } };
	}) => {
		uiInitCount += 1;
		state.ui.outputChannel = {
			appendLine(value: string) {
				outputLines.push(value);
			},
		};
	},
}));

function reset() {
	outputLines.length = 0;
	editorCodeLens = true;
	updateContextCount = 0;
	recreateSessionCount = 0;
	refreshDiagnosticsCount = 0;
	packageWatchingCount = 0;
	subscriptionCount = 0;
	uiInitCount = 0;
	recreateSessionError = undefined;
	originalVersionLensInstalled = false;
	warningSelection = undefined;
	warningMessages.length = 0;
	errorMessages.length = 0;
	executedCommands.length = 0;
}

test("activation warns when VS Code editor code lenses are disabled", async () => {
	reset();
	editorCodeLens = false;
	const { activateExtension } = await import("../../lifecycle/activate.ts");
	const state = { context: undefined, ui: {} };

	await activateExtension(state as never, { subscriptions: [] } as never);

	expect(outputLines).toContain(
		"Code lenses are disabled. This extension won't work unless you enable 'editor.codeLens' in your vscode settings",
	);
	expect(uiInitCount).toBe(1);
	expect(recreateSessionCount).toBe(1);
	expect(updateContextCount).toBe(1);
	expect(packageWatchingCount).toBe(1);
	expect(subscriptionCount).toBe(1);
	expect(refreshDiagnosticsCount).toBe(1);
});

test("activation registers commands before native session creation can fail", async () => {
	reset();
	recreateSessionError = new Error("native session failed");
	const { activateExtension } = await import("../../lifecycle/activate.ts");
	const state = { context: undefined, ui: {} };

	await expect(
		activateExtension(state as never, { subscriptions: [] } as never),
	).rejects.toThrow("native session failed");
	const runtime = [process.platform, process.arch].join("-");
	expect(errorMessages).toEqual([
		`VersionLens Redux could not load its native runtime for ${runtime}. Install the matching platform package.`,
	]);
	expect(outputLines.at(-1)).toContain("native session failed");

	expect(uiInitCount).toBe(1);
	expect(subscriptionCount).toBe(1);
	expect(recreateSessionCount).toBe(1);
	expect(updateContextCount).toBe(0);
	expect(packageWatchingCount).toBe(0);
	expect(refreshDiagnosticsCount).toBe(0);
});

test("activation does not warn when VS Code editor code lenses are enabled", async () => {
	reset();
	const { activateExtension } = await import("../../lifecycle/activate.ts");
	const state = { context: undefined, ui: {} };

	await activateExtension(state as never, { subscriptions: [] } as never);

	expect(outputLines).toEqual([]);
});

test("activation warns when original VersionLens is installed", async () => {
	reset();
	originalVersionLensInstalled = true;
	const { activateExtension } = await import("../../lifecycle/activate.ts");
	const state = { context: undefined, ui: {} };

	await activateExtension(state as never, { subscriptions: [] } as never);

	const message =
		"VersionLens Redux conflicts with the original VersionLens extension. Disable the original extension before using VersionLens Redux in this workspace.";
	expect(outputLines).toContain(message);
	expect(warningMessages).toEqual([message]);
	expect(executedCommands).toEqual([]);
});

test("activation can disable original VersionLens from the conflict prompt", async () => {
	reset();
	originalVersionLensInstalled = true;
	warningSelection = "Disable original VersionLens";
	const { activateExtension } = await import("../../lifecycle/activate.ts");
	const state = { context: undefined, ui: {} };

	await activateExtension(state as never, { subscriptions: [] } as never);

	expect(executedCommands).toEqual([
		[
			"workbench.extensions.action.disableExtension",
			"pflannery.vscode-versionlens",
		],
	]);
});
