import { expect, mock, test } from "bun:test";

const diagnosticCollections: string[] = [];
let outputChannelCount = 0;
let statusBarItemCount = 0;

mock.module("vscode", () => ({
	languages: {
		createDiagnosticCollection(name: string) {
			diagnosticCollections.push(name);
			return { clear: () => undefined, dispose: () => undefined };
		},
	},
	StatusBarAlignment: { Right: 2 },
	window: {
		createOutputChannel() {
			outputChannelCount += 1;
			return { dispose: () => undefined };
		},
		createStatusBarItem() {
			statusBarItemCount += 1;
			return { dispose: () => undefined };
		},
	},
}));

test("initializes upstream diagnostics and output UI without a status bar item", async () => {
	const { initializeUi } = await import("../../lifecycle/ui.ts");
	const state = {
		ui: {
			codeLensRefresh: undefined,
			diagnostics: undefined,
			outputChannel: undefined,
		},
	};

	initializeUi(state as never);

	expect(diagnosticCollections).toEqual(["versionlens-vulnerabilities"]);
	expect(outputChannelCount).toBe(1);
	expect(statusBarItemCount).toBe(0);
});
