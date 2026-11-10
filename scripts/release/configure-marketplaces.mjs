#!/usr/bin/env bun
import process from "node:process";

import { sequentially } from "../lib/sequence.mjs";
import { reportFailure, requireCommand, run } from "./gh.mjs";
import { collectSecrets } from "./marketplace-secrets.mjs";
import { createPrompt } from "./prompt.mjs";

const environment = "marketplaces";
const dryRun = process.argv.includes("--dry-run");
const repoArgumentIndex = process.argv.indexOf("--repo");
const onlyArgumentIndex = process.argv.indexOf("--only");
const marketplaceIds = ["vscode", "jetbrains", "zed", "neovim"];
const marketplaceLabels = new Map([
	["vscode", "VS Code Marketplace"],
	["jetbrains", "JetBrains Marketplace"],
	["zed", "Zed extension registry"],
	["neovim", "Neovim via LuaRocks"],
]);

/** @returns {Set<string> | null} */
const requestedMarketplaces = () => {
	if (onlyArgumentIndex === -1) {
		return null;
	}
	const value = process.argv[onlyArgumentIndex + 1];
	if (value === undefined || value === "") {
		throw new Error(
			`--only requires one or more of: ${marketplaceIds.join(", ")}`,
		);
	}
	const selected = new Set(
		value
			.split(",")
			.map((entry) => entry.trim().toLowerCase())
			.filter((entry) => entry !== ""),
	);
	const unknown = [...selected].filter(
		(entry) => !marketplaceLabels.has(entry),
	);
	if (unknown.length > 0) {
		throw new Error(
			`unknown marketplace ${unknown.join(", ")}; expected: ${marketplaceIds.join(", ")}`,
		);
	}
	return selected;
};

/**
 * @param {(prompt: string) => Promise<string>} ask
 * @returns {Promise<Set<string>>}
 */
const chooseMarketplaces = async (ask) => {
	const requested = requestedMarketplaces();
	if (requested !== null) {
		return requested;
	}
	if (dryRun) {
		return new Set(marketplaceIds);
	}

	console.log(
		"\nSelect the destinations whose provider accounts are ready. Skipped destinations can be configured by running this command again later.",
	);
	/** @type {Set<string>} */
	const selected = new Set();
	await sequentially(marketplaceIds, async (id) => {
		const label = marketplaceLabels.get(id);
		const reply = await ask(`Configure ${label}? [y/N]: `);
		const answer = reply.trim().toLowerCase();
		if (answer === "y" || answer === "yes") {
			selected.add(id);
		} else if (answer !== "" && answer !== "n" && answer !== "no") {
			throw new Error(`expected yes or no for ${label}`);
		}
	});
	return selected;
};

/**
 * @param {string} repo
 * @returns {Promise<Set<string>>}
 */
const existingSecretNames = async (repo) => {
	if (dryRun) {
		return new Set();
	}
	const output = await run(
		[
			"gh",
			"secret",
			"list",
			"--repo",
			repo,
			"--env",
			environment,
			"--json",
			"name",
			"--jq",
			".[].name",
		],
		{ capture: true },
	);
	return new Set(output === "" ? [] : output.split("\n"));
};

/**
 * @param {string} repo
 * @param {string} name
 * @param {string | Uint8Array} value
 * @returns {Promise<void>}
 */
const setSecret = async (repo, name, value) => {
	if (dryRun) {
		console.log(`${environment}: ${name}`);
		return;
	}
	const bytes =
		typeof value === "string" ? new TextEncoder().encode(value) : value;
	await run(
		["gh", "secret", "set", name, "--repo", repo, "--env", environment],
		{ stdin: bytes },
	);
};

/** @returns {Promise<string>} */
const resolveRepository = async () => {
	const requestedRepo =
		repoArgumentIndex === -1 ? undefined : process.argv[repoArgumentIndex + 1];
	const repo =
		requestedRepo ??
		(await run(
			[
				"gh",
				"repo",
				"view",
				"--json",
				"nameWithOwner",
				"--jq",
				".nameWithOwner",
			],
			{ capture: true },
		));
	if (!/^[^/]+\/[^/]+$/u.test(repo)) {
		throw new Error(
			`could not determine a GitHub owner/repository, got ${repo}`,
		);
	}
	return repo;
};

/**
 * @param {string} repo
 * @param {Set<string>} selected
 * @param {(prompt: string, options?: { hidden?: boolean }) => Promise<string>} ask
 * @returns {Promise<void>}
 */
const configureSelected = async (repo, selected, ask) => {
	console.log(`Selected marketplaces: ${[...selected].join(", ")}`);

	if (dryRun) {
		console.log(`Repository: ${repo}`);
	} else {
		await run(
			[
				"gh",
				"api",
				"--method",
				"PUT",
				`repos/${repo}/environments/${environment}`,
			],
			{ quiet: true },
		);
	}
	/** @type {{ name: string, value: string | Uint8Array }[]} */
	const pendingSecrets = [];
	await collectSecrets(
		{
			ask,
			dryRun,
			existingSecrets: await existingSecretNames(repo),
			pendingSecrets,
		},
		selected,
	);

	if (pendingSecrets.length === 0) {
		console.log("\nNo secret updates requested; existing values were kept.");
		return;
	}
	console.log(
		`\nValidated ${pendingSecrets.length} secret value${pendingSecrets.length === 1 ? "" : "s"}. Applying updates to GitHub...`,
	);
	await sequentially(pendingSecrets, ({ name, value }) =>
		setSecret(repo, name, value),
	);
	console.log(
		`\nConfigured GitHub environment secrets in ${repo}/${environment}.`,
	);
};

const configure = async () => {
	await requireCommand("gh");
	await run(["gh", "auth", "status"]);
	const repo = await resolveRepository();
	const prompt = createPrompt();
	try {
		const selected = await chooseMarketplaces(prompt.ask);
		if (selected.size === 0) {
			console.log(
				"No marketplaces selected; no GitHub environment changes were made.",
			);
			return;
		}
		await configureSelected(repo, selected, prompt.ask);
	} finally {
		prompt.close();
	}
};

try {
	await configure();
} catch (error) {
	reportFailure(error);
}
