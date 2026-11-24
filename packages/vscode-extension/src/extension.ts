import { join } from "node:path";
import process from "node:process";

import {
	commands,
	type ExtensionContext,
	type LogOutputChannel,
	type TextDocument,
	type TextEditor,
	Uri,
	window,
	workspace
} from "vscode";
import {
	DidChangeConfigurationNotification,
	LanguageClient,
	type LanguageClientOptions,
	type Middleware
} from "vscode-languageclient/node";

import { withAuthorization } from "./authorization.ts";
import { registerCommands, storedAuthorization } from "./commands.ts";
import { globRegex, matchesAny } from "./globs.ts";
import { contextKeys, parseStatus, type Status } from "./status.ts";

let client: LanguageClient | undefined;
const manifestGlobs = new WeakMap<LanguageClient, RegExp[]>();
/** The latest `versionlens/status` per manifest uri. */
const statuses = new Map<string, Status>();

const nonEmpty = (value: string | undefined): string | undefined => {
	const trimmed = value?.trim();
	return trimmed === undefined || trimmed === "" ? undefined : trimmed;
};

/** The server binary: `VERSIONLENS_LSP`, then `versionlens.serverPath`, then the bundled one. */
const serverPath = (context: ExtensionContext): string =>
	nonEmpty(process.env["VERSIONLENS_LSP"]) ??
	nonEmpty(
		workspace.getConfiguration("versionlens").get<string>("serverPath")
	) ??
	join(
		context.extensionPath,
		"server",
		process.platform === "win32" ? "versionlens-lsp.exe" : "versionlens-lsp"
	);

/** The globs the server advertises in `capabilities.experimental.fileTypes`. */
const advertisedGlobs = (experimental: unknown): string[] => {
	if (typeof experimental !== "object" || experimental === null) {
		return [];
	}
	const fileTypes: unknown = Reflect.get(experimental, "fileTypes");
	return Array.isArray(fileTypes)
		? fileTypes.filter((glob): glob is string => typeof glob === "string")
		: [];
};

/** The globs of the `versionlens.files` setting, which the server reads as extra manifests. */
const configuredGlobs = (): RegExp[] =>
	Object.keys(
		workspace
			.getConfiguration("versionlens")
			.get<Record<string, string>>("files") ?? {}
	).map((glob) => globRegex(glob));

/** Sets the editor title menu context keys from the active editor's manifest status. */
const showStatus = (editor?: TextEditor): void => {
	const status =
		editor === undefined
			? undefined
			: statuses.get(editor.document.uri.toString());
	for (const [key, value] of Object.entries(contextKeys(status))) {
		void commands.executeCommand("setContext", key, value);
	}
};

/**
 * Whether the server reads this document; other files never leave the editor.
 * The client opens already-visible documents inside `start()`, so the filters come from the
 * initialize result on first use rather than after `start()` resolves.
 */
const isManifest = (document: TextDocument): boolean => {
	if (client?.initializeResult === undefined) {
		return false;
	}
	let globs = manifestGlobs.get(client);
	if (globs === undefined) {
		globs = advertisedGlobs(
			client.initializeResult.capabilities.experimental
		).map((glob) => globRegex(glob));
		manifestGlobs.set(client, globs);
	}
	// VS Code's own glob matching is case-sensitive; the server's, and the original's, are not.
	return (
		document.uri.scheme === "file" &&
		matchesAny([...globs, ...configuredGlobs()], document.uri.fsPath)
	);
};

// Menus pass the file's Uri and the palette passes nothing; the server reads uri strings and
// acts on the active editor's file when none is given.
const withActiveUri: Middleware["executeCommand"] = async (
	name,
	args,
	next
) => {
	const strings: unknown[] = args.map((arg: unknown) =>
		arg instanceof Uri ? arg.toString() : arg
	);
	const active = window.activeTextEditor?.document.uri.toString();
	const hasUri = strings.some(
		(arg) => typeof arg === "string" && arg.includes(":")
	);
	const result: unknown = await next(
		name,
		hasUri || active === undefined ? strings : [active, ...strings]
	);
	return result;
};

