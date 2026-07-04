import { expect, mock, test } from "bun:test";

mock.module("vscode", () => ({
	languages: {
		createDiagnosticCollection: () => ({
			clear: () => undefined,
			dispose: () => undefined,
		}),
	},
	window: {
		createOutputChannel: () => ({ dispose: () => undefined }),
	},
}));

test("deactivation disposes package file watchers and clears lifecycle state", async () => {
	const { deactivateExtension } = await import("./deactivate.ts");
	let sessionDisposeCount = 0;
	let packageWatcherDisposeCount = 0;
	let externalWatcherDisposeCount = 0;
	let subscriptionDisposeCount = 0;
	const subscriptions = [
		{
			dispose() {
				subscriptionDisposeCount += 1;
			},
		},
	];
	const state = {
		context: { subscriptions },
		lifecycle: {
			externalPackageFileWatchers: new Map([
				[
					"file:///outside/package.json",
					[
						{
							dispose() {
								externalWatcherDisposeCount += 1;
							},
						},
					],
				],
			]),
			packageFileWatchers: [
				{
					dispose() {
						packageWatcherDisposeCount += 1;
					},
				},
			],
		},
		session: {
			disposeSession() {
				sessionDisposeCount += 1;
			},
		},
		snapshots: {
			editedDependencies: new Map([["file:///package.json", "edited"]]),
			savedDependencies: new Map([["file:///package.json", "saved"]]),
		},
		ui: {
			codeLensRefresh: undefined,
			diagnostics: undefined,
			outputChannel: undefined,
		},
	};

	deactivateExtension(state as never);

	expect(packageWatcherDisposeCount).toBe(1);
	expect(externalWatcherDisposeCount).toBe(1);
	expect(state.lifecycle.packageFileWatchers).toEqual([]);
	expect(state.lifecycle.externalPackageFileWatchers.size).toBe(0);
	expect(subscriptionDisposeCount).toBe(1);
	expect(subscriptions).toEqual([]);
	expect(sessionDisposeCount).toBe(1);
	expect(state.session).toBeUndefined();
	expect(state.context).toBeUndefined();
	expect(state.snapshots.editedDependencies.size).toBe(0);
	expect(state.snapshots.savedDependencies.size).toBe(0);
});
