import { expect, mock } from "bun:test";
import { join, resolve } from "node:path";

import { createExtensionState } from "../../src/extension/state.ts";
import type { Json } from "./lsp-client.ts";
import { isJson, LspConnection } from "./lsp-client.ts";

/**
 * Client tests against the real `versionlens-lsp` binary over stdio. The
 * connection answers what the VS Code client answers, using the client's own
 * authorization and settings code, and a local registry replaces the network.
 */

await mock.module("#vscode-host", () => ({
	TextDocumentChangeReason: undefined,
	commands: undefined,
	extensions: undefined,
	tasks: undefined,
	window: { activeTextEditor: undefined },
	workspace: undefined,
}));

const repositoryRoot = resolve(import.meta.dir, "../../../..");
const ASTRO = JSON.stringify({
	"dist-tags": { latest: "7.3.5" },
	versions: { "7.2.2": {}, "7.3.5": {} },
});
const MANIFEST = '{"dependencies":{"astro":"^7.2.2"}}';
const URI = "file:///workspace/astro/package.json";

/** The capabilities `vscode-languageclient` advertises by default. */
const clientCapabilities = {
	textDocument: {
		codeAction: {
			codeActionLiteralSupport: {
				codeActionKind: { valueSet: ["", "quickfix", "refactor", "source"] },
			},
			isPreferredSupport: true,
		},
		codeLens: { dynamicRegistration: true },
		publishDiagnostics: { relatedInformation: true },
		synchronization: { didSave: true },
	},
	workspace: {
		applyEdit: true,
		codeLens: { refreshSupport: true },
		configuration: true,
		didChangeWatchedFiles: { dynamicRegistration: true },
		workspaceEdit: { documentChanges: true },
		workspaceFolders: true,
	},
};

interface FakeRegistry {
	readonly authorization: (string | null)[];
	readonly url: string;
	stop: () => void;
}

/** Serves the astro package for any path; with `required`, 401 without it. */
const startRegistry = (required?: string): FakeRegistry => {
	const authorization: (string | null)[] = [];
	const server = Bun.serve({
		fetch(request): Response {
			const header = request.headers.get("authorization");
			authorization.push(header);
			if (required !== undefined && header !== required) {
				return new Response(null, { status: 401 });
			}
			return new Response(ASTRO, {
				headers: { "content-type": "application/json" },
			});
		},
		hostname: "127.0.0.1",
		port: 0,
	});
	return {
		authorization,
		stop: (): void => {
			void server.stop(true);
		},
		url: `http://127.0.0.1:${server.port}`,
	};
};

const serverBinary = (): string => {
	const override = process.env["VERSIONLENS_LSP"]?.trim();
	if (override !== undefined && override !== "") {
		return override;
	}
	const targetOverride = process.env["CARGO_TARGET_DIR"];
	const targetDirectory =
		targetOverride === undefined || targetOverride === ""
			? join(repositoryRoot, "target")
			: targetOverride;
	const name =
		process.platform === "win32" ? "versionlens-lsp.exe" : "versionlens-lsp";
	return resolve(targetDirectory, "debug", name);
};

/** Fails the suite early when the server binary was not built. */
const requireServerBinary = async (): Promise<void> => {
	const binary = serverBinary();
	if (!(await Bun.file(binary).exists())) {
		throw new Error(
			`the language server is not built (${binary}). Run \`cargo build -p versionlens-lsp\` or set VERSIONLENS_LSP.`,
		);
	}
};

interface Session {
	connection: LspConnection;
	/** The `versionlens` settings the server was handed, per request. */
	configurations: unknown[];
	registry: FakeRegistry;
}

const cleanups: (() => Promise<void> | void)[] = [];

/** Runs, and forgets, what the sessions registered to tear down. */
const runCleanups = async (): Promise<void> => {
	await Promise.all(
		cleanups.splice(0).map(async (cleanup): Promise<void> => {
			await cleanup();
		}),
	);
};

