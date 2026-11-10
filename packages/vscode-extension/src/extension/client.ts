import { stat } from "node:fs/promises";

import type { LanguageClient } from "vscode-languageclient/node";

import { languages, window, workspace } from "#vscode-host";

import { documentSelectors } from "./documents.ts";
import { createLanguageClient } from "./language-client.ts";
import { clearManifestInfo } from "./manifest.ts";
import { resolveServerPath, SERVER_ENVIRONMENT_VARIABLE } from "./server.ts";
import { resetProviderState } from "./session/status.ts";
import type { ExtensionState } from "./state.ts";

/**
 * Start, stop and restart run one after another on one chain. A client that
 * is still starting cannot be shut down, so a stop waits for its start.
 */
const enqueue = (
	state: ExtensionState,
	step: () => Promise<void>,
): Promise<void> => {
	const run = (async (): Promise<void> => {
		await state.lifecycle;
		await step();
	})();
	state.lifecycle = (async (): Promise<void> => {
		try {
			await run;
		} catch {
			// The caller of `run` receives the failure; the chain goes on.
		}
	})();
	return run;
};

/** `required` starts the server even when no manifest needs it. */
const startClient = (state: ExtensionState, required = false): Promise<void> =>
	enqueue(state, (): Promise<void> => start(state, required));

const stopClient = (state: ExtensionState): Promise<void> =>
	enqueue(state, (): Promise<void> => stop(state));

const restartClient = (
	state: ExtensionState,
	required = false,
): Promise<void> =>
	enqueue(state, async (): Promise<void> => {
		try {
			await stop(state);
		} finally {
			await start(state, required);
		}
	});

/**
 * Whether a supported manifest is open, or a folder asks for the whole
 * workspace to be checked. Until then the server is not started.
 */
const serverNeeded = (): boolean => {
	const selectors = documentSelectors();
	const scopes = workspace.workspaceFolders?.map(({ uri }) => uri) ?? [
		undefined,
	];
	return (
		workspace.textDocuments.some(
			(document): boolean => languages.match(selectors, document) > 0,
		) ||
		scopes.some(
			(scope): boolean =>
				workspace
					.getConfiguration("versionlens", scope)
					.get<string>("workspace.check") === "workspace",
		)
	);
};

const pathExists = async (path: string): Promise<boolean> => {
	try {
		await stat(path);
		return true;
	} catch {
		return false;
	}
};

/**
 * Starts the server when it is needed and not running. A server that cannot
 * start is reported, and the commands that do not need it keep working.
 */
const start = async (
	state: ExtensionState,
	required: boolean,
): Promise<void> => {
	const extensionPath = state.context?.extensionPath;
	if (
		extensionPath === undefined ||
		extensionPath === "" ||
		state.client !== undefined ||
		!(required || serverNeeded())
	) {
		return;
	}
	const serverPath = resolveServerPath(extensionPath);
	if (!(await pathExists(serverPath))) {
		const message = `VersionLens Redux could not find its language server (${serverPath}). Install the extension for your platform, or set ${SERVER_ENVIRONMENT_VARIABLE} to a versionlens-lsp binary.`;
		state.ui.outputChannel?.error(message);
		// Not awaited: the promise settles only when the user closes the message.
		void window.showErrorMessage(message);
		return;
	}
	state.selectorKey = JSON.stringify(documentSelectors());
	const client = createLanguageClient(state, serverPath);
	state.client = client;
	try {
		await client.start();
		state.ui.outputChannel?.info(`language server started (${serverPath})`);
	} catch (error) {
		// The client already told the user that it could not connect.
		state.client = undefined;
		await disposeClient(state, client);
		state.ui.outputChannel?.error(
			`could not start the language server (${serverPath}): ${String(error)}`,
		);
	}
};

const stop = async (state: ExtensionState): Promise<void> => {
	const { client } = state;
	state.client = undefined;
	clearManifestInfo();
	state.diagnostics.clear();
	state.promptedHosts.clear();
	await resetProviderState(state);
	if (client) {
		await disposeClient(state, client);
	}
};

const disposeClient = async (
	state: ExtensionState,
	client: LanguageClient,
): Promise<void> => {
	try {
		await client.dispose();
	} catch (error) {
		state.ui.outputChannel?.error(
			`could not stop the server: ${String(error)}`,
		);
	}
};

/** Shows or hides the diagnostics the server published. */
const applyDiagnosticsVisibility = (state: ExtensionState): void => {
	const collection = state.client?.diagnostics;
	for (const { diagnostics, uri } of state.diagnostics.values()) {
		collection?.set(uri, state.flags.diagnosticsHidden ? [] : diagnostics);
	}
};

/** Asks the editor to ask the server for lenses again. */
const refreshCodeLenses = (state: ExtensionState): void => {
	const feature = state.client?.getFeature("textDocument/codeLens");
	for (const editor of window.visibleTextEditors) {
		const provider = feature?.getProvider(editor.document);
		if (provider) {
			provider.onDidChangeCodeLensEmitter.fire();
			return;
		}
	}
};

export {
	applyDiagnosticsVisibility,
	refreshCodeLenses,
	restartClient,
	startClient,
	stopClient,
};
