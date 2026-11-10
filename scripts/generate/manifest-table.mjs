// Reads the manifest table in `crates/versionlens-parsers`, which
// `versionlens-lsp --print-manifest-table` prints as JSON, and derives from it
// the lists that every generated file shares.
import { join } from "node:path";

import { field, items, parseJson, strings } from "../lib/json.mjs";
import { capture } from "../lib/spawn.mjs";

/**
 * @typedef {object} ManifestRow
 * @property {string} provider
 * @property {string[]} files
 * @property {string[]} globs
 * @property {string[]} names
 * @property {string[]} editorNames
 * @property {string[]} extensions
 * @property {string} format
 * @property {string[]} languages
 * @property {string[]} settingsProperties
 */

/** @typedef {"files" | "languages" | "settingsProperties"} RowListField */

/**
 * @typedef {object} EditorSupport
 * @property {string[]} names
 * @property {string[]} extensions
 * @property {string[]} suffixes
 * @property {[string, string][]} patterns
 */

const ROOT = join(import.meta.dir, "..", "..");

/** Extensions that a data format implies for editor pre-filters. */
const FORMAT_EXTENSIONS = new Map([
	["json", ["json"]],
	["toml", ["toml"]],
	["yaml", ["yaml", "yml"]],
	["xml", ["xml"]],
]);

/**
 * @param {unknown} value
 * @returns {ManifestRow}
 */
const toRow = (value) => {
	const provider = field(value, "provider");
	const format = field(value, "format");
	return {
		editorNames: strings(field(value, "editorNames")),
		extensions: strings(field(value, "extensions")),
		files: strings(field(value, "files")),
		format: typeof format === "string" ? format : "custom",
		globs: strings(field(value, "globs")),
		languages: strings(field(value, "languages")),
		names: strings(field(value, "names")),
		provider: typeof provider === "string" ? provider : "",
		settingsProperties: strings(field(value, "settingsProperties")),
	};
};

/** @returns {Promise<ManifestRow[]>} */
const readManifestTable = async () => {
	const binary = Bun.env["VERSIONLENS_LSP"];
	const command =
		binary === undefined || binary === ""
			? [
					"cargo",
					"run",
					"--locked",
					"-q",
					"-p",
					"versionlens-lsp",
					"--",
					"--print-manifest-table",
				]
			: [binary, "--print-manifest-table"];
	const result = await capture(command, { cwd: ROOT });
	if (result.exitCode !== 0) {
		throw new Error(
			`${command[0]} --print-manifest-table failed: ${result.stderr}`,
		);
	}
	return items(field(parseJson(result.stdout), "manifests")).map((row) =>
		toRow(row),
	);
};

/**
 * @param {string} a
 * @param {string} b
 * @returns {number}
 */
const ascii = (a, b) => {
	if (a < b) {
		return -1;
	}
	return a > b ? 1 : 0;
};

/**
 * @param {string} a
 * @param {string} b
 * @returns {number}
 */
const caseless = (a, b) => ascii(a.toLowerCase(), b.toLowerCase());

/**
 * @template T
 * @param {T[]} values
 * @returns {T[]}
 */
const unique = (values) => [...new Set(values)];

/**
 * @param {string[]} values
 * @returns {string[]}
 */
const sortedUnique = (values) => unique(values).toSorted(ascii);

/**
 * The ordered union of `field` over the rows of each provider.
 * @param {ManifestRow[]} manifests
 * @param {RowListField} listField
 * @returns {Map<string, string[]>}
 */
const perProvider = (manifests, listField) => {
	/** @type {Map<string, string[]>} */
	const providers = new Map();
	for (const row of manifests) {
		const list = providers.get(row.provider) ?? [];
		list.push(...row[listField]);
		providers.set(row.provider, list);
	}
	return new Map([...providers].map(([key, list]) => [key, unique(list)]));
};

/**
 * The `files` setting default for a list of globs.
 * @param {string[]} globs
 * @returns {string}
 */
const filesDefault = (globs) =>
	globs.length === 1 ? `**/${globs[0]}` : `**/{${globs.join(",")}}`;

/**
 * The `versionlens.*` settings that the table decides, keyed by setting.
 * @param {ManifestRow[]} manifests
 * @returns {Map<string, string | string[]>}
 */
const derivedSettings = (manifests) => {
	/** @type {Map<string, string | string[]>} */
	const settings = new Map();
	for (const [provider, globs] of perProvider(manifests, "files")) {
		if (globs.length > 0) {
			settings.set(`versionlens.${provider}.files`, filesDefault(globs));
		}
	}
	const properties = perProvider(manifests, "settingsProperties");
	for (const [provider, list] of properties) {
		if (list.length > 0) {
			settings.set(`versionlens.${provider}.dependencyProperties`, list);
		}
	}
	return settings;
};

/**
 * @param {ManifestRow[]} manifests
 * @returns {string[]}
 */
const languageIds = (manifests) =>
	unique(manifests.flatMap((row) => row.languages));

/**
 * @param {string} glob
 * @returns {boolean}
 */
const isPlainName = (glob) => !/[*/{}?]/u.test(glob);

/**
 * What a glob contributes to the editor pre-filters.
 * @param {string} glob
 * @returns {{ suffixes: string[], patterns: string[], extensions: string[] }}
 */
const classifyGlob = (glob) => {
	const contains = /^\*([^*/{}]+)\*\.([A-Za-z0-9]+)$/u.exec(glob);
	const extension = /^\*\.([A-Za-z0-9]+)$/u.exec(glob)?.[1];
	return {
		extensions: extension === undefined ? [] : [extension.toLowerCase()],
		patterns:
			contains?.[1] === undefined || contains[2] === undefined
				? []
				: [`${contains[1].toLowerCase()}.${contains[2].toLowerCase()}`],
		suffixes: /^[^*{}?]+\/[^*{}?]+$/u.test(glob) ? [glob] : [],
	};
};

/**
 * @param {string} pattern
 * @returns {[string, string]}
 */
const splitPattern = (pattern) => {
	const dot = pattern.lastIndexOf(".");
	return [pattern.slice(0, dot), pattern.slice(dot + 1)];
};

/**
 * Exact file names and extensions that the editor pre-filters accept.
 * @param {ManifestRow[]} manifests
 * @returns {EditorSupport}
 */
const editorSupport = (manifests) => {
	const globs = manifests.flatMap((row) =>
		[...row.globs, ...row.files].map((glob) => classifyGlob(glob)),
	);
	const names = manifests.flatMap((row) => [
		...row.names,
		...row.editorNames,
		...row.globs.filter((glob) => isPlainName(glob)),
	]);
	const extensions = manifests.flatMap((row) => [
		...row.extensions.map((extension) => extension.toLowerCase()),
		...(FORMAT_EXTENSIONS.get(row.format) ?? []),
	]);
	return {
		extensions: sortedUnique([
			...extensions,
			...globs.flatMap((glob) => glob.extensions),
		]),
		names: sortedUnique(names),
		patterns: sortedUnique(globs.flatMap((glob) => glob.patterns)).map(
			(pattern) => splitPattern(pattern),
		),
		suffixes: sortedUnique(globs.flatMap((glob) => glob.suffixes)),
	};
};

export {
	caseless,
	derivedSettings,
	editorSupport,
	filesDefault,
	languageIds,
	perProvider,
	readManifestTable,
	ROOT,
};
