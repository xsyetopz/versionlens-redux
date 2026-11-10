import { homedir } from "node:os";
import { resolve } from "node:path";

import { sequentially } from "../lib/sequence.mjs";

/**
 * @param {string} value
 * @param {string} [home]
 * @returns {string}
 */
const expandUserPath = (value, home = homedir()) => {
	if (value === "~") {
		return home;
	}
	if (value.startsWith("~/") || value.startsWith("~\\")) {
		return resolve(home, value.slice(2));
	}
	if (value.startsWith("~")) {
		throw new Error(
			"paths for another user's home directory are not supported",
		);
	}
	return resolve(value);
};

/**
 * @param {string} value
 * @param {string} [home]
 * @returns {Promise<{ contents: Uint8Array, path: string }>}
 */
const readSecretFile = async (value, home = homedir()) => {
	const path = expandUserPath(value, home);
	const contents = await Bun.file(path).bytes();
	if (contents.length === 0) {
		throw new Error(`${path} is empty`);
	}
	return { contents, path };
};

/**
 * @typedef {object} SecretContext
 * @property {(prompt: string, options?: { hidden?: boolean }) => Promise<string>} ask
 * @property {boolean} dryRun
 * @property {Set<string>} existingSecrets
 * @property {{ name: string, value: string | Uint8Array }[]} pendingSecrets
 */

/**
 * @param {SecretContext} context
 * @param {string} name
 * @returns {Promise<boolean>}
 */
const replaceExistingSecret = async ({ ask, existingSecrets }, name) => {
	if (!existingSecrets.has(name)) {
		return true;
	}
	const reply = await ask(`${name} is already configured. Replace it? [y/N]: `);
	const answer = reply.trim().toLowerCase();
	if (answer === "y" || answer === "yes") {
		return true;
	}
	if (answer === "" || answer === "n" || answer === "no") {
		console.log(`Keeping existing ${name}.`);
		return false;
	}
	throw new Error(`expected yes or no for ${name}`);
};

/**
 * Ask whether to collect `name` at all: a dry run queues a placeholder, and an
 * existing secret is kept unless the user agrees to replace it.
 * @param {SecretContext} context
 * @param {string} name
 * @returns {Promise<boolean>}
 */
const shouldAsk = (context, name) => {
	if (context.dryRun) {
		context.pendingSecrets.push({ name, value: "dry-run" });
		return Promise.resolve(false);
	}
	return replaceExistingSecret(context, name);
};

/**
 * @param {SecretContext} context
 * @param {string} name
 * @param {string} prompt
 * @returns {Promise<void>}
 */
const visibleSecret = async (context, name, prompt) => {
	if (!(await shouldAsk(context, name))) {
		return;
	}
	const reply = await context.ask(`${prompt}: `, { hidden: true });
	const value = reply.trim();
	if (value === "") {
		throw new Error(`${name} cannot be empty`);
	}
	context.pendingSecrets.push({ name, value });
};

/**
 * @param {SecretContext} context
 * @param {string} name
 * @param {string} prompt
 * @returns {Promise<void>}
 */
const hiddenSecret = async (context, name, prompt) => {
	if (!(await shouldAsk(context, name))) {
		return;
	}
	const value = await context.ask(`${prompt}: `, { hidden: true });
	if (value.length === 0) {
		throw new Error(`${name} cannot be empty`);
	}
	context.pendingSecrets.push({ name, value });
};

/**
 * @param {SecretContext} context
 * @param {string} name
 * @param {string} prompt
 * @returns {Promise<void>}
 */
const askForFile = async (context, name, prompt) => {
	const reply = await context.ask(`${prompt} file path: `);
	const value = reply.trim();
	if (value === "") {
		throw new Error(`${name} file path cannot be empty`);
	}
	try {
		const { contents, path } = await readSecretFile(value);
		console.log(`Validated ${name}: ${path}`);
		context.pendingSecrets.push({ name, value: contents });
	} catch (error) {
		console.error(
			`Could not read ${name}: ${error instanceof Error ? error.message : String(error)}`,
		);
		console.error(
			"Enter another path, or press Ctrl-C to leave all secrets unchanged.",
		);
		await askForFile(context, name, prompt);
	}
};

/**
 * @param {SecretContext} context
 * @param {string} name
 * @param {string} prompt
 * @returns {Promise<void>}
 */
const fileSecret = async (context, name, prompt) => {
	if (await shouldAsk(context, name)) {
		await askForFile(context, name, prompt);
	}
};

/** @type {Record<string, (context: SecretContext) => Promise<void>>} */
const collectors = {
	vscode: async (context) => {
		console.log("\nVS Code Marketplace (Microsoft Entra workload identity)");
		await visibleSecret(
			context,
			"AZURE_CLIENT_ID",
			"Azure managed identity or app client ID",
		);
		await visibleSecret(context, "AZURE_TENANT_ID", "Azure tenant ID");
	},
	jetbrains: async (context) => {
		console.log("\nJetBrains Marketplace");
		await hiddenSecret(
			context,
			"JETBRAINS_MARKETPLACE_TOKEN",
			"JetBrains permanent Marketplace token",
		);
		await fileSecret(
			context,
			"JB_CERTIFICATE_CHAIN",
			"JetBrains signing certificate chain PEM",
		);
		await fileSecret(
			context,
			"JB_PRIVATE_KEY",
			"JetBrains signing private key PEM",
		);
		await hiddenSecret(
			context,
			"JB_PRIVATE_KEY_PASSWORD",
			"JetBrains signing private key password",
		);
	},
	zed: async (context) => {
		console.log("\nZed extension registry");
		await visibleSecret(
			context,
			"ZED_EXTENSIONS_FORK",
			"Fork of zed-industries/extensions (owner/extensions)",
		);
		await hiddenSecret(
			context,
			"ZED_EXTENSIONS_TOKEN",
			"GitHub token that can push to the fork and open a pull request against zed-industries/extensions",
		);
	},
	neovim: async (context) => {
		console.log("\nNeovim via LuaRocks");
		await hiddenSecret(context, "LUAROCKS_API_KEY", "LuaRocks API key");
	},
};

/**
 * Prompt for the secrets of every selected marketplace and queue them, in the
 * order the marketplaces are declared.
 * @param {SecretContext} context
 * @param {Set<string>} selected
 * @returns {Promise<void>}
 */
const collectSecrets = async (context, selected) => {
	const ids = Object.keys(collectors).filter((id) => selected.has(id));
	await sequentially(
		ids,
		(id) => collectors[id]?.(context) ?? Promise.resolve(),
	);
};

export { collectSecrets, expandUserPath, readSecretFile };