// The server answers an empty notification by pulling `workspace/configuration`.
const reconfigure = async (): Promise<void> => {
	await client?.sendNotification(DidChangeConfigurationNotification.type, {
		settings: null
	});
};

const clientOptions = (
	context: ExtensionContext,
	output: LogOutputChannel
): LanguageClientOptions => ({
	documentSelector: [{ scheme: "file" }],
	middleware: {
		didChange: async (event, next) => {
			if (isManifest(event.document)) {
				await next(event);
			}
		},
		didOpen: async (document, next) => {
			if (isManifest(document)) {
				await next(document);
			}
		},
		executeCommand: withActiveUri,
		// Authentication added with the "Add url authentication" command lives in secret
		// storage, not in settings.json, and joins the section here.
		workspace: {
			configuration: async (params, token, next) => {
				const result = await next(params, token);
				if (!Array.isArray(result)) {
					return result;
				}
				const stored = await storedAuthorization(context.secrets);
				return result.map((section: unknown, index: number) =>
					params.items[index]?.section === "versionlens"
						? withAuthorization(section, stored)
						: section
				);
			}
		}
	},
	outputChannel: output
});

const start = async (
	context: ExtensionContext,
	output: LogOutputChannel
): Promise<void> => {
	const command = serverPath(context);
	const started = new LanguageClient(
		"versionlens",
		"VersionLens",
		{ args: ["--stdio"], command },
		clientOptions(context, output)
	);
	output.info(`starting ${command}`);
	client = started;
	started.onNotification("versionlens/status", (params: unknown) => {
		const status = parseStatus(params);
		if (status === undefined) {
			return;
		}
		statuses.set(status.uri, status);
		if (window.activeTextEditor?.document.uri.toString() === status.uri) {
			showStatus(window.activeTextEditor);
		}
	});
	try {
		await started.start();
	} catch (error) {
		client = undefined;
		output.error(`cannot start ${command}: ${String(error)}`);
		void window.showErrorMessage(
			`VersionLens cannot start its server (${command}). Set versionlens.serverPath or reinstall the extension.`
		);
		return;
	}
	const globs = advertisedGlobs(
		started.initializeResult?.capabilities.experimental
	);
	output.info(`server reads ${globs.length} file globs`);
};

const stop = async (): Promise<void> => {
	const running = client;
	client = undefined;
	statuses.clear();
	showStatus();
	await running?.stop();
};

/** What the extension exports to other extensions and to the live test. */
export interface Api {
	/** The editor title menu context keys for a file uri, from the server's latest status. */
	contextKeys: (uri: string) => Record<string, boolean>;
}

/**
 * Starts the VersionLens language server for the manifests it supports.
 *
 * @param context Extension context that owns the commands and the output channel.
 * @returns The context keys per file, as the editor title menus see them.
 */
export const activate = async (context: ExtensionContext): Promise<Api> => {
	const output = window.createOutputChannel("VersionLens", { log: true });
	const restart = async (): Promise<void> => {
		await stop();
		await start(context, output);
	};
	context.subscriptions.push(
		output,
		...registerCommands({
			output,
			reconfigure,
			restart,
			secrets: context.secrets
		}),
		window.onDidChangeActiveTextEditor(showStatus),
		workspace.onDidCloseTextDocument((document) => {
			statuses.delete(document.uri.toString());
		}),
		workspace.onDidChangeConfiguration(async (event) => {
			// New `files` globs change which open documents reach the server.
			if (
				event.affectsConfiguration("versionlens.serverPath") ||
				event.affectsConfiguration("versionlens.files")
			) {
				await restart();
			} else if (event.affectsConfiguration("versionlens")) {
				await reconfigure();
			}
		})
	);
	await start(context, output);
	return { contextKeys: (uri) => contextKeys(statuses.get(uri)) };
};

/** Stops the language server. */
export const deactivate = (): Promise<void> => stop();