/** The `section` of each configuration item the server asked for. */
const requestedSections = (params: Json): (string | undefined)[] => {
	const { items } = params;
	if (!Array.isArray(items)) {
		return [];
	}
	return items.map((item: unknown): string | undefined =>
		isJson(item) && typeof item["section"] === "string"
			? item["section"]
			: undefined,
	);
};

const startSession = async (
	options: {
		registry?: FakeRegistry;
		flags?: { showPrereleases: boolean };
		authorization?: (params: Json) => unknown;
	} = {},
): Promise<Session> => {
	const { withToggles } = await import("../../src/extension/config/toggles.ts");
	const registry = options.registry ?? startRegistry();
	const connection = await LspConnection.start(serverBinary());
	const configurations: unknown[] = [];
	const state = createExtensionState();
	state.flags.showPrereleases = options.flags?.showPrereleases ?? false;
	cleanups.push(async (): Promise<void> => {
		try {
			await connection.shutdown();
		} finally {
			registry.stop();
		}
	});
	// What the client's `workspace.configuration` middleware does.
	connection.onRequest("workspace/configuration", (params: Json): unknown[] =>
		requestedSections(params).map((section): unknown => {
			if (section !== "versionlens") {
				return null;
			}
			const value = withToggles(state, {
				npm: { apiUrl: registry.url },
				suggestions: { showVulnerabilities: false },
			});
			configurations.push(value);
			return value;
		}),
	);
	connection.onRequest("client/registerCapability", (): null => null);
	connection.onRequest("client/unregisterCapability", (): null => null);
	if (options.authorization !== undefined) {
		connection.onRequest("versionlens/authorization", options.authorization);
	}
	await connection.request("initialize", {
		capabilities: clientCapabilities,
		processId: process.pid,
		rootUri: "file:///workspace",
		workspaceFolders: [{ name: "workspace", uri: "file:///workspace" }],
	});
	connection.notify("initialized", {});
	return { configurations, connection, registry };
};

const open = (connection: LspConnection, text = MANIFEST): void => {
	connection.notify("textDocument/didOpen", {
		textDocument: { languageId: "json", text, uri: URI, version: 1 },
	});
};

const hasStatus = (params: Json, status: string): boolean =>
	params["uri"] === URI && params["status"] === status;

/** Waits until the server reports the document idle after being busy. */
const settled = async (connection: LspConnection): Promise<void> => {
	await connection.waitFor("versionlens/status", (params): boolean =>
		hasStatus(params, "idle"),
	);
};

const publishedDiagnostics = (params: Json): unknown[] => {
	const list = params["diagnostics"];
	return Array.isArray(list) ? list : [];
};

const hasDiagnostics = (params: Json): boolean =>
	params["uri"] === URI && publishedDiagnostics(params).length > 0;

const diagnostics = async (connection: LspConnection): Promise<unknown[]> => {
	const published = await connection.waitFor(
		"textDocument/publishDiagnostics",
		hasDiagnostics,
	);
	return publishedDiagnostics(published);
};

const hasPublishedDiagnostics = (connection: LspConnection): boolean =>
	connection.notifications.some(
		(entry): boolean =>
			entry.method === "textDocument/publishDiagnostics" &&
			hasDiagnostics(entry.params),
	);

/** Asymmetric matchers, typed `unknown` so they can sit in expected values. */
const objectContaining = (shape: Json): unknown =>
	expect.objectContaining(shape);
const stringContaining = (text: string): unknown =>
	expect.stringContaining(text);
const stringMatching = (pattern: RegExp): unknown =>
	expect.stringMatching(pattern);

export type { FakeRegistry };
export {
	diagnostics,
	hasDiagnostics,
	hasPublishedDiagnostics,
	hasStatus,
	MANIFEST,
	objectContaining,
	open,
	requireServerBinary,
	runCleanups,
	settled,
	startRegistry,
	startSession,
	stringContaining,
	stringMatching,
	URI,
};
