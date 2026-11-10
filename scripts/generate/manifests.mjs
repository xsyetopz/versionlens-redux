// Regenerates the editor file lists from the manifest table in
// `crates/versionlens-parsers`. Everything that is not derived from the table
// stays as is. `--check` reports drift instead of writing.
import { join } from "node:path";

import {
	CONFIG_KEYS,
	formatLikeRepo,
	generateConfigKeys,
	generateSupportMatrix,
	SUPPORT_MATRIX,
} from "./config-files.mjs";
import {
	configKeysEntries,
	findDrift,
	jetbrainsEntries,
	neovimEntries,
	neovimTargetEntries,
	packageJsonEntries,
	supportMatrixEntries,
	zedEntries,
} from "./drift.mjs";
import {
	generateJetBrains,
	generateJetBrainsTargets,
	generateNeovim,
	generateNeovimTargets,
	generateZed,
	JETBRAINS_SUPPORT,
	NEOVIM_DOWNLOAD,
	NEOVIM_SUPPORT,
	ZED_EXTENSION,
} from "./editor-lists.mjs";
import { readManifestTable, ROOT } from "./manifest-table.mjs";
import { generatePackageJson, PACKAGE_JSON } from "./package-json.mjs";

/** @typedef {import("./manifest-table.mjs").ManifestRow[]} Manifests */
/** @typedef {import("./drift.mjs").Entries} Entries */
/** @typedef {import("./drift.mjs").GeneratedFile} GeneratedFile */

/**
 * @typedef {object} Target
 * @property {string} path
 * @property {(text: string, manifests: Manifests) => string | Promise<string>} generate
 * @property {(text: string, manifests: Manifests) => Entries} entriesOf
 */

/** @type {Target[]} */
const targets = [
	{
		entriesOf: packageJsonEntries,
		generate: (text, manifests) =>
			formatLikeRepo(PACKAGE_JSON, generatePackageJson(text, manifests)),
		path: PACKAGE_JSON,
	},
	{
		entriesOf: neovimEntries,
		generate: generateNeovim,
		path: NEOVIM_SUPPORT,
	},
	{
		entriesOf: neovimTargetEntries,
		generate: generateNeovimTargets,
		path: NEOVIM_DOWNLOAD,
	},
	{
		entriesOf: (text) => new Map(jetbrainsEntries(text)),
		generate: (text, manifests) =>
			generateJetBrainsTargets(generateJetBrains(text, manifests)),
		path: JETBRAINS_SUPPORT,
	},
	{ entriesOf: zedEntries, generate: generateZed, path: ZED_EXTENSION },
	{
		entriesOf: configKeysEntries,
		generate: generateConfigKeys,
		path: CONFIG_KEYS,
	},
	{
		entriesOf: supportMatrixEntries,
		generate: generateSupportMatrix,
		path: SUPPORT_MATRIX,
	},
];

/**
 * @param {Manifests} manifests
 * @returns {Promise<GeneratedFile[]>}
 */
const generateAll = (manifests) =>
	Promise.all(
		targets.map(async ({ path, generate, entriesOf }) => {
			const current = await Bun.file(join(ROOT, path)).text();
			/** @type {GeneratedFile} */
			const file = {
				current,
				entriesOf: (text) => entriesOf(text, manifests),
				next: await generate(current, manifests),
				path,
			};
			return file;
		}),
	);

/**
 * @param {GeneratedFile[]} results
 * @returns {void}
 */
const check = (results) => {
	const drift = results.flatMap((file) => findDrift(file));
	if (drift.length > 0) {
		console.error(
			`generated lists drifted from the manifest table:\n${drift.join("\n")}\nrun \`bun scripts/generate/manifests.mjs\` to regenerate`,
		);
		process.exitCode = 1;
		return;
	}
	console.log(
		`manifests: ${results.length} generated files match the manifest table`,
	);
};

/**
 * @param {GeneratedFile[]} results
 * @returns {Promise<void>}
 */
const write = async (results) => {
	const changed = results.filter(({ current, next }) => next !== current);
	await Promise.all(
		changed.map(({ path, next }) => Bun.write(join(ROOT, path), next)),
	);
	for (const { path } of changed) {
		console.log(`updated ${path}`);
	}
};

const main = async () => {
	try {
		const results = await generateAll(await readManifestTable());
		if (Bun.argv.includes("--check")) {
			check(results);
		} else {
			await write(results);
		}
	} catch (error) {
		console.error(
			`manifests: ${error instanceof Error ? error.message : String(error)}`,
		);
		process.exitCode = 1;
	}
};

if (import.meta.main) {
	await main();
}
