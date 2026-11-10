import {
	type Executable,
	ExecuteCommandRequest,
	LanguageClient,
	type LanguageClientOptions,
	TransportKind,
} from "vscode-languageclient/node";

import {
	answerAuthorization,
	type AuthorizationParams,
} from "./auth/authorization.ts";
import { reportError } from "./commands/error.ts";
import { withToggles } from "./config/toggles.ts";
import { documentSelectors } from "./documents.ts";
import { applyStatus, type StatusParams } from "./session/status.ts";
import type { ExtensionState } from "./state.ts";

const STATUS_METHOD = "versionlens/status";
const AUTHORIZATION_METHOD = "versionlens/authorization";

type Middleware = NonNullable<LanguageClientOptions["middleware"]>;

/**
 * Client capabilities come from `vscode-languageclient`: `applyEdit`,
 * `workspaceEdit.documentChanges`, `codeLens.refreshSupport` and
 * `workspace.configuration` are all advertised by default.
 */
const createLanguageClient = (
	state: ExtensionState,
	serverPath: string,
): LanguageClient => {
	const serverOptions: Executable = {
		command: serverPath,
		transport: TransportKind.stdio,
	};
	const clientOptions: LanguageClientOptions = {
		documentSelector: documentSelectors(),
		middleware: middleware(state, (command, args): Promise<unknown> =>
			client.sendRequest(ExecuteCommandRequest.type, {
				arguments: args,
				command,
			}),
		),
	};
	if (state.ui.outputChannel) {
		clientOptions.outputChannel = state.ui.outputChannel;
	}

	const client = new LanguageClient(
		"versionlens",
		"VersionLens Redux",
		serverOptions,
		clientOptions,
	);
	client.onNotification(
		STATUS_METHOD,
		async (params: StatusParams): Promise<void> => {
			try {
				await applyStatus(state, params);
			} catch (error) {
				state.ui.outputChannel?.error(String(error));
			}
		},
	);
	client.onRequest(AUTHORIZATION_METHOD, (params: AuthorizationParams) =>
		answerAuthorization(state, params),
	);
	return client;
};

const middleware = (
	state: ExtensionState,
	sendCommand: (command: string, args: unknown[]) => Promise<unknown>,
): Middleware => ({
	// Choosing "hide" hides the diagnostics too. The last
	// ones are kept so showing them again needs no new server work.
	handleDiagnostics: (uri, diagnostics, next) => {
		const key = uri.toString();
		if (diagnostics.length > 0) {
			state.diagnostics.set(key, { diagnostics, uri });
		} else {
			state.diagnostics.delete(key);
		}
		next(uri, state.flags.diagnosticsHidden ? [] : diagnostics);
	},
	provideCodeLenses: (document, token, next) => {
		if (!state.flags.showVersionLenses) {
			return [];
		}
		return next(document, token);
	},
	// The default handler turns a failure into a generic notification, so
	// send the request here and show the server's message once.
	executeCommand: async (command, args) => {
		let result: unknown;
		try {
			result = await sendCommand(command, args);
		} catch (error) {
			await reportError(state, `${command} failed`, error);
		}
		return result;
	},
	workspace: {
		configuration: async (params, token, next) => {
			const settings = await next(params, token);
			if (!Array.isArray(settings)) {
				return settings;
			}
			return settings.map((value: unknown, index): unknown => {
				if (params.items[index]?.section === "versionlens") {
					return withToggles(state, value);
				}
				return value;
			});
		},
	},
});

export { createLanguageClient };
