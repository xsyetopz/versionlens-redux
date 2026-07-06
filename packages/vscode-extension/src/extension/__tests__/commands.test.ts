import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

import {
	appliedEdits,
	clearRegisteredCommands,
	commandState,
	completeTask,
	configurationChangeListeners,
	contexts,
	createdSessionConfigs,
	executedTasks,
	fileSystemWatchers,
	openedExternalUris,
	quickPickItems,
	quickPickOptions,
	registeredCommands,
	shownTextDocuments,
	smokeTaskLabel,
	testState,
	warningMessages,
} from "./commands.test-support.ts";

test("updateContexts marks provider active only for supported manifests", async () => {
	const { updateContexts } = await import("../commands.ts");
	testState.activeTextEditor = {
		document: {
			uri: { scheme: "file", toString: () => "file:///package.json" },
		},
	};
	testState.analyzed = {
		canSortDependencies: true,
		isSupportedManifest: false,
	};

	await updateContexts({
		flags: { showVersionLenses: true, showPrereleases: false },
	} as never);

	expect(contexts["versionlens.providerActive"]).toBe(false);
	expect(contexts["versionlens.showSortAlphabetically"]).toBe(false);

	testState.analyzed = {
		activeProviderName: "npm",
		canSortDependencies: false,
		isSupportedManifest: true,
	};
	await updateContexts({
		flags: { showVersionLenses: true, showPrereleases: false },
	} as never);

	expect(contexts["versionlens.providerActive"]).toBe("npm");
	expect(contexts["versionlens.showSortAlphabetically"]).toBe(false);

	await updateContexts({
		flags: {
			showOutdated: true,
			showPrereleases: false,
			showVersionLenses: true,
		},
	} as never);

	expect(contexts["versionlens.showOutdated"]).toBe(true);
});

test("updateContexts disables provider actions for non-file active editors", async () => {
	const { updateContexts } = await import("../commands.ts");
	testState.activeTextEditor = {
		document: {
			uri: {
				scheme: "versionlens",
				toString: () => "versionlens:/versionlens.multi-registries.json",
			},
		},
	};
	testState.analyzed = {
		activeProviderName: "npm",
		canSortDependencies: true,
		installTaskConfigKey: "npm.onSaveChanges",
		isSupportedManifest: true,
	};

	await updateContexts({
		flags: { showVersionLenses: true, showPrereleases: false },
	} as never);

	expect(contexts["versionlens.providerActive"]).toBe(false);
	expect(contexts["versionlens.showCustomInstall"]).toBe(false);
	expect(contexts["versionlens.showSortAlphabetically"]).toBe(false);
});

test("provider busy context is a numeric counter like upstream", async () => {
	const { setProviderState } = await import("../diagnostics/provider.ts");
	const state = commandState(undefined);

	setProviderState(state as never, true, false);
	expect(contexts["versionlens.providerBusy"]).toBe(1);
	setProviderState(state as never, true, false);
	expect(contexts["versionlens.providerBusy"]).toBe(2);
	setProviderState(state as never, false, false);
	expect(contexts["versionlens.providerBusy"]).toBe(1);
	setProviderState(state as never, false, false);
	expect(contexts["versionlens.providerBusy"]).toBe(0);
	setProviderState(state as never, false, false);
	expect(contexts["versionlens.providerBusy"]).toBe(0);
});

test("error click clears all provider busy state like upstream", async () => {
	const { showProviderError } = await import("../commands/error.ts");
	testState.activeTextEditor = {
		document: {
			uri: { scheme: "file", toString: () => "file:///package.json" },
		},
	};
	const shown: string[] = [];
	const state = commandState(undefined, {
		flags: {
			providerBusy: 3,
			providerError: true,
			showOutdated: false,
			showPrereleases: false,
			showSuggestionStats: false,
			showVersionLenses: true,
			codeLensReplace: true,
		},
		ui: {
			codeLensRefresh: undefined,
			diagnostics: undefined,
			outputChannel: { show: () => shown.push("show") },
		},
	});

	showProviderError(state as never);

	expect(shown).toEqual(["show"]);
	expect(state.flags.providerBusy).toBe(0);
	expect(state.flags.providerError).toBe(false);
	expect(contexts["versionlens.providerBusy"]).toBe(0);
	expect(contexts["versionlens.providerError"]).toBe(false);
	expect(shownTextDocuments).toHaveLength(1);
});

