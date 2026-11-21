import { join } from "node:path";
import process from "node:process";

import {
	commands,
	type DocumentFilter,
	type ExtensionContext,
	languages,
	type LogOutputChannel,
	type TextDocument,
	window,
	workspace
} from "vscode";
import {
	DidChangeConfigurationNotification,
	LanguageClient,
	type LanguageClientOptions
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;
const manifestFilters = new WeakMap<LanguageClient, DocumentFilter[]>();

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

/**
 * Whether the server reads this document; other files never leave the editor.
 * The client opens already-visible documents inside `start()`, so the filters come from the
 * initialize result on first use rather than after `start()` resolves.
 */
const isManifest = (document: TextDocument): boolean => {
	if (client?.initializeResult === undefined) {
		return false;
	}
	let filters = manifestFilters.get(client);
	if (filters === undefined) {
		filters = advertisedGlobs(
			client.initializeResult.capabilities.experimental
		).map((glob) => ({ pattern: `**/${glob}`, scheme: "file" }));
		manifestFilters.set(client, filters);
	}
	return languages.match(filters, document) > 0;
};

const clientOptions = (output: LogOutputChannel): LanguageClientOptions => ({
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
		// Palette commands carry no arguments; the server acts on the active editor's file.
		executeCommand: async (name, args, next) => {
			const uri = window.activeTextEditor?.document.uri.toString();
			const result: unknown = await next(
				name,
				args.length === 0 && uri !== undefined ? [uri] : args
			);
			return result;
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
		clientOptions(output)
	);
	output.info(`starting ${command}`);
	client = started;
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
	await running?.stop();
};

/**
 * Starts the VersionLens language server for the manifests it supports.
 *
 * @param context Extension context that owns the commands and the output channel.
 */
export const activate = async (context: ExtensionContext): Promise<void> => {
	const output = window.createOutputChannel("VersionLens", { log: true });
	const restart = async (): Promise<void> => {
		await stop();
		await start(context, output);
	};
	context.subscriptions.push(
		output,
		commands.registerCommand("versionlens.showOutput", () => {
			output.show();
		}),
		commands.registerCommand("versionlens.restart", restart),
		workspace.onDidChangeConfiguration(async (event) => {
			if (event.affectsConfiguration("versionlens.serverPath")) {
				await restart();
			} else if (event.affectsConfiguration("versionlens")) {
				// The server answers an empty notification by pulling `workspace/configuration`.
				await client?.sendNotification(
					DidChangeConfigurationNotification.type,
					{ settings: null }
				);
			}
		})
	);
	await start(context, output);
};

/** Stops the language server. */
export const deactivate = (): Promise<void> => stop();
