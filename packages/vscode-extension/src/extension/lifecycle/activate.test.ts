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

mock.module("vscode", () => ({
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

mock.module("../commands.ts", () => ({
	updateContexts: () => {
		updateContextCount += 1;
	},
}));

mock.module("../diagnostics.ts", () => ({
	refreshActiveDiagnostics: () => {
		refreshDiagnosticsCount += 1;
	},
}));

mock.module("../session.ts", () => ({
	recreateSession: () => {
		recreateSessionCount += 1;
		if (recreateSessionError) {
			throw recreateSessionError;
		}
	},
	reloadConfigurationState: () => undefined,
}));

mock.module("./package-watchers.ts", () => ({
	initializePackageFileWatching: () => {
		packageWatchingCount += 1;
	},
}));

mock.module("./subscriptions.ts", () => ({
	registerExtensionSubscriptions: () => {
		subscriptionCount += 1;
	},
}));

mock.module("./ui.ts", () => ({
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
}

test("activation warns when VS Code editor code lenses are disabled", async () => {
	reset();
	editorCodeLens = false;
	const { activateExtension } = await import("./activate.ts");
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
	const { activateExtension } = await import("./activate.ts");
	const state = { context: undefined, ui: {} };

	await expect(
		activateExtension(state as never, { subscriptions: [] } as never),
	).rejects.toThrow("native session failed");

	expect(uiInitCount).toBe(1);
	expect(subscriptionCount).toBe(1);
	expect(recreateSessionCount).toBe(1);
	expect(updateContextCount).toBe(0);
	expect(packageWatchingCount).toBe(0);
	expect(refreshDiagnosticsCount).toBe(0);
});

test("activation does not warn when VS Code editor code lenses are enabled", async () => {
	reset();
	const { activateExtension } = await import("./activate.ts");
	const state = { context: undefined, ui: {} };

	await activateExtension(state as never, { subscriptions: [] } as never);

	expect(outputLines).toEqual([]);
});