test("versionlens configuration changes recreate the native session and refresh active diagnostics", async () => {
	const { registerExtensionSubscriptions } = await import(
		"../lifecycle/subscriptions.ts"
	);
	const document = {
		uri: { scheme: "file", toString: () => "file:///package.json" },
	};
	const context = { extensionPath: "/extension", subscriptions: [] };
	const disposed: string[] = [];
	configurationChangeListeners.length = 0;
	createdSessionConfigs.length = 0;
	fileSystemWatchers.length = 0;
	testState.activeRefreshCount = 0;
	testState.codeLensRefreshCount = 0;
	testState.activeTextEditor = { document };
	testState.analyzed = {
		activeProviderName: "npm",
		canSortDependencies: true,
		isSupportedManifest: true,
	};
	let editedClearCount = 0;
	let savedClearCount = 0;
	const editedDependencies = new Map([["file:///package.json", "edited"]]);
	const savedDependencies = new Map([["file:///package.json", "saved"]]);
	const clearEditedDependencies =
		editedDependencies.clear.bind(editedDependencies);
	const clearSavedDependencies =
		savedDependencies.clear.bind(savedDependencies);
	editedDependencies.clear = () => {
		editedClearCount += 1;
		clearEditedDependencies();
	};
	savedDependencies.clear = () => {
		savedClearCount += 1;
		clearSavedDependencies();
	};

	const state = {
		context,
		flags: {
			providerBusy: 0,
			providerError: false,
			showPrereleases: true,
			showSuggestionStats: true,
			showVersionLenses: true,
		},
		session: {
			disposeSession() {
				disposed.push("disposed");
			},
		},
		snapshots: {
			editedDependencies,
			savedDependencies,
		},
		ui: {
			diagnostics: { delete: () => undefined },
			outputChannel: {},
		},
	};

	registerExtensionSubscriptions(state as never, context as never);
	const initialWatcherCount = fileSystemWatchers.length;
	await configurationChangeListeners[0]?.({
		affectsConfiguration: (section) => section === "versionlens",
	});

	expect(disposed).toEqual(["disposed"]);
	expect(createdSessionConfigs).toHaveLength(1);
	expect(
		fileSystemWatchers
			.slice(0, initialWatcherCount)
			.every((watcher) => watcher.disposed),
	).toBe(true);
	expect(fileSystemWatchers.length).toBeGreaterThan(initialWatcherCount);
	expect(editedClearCount).toBe(1);
	expect(savedClearCount).toBe(1);
	expect(testState.codeLensRefreshCount).toBe(1);
	expect(testState.activeRefreshCount).toBe(1);
	expect(contexts["versionlens.providerActive"]).toBe("npm");
});

test("clear cache command clears native cache, diagnostics, and dependency snapshots", async () => {
	const { registerCommands } = await import("../commands.ts");
	let nativeClearCount = 0;
	let diagnosticsClearCount = 0;
	testState.activeRefreshCount = 0;
	testState.codeLensRefreshCount = 0;
	clearRegisteredCommands();
	const snapshots = {
		editedDependencies: new Map([["file:///package.json", "edited"]]),
		savedDependencies: new Map([["file:///package.json", "saved"]]),
	};
	const state = commandState(
		{ clearCache: () => (nativeClearCount += 1) },
		{
			snapshots,
			ui: { diagnostics: { clear: () => (diagnosticsClearCount += 1) } },
		},
	);

	registerCommands(state as never);
	await registeredCommands["versionlens.suggestion.onClearCache"]?.();

	expect(nativeClearCount).toBe(1);
	expect(diagnosticsClearCount).toBe(1);
	expect(snapshots.editedDependencies.size).toBe(0);
	expect(snapshots.savedDependencies.size).toBe(0);
	expect(testState.codeLensRefreshCount).toBe(1);
	expect(testState.activeRefreshCount).toBe(1);
});

test("choose build returns before QuickPick when CodeLens replacement is disabled", async () => {
	const { chooseBuild } = await import("../commands/build.ts");
	quickPickItems.length = 0;

	await chooseBuild(
		commandState(undefined, {
			flags: { codeLensReplace: false },
		}) as never,
		"left-pad\u001f0:30,0:43",
		"left-pad",
		"1.0.0+build.1",
		["1.0.0+build.1", "1.0.0+build.2"],
	);

	expect(quickPickItems).toHaveLength(0);
});

