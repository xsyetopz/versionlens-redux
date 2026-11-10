import { field, items, parseJson } from "../lib/json.mjs";
import { capture } from "../lib/spawn.mjs";
import { filesDefault, perProvider, ROOT } from "./manifest-table.mjs";

const CONFIG_KEYS = "packages/vscode-extension/src/extension/config/keys.ts";
const SUPPORT_MATRIX = "tests/fixtures/manifest-support-matrix.json";

/**
 * Formats generated text as `bun fmt` formats the file at `path`.
 * @param {string} path
 * @param {string} text
 * @returns {Promise<string>}
 */
const formatLikeRepo = async (path, text) => {
	const result = await capture(
		[process.execPath, "fmt", `--stdin-filepath=${path}`],
		{ cwd: ROOT, stdin: new TextEncoder().encode(text) },
	);
	if (result.exitCode !== 0) {
		throw new Error(`bun fmt could not format ${path}: ${result.stderr}`);
	}
	return result.stdout;
};

/**
 * The CodeLens document selector languages of each provider.
 * @param {string} text
 * @param {import("./manifest-table.mjs").ManifestRow[]} manifests
 * @returns {Promise<string>}
 */
const generateConfigKeys = (text, manifests) => {
	const start = text.indexOf("const filePatternKeys");
	const end = text.indexOf("const enabledFilePatternKeys");
	if (start === -1 || end < start) {
		throw new Error(`${CONFIG_KEYS} lacks the filePatternKeys table`);
	}
	let block = text.slice(start, end);
	for (const [provider, languages] of perProvider(manifests, "languages")) {
		if (languages.length === 0) {
			continue;
		}
		const pattern = new RegExp(
			`(\\[\\s*"${provider}",\\s*"${provider}\\.files",\\s*)\\[[^\\]]*\\]`,
			"u",
		);
		const match = pattern.exec(block);
		if (match === null) {
			throw new Error(
				`${CONFIG_KEYS} has no filePatternKeys row for ${provider}`,
			);
		}
		const list = languages.map((language) => JSON.stringify(language));
		block = `${block.slice(0, match.index)}${match[1] ?? ""}[${list.join(", ")}]${block.slice(match.index + match[0].length)}`;
	}
	return formatLikeRepo(
		CONFIG_KEYS,
		text.slice(0, start) + block + text.slice(end),
	);
};

/**
 * The `defaultGlob` and `languages` of each provider in the support matrix.
 * @param {string} text
 * @param {import("./manifest-table.mjs").ManifestRow[]} manifests
 * @returns {Promise<string>}
 */
const generateSupportMatrix = (text, manifests) => {
	const languages = perProvider(manifests, "languages");
	const globs = perProvider(manifests, "files");
	const matrix = parseJson(text);
	const entries = items(field(matrix, "entries"));
	const listed = new Set(entries.map((entry) => field(entry, "provider")));
	for (const [provider, files] of globs) {
		if (files.length > 0 && !listed.has(provider)) {
			throw new Error(
				`${SUPPORT_MATRIX}: no entry for ${provider}; add it by hand, then rerun`,
			);
		}
	}
	for (const entry of entries) {
		const provider = String(field(entry, "provider"));
		const providerGlobs = globs.get(provider);
		const providerLanguages = languages.get(provider);
		for (const manifest of items(field(entry, "manifests"))) {
			if (
				providerGlobs === undefined ||
				providerGlobs.length === 0 ||
				providerLanguages === undefined ||
				typeof manifest !== "object" ||
				manifest === null
			) {
				throw new Error(
					`${SUPPORT_MATRIX}: the table has no row for ${provider}`,
				);
			}
			Object.assign(manifest, {
				defaultGlob: filesDefault(providerGlobs),
				languages: providerLanguages,
			});
		}
	}
	// tests/fixtures is outside bun fmt (.prettierignore), so this layout is final.
	return Promise.resolve(`${JSON.stringify(matrix, null, 2)}\n`);
};

export {
	CONFIG_KEYS,
	formatLikeRepo,
	generateConfigKeys,
	generateSupportMatrix,
	SUPPORT_MATRIX,
};
