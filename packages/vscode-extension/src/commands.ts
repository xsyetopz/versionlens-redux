import {
	commands,
	type Disposable,
	type LogOutputChannel,
	type SecretStorage,
	Uri,
	window
} from "vscode";

import { parseStored } from "./authorization.ts";
import { LEVEL_COMMANDS } from "./levels.ts";

const SECRET = "versionlens.authorization";

/** The URL-prefix-to-header map kept in secret storage. */
export const storedAuthorization = async (
	secrets: SecretStorage
): Promise<Record<string, string>> => parseStored(await secrets.get(SECRET));

const ask = (prompt: string, password = false): Thenable<string | undefined> =>
	window.showInputBox({ ignoreFocusOut: true, password, prompt });

/** Asks for the Authorization header value: Basic from a user and password, or a token. */
const askHeader = async (): Promise<string | undefined> => {
	const kind = await window.showQuickPick(
		["Basic (user and password)", "Bearer token", "Header value"],
		{ placeHolder: "How the registry authenticates" }
	);
	if (kind === undefined) {
		return undefined;
	}
	if (kind.startsWith("Basic")) {
		const user = await ask("User name");
		const password =
			user === undefined ? undefined : await ask("Password", true);
		return user === undefined || password === undefined
			? undefined
			: `Basic ${Buffer.from(`${user}:${password}`).toString("base64")}`;
	}
	const value = await ask(
		kind === "Bearer token" ? "Token" : "Authorization header value",
		true
	);
	if (value === undefined || value.trim() === "") {
		return undefined;
	}
	return kind === "Bearer token" ? `Bearer ${value.trim()}` : value.trim();
};

const addAuthentication = async (secrets: SecretStorage): Promise<boolean> => {
	const url = await window.showInputBox({
		ignoreFocusOut: true,
		placeHolder: "https://npm.example.com/",
		prompt: "Registry URL prefix the header is sent to",
		validateInput: (text) =>
			/^https?:\/\/\S+$/u.test(text.trim())
				? undefined
				: "Enter an http or https URL"
	});
	const header = url === undefined ? undefined : await askHeader();
	if (url === undefined || header === undefined) {
		return false;
	}
	const stored = await storedAuthorization(secrets);
	stored[url.trim()] = header;
	await secrets.store(SECRET, JSON.stringify(stored));
	void window.showInformationMessage(
		`VersionLens: authentication saved for ${url.trim()}`
	);
	return true;
};

const removeAuthentication = async (
	secrets: SecretStorage
): Promise<boolean> => {
	const stored = await storedAuthorization(secrets);
	const urls = Object.keys(stored);
	if (urls.length === 0) {
		void window.showInformationMessage(
			"VersionLens: no stored url authentication"
		);
		return false;
	}
	const picked = await window.showQuickPick(urls, {
		canPickMany: true,
		placeHolder: "URLs to forget"
	});
	if (picked === undefined || picked.length === 0) {
		return false;
	}
	const kept = Object.fromEntries(
		Object.entries(stored).filter(([url]) => !picked.includes(url))
	);
	await secrets.store(SECRET, JSON.stringify(kept));
	return true;
};

/** What the client commands need from the extension. */
export interface Host {
	/** Sends the settings again after the stored authentication changed. */
	reconfigure: () => Promise<void>;
	restart: () => Promise<void>;
	secrets: SecretStorage;
	output: LogOutputChannel;
}

/**
 * Registers the commands the client runs itself: the per-level update commands the menus call,
 * url authentication, the error and progress icons, the output, and restart.
 */
export const registerCommands = (host: Host): Disposable[] => {
	const showOutput = (): void => {
		host.output.show();
	};
	const changed = async (saved: boolean): Promise<void> => {
		if (saved) {
			await host.reconfigure();
		}
	};
	return [
		commands.registerCommand("versionlens.showOutput", showOutput),
		commands.registerCommand("versionlens.showError", showOutput),
		commands.registerCommand("versionlens.progress", showOutput),
		commands.registerCommand("versionlens.restart", host.restart),
		commands.registerCommand("versionlens.addAuthentication", async () =>
			changed(await addAuthentication(host.secrets))
		),
		commands.registerCommand("versionlens.removeAuthentication", async () =>
			changed(await removeAuthentication(host.secrets))
		),
		...LEVEL_COMMANDS.map(({ id, level, server }) =>
			// Editor title menus pass the file's uri; the palette passes nothing.
			commands.registerCommand(id, (resource?: unknown) => {
				const uri =
					resource instanceof Uri
						? resource
						: window.activeTextEditor?.document.uri;
				return commands.executeCommand(
					server,
					...(uri === undefined ? [] : [uri.toString()]),
					level
				);
			})
		)
	];
};