test("version lens toggles refresh registered code lenses", async () => {
	const { registerCommands } = await import("../commands.ts");
	clearRegisteredCommands();
	testState.codeLensRefreshCount = 0;
	testState.activeRefreshCount = 0;
	createdSessionConfigs.length = 0;
	testState.activeTextEditor = {
		document: {
			uri: { scheme: "file", toString: () => "file:///package.json" },
		},
	};
	testState.analyzed = {
		activeProviderName: "npm",
		canSortDependencies: true,
		isSupportedManifest: true,
	};
	const state = commandState(
		{ disposeSession: () => undefined },
		{
			context: {
				extensionPath: "/extension",
				secrets: { get: () => undefined },
			},
		},
	);

	registerCommands(state as never);
	await registeredCommands["versionlens.editor.onHideVersionLenses"]?.();
	await registeredCommands["versionlens.editor.onShowPrereleaseVersions"]?.();

	expect(testState.codeLensRefreshCount).toBe(2);
	expect(testState.activeRefreshCount).toBe(0);
	expect(state.flags.showVersionLenses).toBe(false);
	expect(state.flags.showPrereleases).toBe(true);
	expect(createdSessionConfigs).toHaveLength(1);
});

test("refresh timer refreshes active diagnostics on schedule", async () => {
	const { registerRefreshTimer } = await import(
		"../lifecycle/refresh-timer.ts"
	);
	const originalSetInterval = globalThis.setInterval;
	const originalClearInterval = globalThis.clearInterval;
	let scheduled:
		| {
				callback: () => void;
				delay: number | undefined;
				timer: { unref: () => void };
		  }
		| undefined;
	let cleared: unknown;
	testState.activeRefreshCount = 0;

	globalThis.setInterval = ((callback: () => void, delay?: number) => {
		const timer = {
			[Symbol.dispose]: () => undefined,
			[Symbol.toPrimitive]: () => 0,
			hasRef: () => false,
			ref: () => timer,
			refresh: () => timer,
			unref: () => timer,
		};
		scheduled = { callback, delay, timer };
		return timer;
	}) as unknown as typeof setInterval;
	globalThis.clearInterval = ((timer: unknown) => {
		cleared = timer;
	}) as typeof clearInterval;

	try {
		const disposable = registerRefreshTimer({} as never, 123);
		expect(scheduled?.delay).toBe(123);

		scheduled?.callback();

		expect(testState.activeRefreshCount).toBe(1);
		disposable.dispose();
		expect(cleared).toBe(scheduled?.timer);
	} finally {
		globalThis.setInterval = originalSetInterval;
		globalThis.clearInterval = originalClearInterval;
	}
});

test("custom install command only runs for file-backed active editors", async () => {
	const { registerCommands } = await import("../commands.ts");
	executedTasks.length = 0;
	testState.taskCompletionMode = "auto";
	clearRegisteredCommands();
	testState.analyzed = {
		activeProviderName: "npm",
		canSortDependencies: true,
		installTaskConfigKey: "npm.onSaveChanges",
		isSupportedManifest: true,
	};

	testState.activeTextEditor = {
		document: {
			uri: {
				scheme: "versionlens",
				toString: () => "versionlens:/versionlens.multi-registries.json",
			},
		},
	};
	registerCommands(commandState({}) as never);
	await registeredCommands["versionlens.editor.onCustomInstall"]?.();
	expect(executedTasks).toEqual([]);

	testState.activeTextEditor = {
		document: {
			uri: { scheme: "file", toString: () => "file:///package.json" },
		},
	};
	await registeredCommands["versionlens.editor.onCustomInstall"]?.();
	expect(executedTasks).toEqual([smokeTaskLabel]);
});

test("save task ignores unsupported documents without creating snapshots", async () => {
	const { handleDidSaveTextDocument } = await import("../tasks.ts");
	executedTasks.length = 0;
	testState.analyzed = {
		canSortDependencies: false,
		isSupportedManifest: false,
	};
	testState.dependencySnapshotValue = "";
	const state = {
		flags: { showOutdated: true },
		snapshots: {
			editedDependencies: new Map<string, string>(),
			savedDependencies: new Map<string, string>(),
		},
	};
	const document = {
		uri: { scheme: "file", toString: () => "file:///README.md" },
	};

	await handleDidSaveTextDocument(state as never, document as never);

	expect(executedTasks).toEqual([]);
	expect(state.snapshots.savedDependencies.has("file:///README.md")).toBe(
		false,
	);
	expect(state.snapshots.editedDependencies.has("file:///README.md")).toBe(
		false,
	);
	expect(state.flags.showOutdated).toBe(true);
});

test("save task runs only after dependency signature changes", async () => {
	const { handleDidSaveTextDocument } = await import("../tasks.ts");
	executedTasks.length = 0;
	const startingRefreshCount = testState.refreshCount;
	testState.taskCompletionMode = "auto";
	const state = {
		flags: { showOutdated: false },
		snapshots: {
			editedDependencies: new Map<string, string>(),
			savedDependencies: new Map<string, string>(),
		},
	};
	const document = {
		uri: { scheme: "file", toString: () => "file:///package.json" },
	};

	testState.analyzed = {
		activeProviderName: "npm",
		canSortDependencies: true,
		installTaskConfigKey: "npm.onSaveChanges",
		isSupportedManifest: true,
	};
	testState.dependencySnapshotValue = "left-pad@1";

	await handleDidSaveTextDocument(state as never, document as never);
	expect(executedTasks).toEqual([]);

	state.snapshots.editedDependencies.set("file:///package.json", "left-pad@2");
	await handleDidSaveTextDocument(state as never, document as never);
	expect(executedTasks).toEqual([smokeTaskLabel]);

	testState.dependencySnapshotValue = "left-pad@2";
	await handleDidSaveTextDocument(state as never, document as never);
	expect(executedTasks).toEqual([smokeTaskLabel]);
	expect(testState.refreshCount).toBe(startingRefreshCount + 3);
});

test("save task can retry while install is running or failed", async () => {
	const { handleDidSaveTextDocument } = await import("../tasks.ts");
	executedTasks.length = 0;
	testState.taskCompletionMode = "manual";
	const state = {
		flags: { showOutdated: false },
		snapshots: {
			editedDependencies: new Map<string, string>(),
			savedDependencies: new Map<string, string>(),
		},
	};
	const document = {
		uri: { scheme: "file", toString: () => "file:///retry-package.json" },
	};
	const key = "file:///retry-package.json";

	testState.analyzed = {
		activeProviderName: "npm",
		canSortDependencies: true,
		installTaskConfigKey: "npm.onSaveChanges",
		isSupportedManifest: true,
	};
	testState.dependencySnapshotValue = "left-pad@1";

	await handleDidSaveTextDocument(state as never, document as never);
	expect(state.snapshots.savedDependencies.get(key)).toBe("left-pad@1");

	state.snapshots.editedDependencies.set(key, "left-pad@2");
	const running = handleDidSaveTextDocument(state as never, document as never);
	await Promise.resolve();
	expect(executedTasks).toEqual([smokeTaskLabel]);
	expect(state.snapshots.savedDependencies.get(key)).toBe("left-pad@1");

	await handleDidSaveTextDocument(state as never, document as never);
	expect(executedTasks).toEqual([smokeTaskLabel]);
	expect(state.snapshots.savedDependencies.get(key)).toBe("left-pad@1");

	completeTask(smokeTaskLabel, 1);
	await running;
	expect(state.snapshots.savedDependencies.get(key)).toBe("left-pad@1");

	testState.taskCompletionMode = "auto";
	await handleDidSaveTextDocument(state as never, document as never);
	expect(executedTasks).toEqual([smokeTaskLabel, smokeTaskLabel]);
	expect(state.snapshots.savedDependencies.get(key)).toBe("left-pad@2");
});

test("resolve command applies Rust-produced edits", async () => {
	const { registerCommands } = await import("../commands.ts");
	const applyInputs: unknown[] = [];
	const startingRefreshCount = testState.refreshCount;
	appliedEdits.length = 0;
	clearRegisteredCommands();
	const document = {
		getText: () => packageFileFixture("package-left-pad.json"),
		languageId: "json",
		uri: { toString: () => "file:///package.json" },
	};
	const session = {
		applyCommand: (input: unknown) => {
			applyInputs.push(input);
			return {
				edits: [
					{
						newText: "1.1.0",
						range: {
							end: { character: 35, line: 0 },
							start: { character: 30, line: 0 },
						},
					},
				],
			};
		},
	};

	testState.activeTextEditor = { document };
	const state = commandState(session);
	registerCommands(state as never);
	await registeredCommands["versionlens.suggestion.onUpdateDependency"]?.(
		"left-pad",
		"left-pad\u001f0:30,0:35",
	);
	await registeredCommands["versionlens.editor.onSortDependencies"]?.();
	state.flags.codeLensReplace = true;
	await registeredCommands["versionlens.editor.onUpdateDependenciesMinor"]?.();

	expect(applyInputs).toMatchObject([
		{ dependencyName: "left-pad\u001f0:30,0:35" },
		{ command: "sort" },
		{ command: "updateMinor" },
	]);
	expect(appliedEdits).toHaveLength(3);
	expect(appliedEdits[0]).toMatchObject({ newText: "1.1.0" });
	expect(testState.refreshCount).toBe(startingRefreshCount + 3);
});

test("open dependency command opens Rust-produced local paths", async () => {
	const { registerCommands } = await import("../commands.ts");
	openedExternalUris.length = 0;
	shownTextDocuments.length = 0;
	testState.dependencyFileType = 2;
	clearRegisteredCommands();

	registerCommands(commandState(undefined) as never);
	await registeredCommands["versionlens.suggestion.onFileLink"]?.(
		"/repo/local",
	);

	expect(openedExternalUris).toEqual([{ path: "/repo/local", scheme: "file" }]);
	expect(shownTextDocuments).toEqual([]);
});

test("choose build command applies the selected Rust build edit", async () => {
	const { registerCommands } = await import("../commands.ts");
	const applyInputs: unknown[] = [];
	appliedEdits.length = 0;
	quickPickItems.length = 0;
	quickPickOptions.length = 0;
	clearRegisteredCommands();
	const document = {
		getText: () => packageFileFixture("package-left-pad-build.json"),
		languageId: "json",
		uri: { toString: () => "file:///package.json" },
	};
	const session = {
		applyCommand: (input: unknown) => {
			applyInputs.push(input);
			return {
				edits: [
					{
						newText: "1.0.0+build.2",
						range: {
							end: { character: 43, line: 0 },
							start: { character: 30, line: 0 },
						},
					},
				],
				vulnerableUpdateCount: 0,
			};
		},
	};

	testState.activeTextEditor = { document };
	registerCommands(commandState(session) as never);
	await registeredCommands["versionlens.suggestion.onChooseBuild"]?.(
		"left-pad\u001f0:30,0:43",
		"left-pad",
		"1.0.0+build.1",
		"1.0.0+build.2",
		"1.0.0+build.1",
	);

	expect(quickPickOptions).toContainEqual({
		placeHolder: "Choose a build or press escape to cancel",
		title: "Choose a build for left-pad",
	});
	expect(quickPickItems).toContainEqual({
		label: "1.0.0+build.1",
		picked: true,
	});
	expect(quickPickItems).toContainEqual({
		label: "1.0.0+build.2",
		picked: false,
	});
	expect(applyInputs[0]).toMatchObject({
		dependencyName: "left-pad\u001f0:30,0:43",
		selectedVersion: "1.0.0+build.2",
	});
	expect(appliedEdits[0]).toMatchObject({ newText: "1.0.0+build.2" });
});

test("resolve command confirms vulnerable updates before applying edits", async () => {
	const { registerCommands } = await import("../commands.ts");
	appliedEdits.length = 0;
	warningMessages.length = 0;
	testState.warningChoice = undefined;
	clearRegisteredCommands();
	const document = {
		getText: () => packageFileFixture("package-left-pad.json"),
		languageId: "json",
		uri: { toString: () => "file:///package.json" },
	};
	const session = {
		applyCommand: () => ({
			vulnerableUpdatePackage: "left-pad",
			vulnerableUpdateVersion: "1.1.0",
			edits: [
				{
					newText: "1.1.0",
					range: {
						end: { character: 35, line: 0 },
						start: { character: 30, line: 0 },
					},
				},
			],
			vulnerableUpdateCount: 1,
		}),
	};

	testState.activeTextEditor = { document };
	registerCommands(commandState(session) as never);
	await registeredCommands["versionlens.suggestion.onUpdateDependency"]?.(
		"left-pad",
	);

	expect(warningMessages).toHaveLength(1);
	expect(appliedEdits).toEqual([]);

	testState.warningChoice = "Update Anyway";
	await registeredCommands["versionlens.suggestion.onUpdateDependency"]?.(
		"left-pad",
	);

	expect(warningMessages).toHaveLength(2);
	expect(warningMessages[1]).toEqual([
		"Vulnerabilities found in left-pad@1.1.0. Do you want to continue?",
		{ modal: true },
		"Update Anyway",
	]);
	expect(appliedEdits).toHaveLength(1);
	testState.warningChoice = undefined;
});

function packageFileFixture(name: string): string {
	return readFileSync(
		`${process.cwd()}/tests/fixtures/vscode-extension/${name}`,
		"utf8",
	);
}
