// Reads the named lists of each generated file back out of its text, so that
// `--check` can say which entry drifted instead of only that the text differs.
import { field, items, parseJson, strings } from "../lib/json.mjs";
import { derivedSettings, perProvider } from "./manifest-table.mjs";

/** @typedef {Map<string, string[]>} Entries */

/**
 * @typedef {object} GeneratedFile
 * @property {string} path
 * @property {string} current
 * @property {string} next
 * @property {(text: string) => Entries} entriesOf
 */

const NEOVIM_VARIABLES = [
	"supported_file_names",
	"supported_extensions",
	"supported_path_suffixes",
	"supported_name_patterns",
];
const JETBRAINS_VARIABLES = [
	"supportedFileNames",
	"supportedExtensions",
	"supportedPathSuffixes",
	"supportedNamePatterns",
];

/**
 * The glob alternatives of a `files` default, without the leading globstar.
 * @param {unknown} value
 * @returns {string[]}
 */
const globsOf = (value) => {
	if (typeof value !== "string") {
		return [JSON.stringify(value)];
	}
	const bare = value.replace(/^\*\*\//u, "");
	const braced = /^\{(.*)\}$/u.exec(bare);
	return braced?.[1] === undefined ? [bare] : braced[1].split(",");
};

/**
 * @param {string | undefined} block
 * @returns {string[]}
 */
const lines = (block) =>
	block === undefined
		? []
		: block
				.split("\n")
				.map((line) => line.trim().replace(/,$/u, ""))
				.filter((line) => line !== "");

/**
 * @param {string} text
 * @param {string[]} variables
 * @param {(variable: string) => RegExp} pattern
 * @returns {Entries}
 */
const blockEntries = (text, variables, pattern) =>
	new Map(
		variables.map((variable) => [
			variable,
			lines(pattern(variable).exec(text)?.[1]),
		]),
	);

/**
 * @param {string} text
 * @param {import("./manifest-table.mjs").ManifestRow[]} manifests
 * @returns {Entries}
 */
const packageJsonEntries = (text, manifests) => {
	const json = parseJson(text);
	const properties = field(
		field(field(json, "contributes"), "configuration"),
		"properties",
	);
	/** @type {Entries} */
	const entries = new Map();
	for (const key of derivedSettings(manifests).keys()) {
		const value = field(field(properties, key), "default");
		entries.set(
			`${key} default`,
			Array.isArray(value) ? value.map(String) : globsOf(value),
		);
	}
	entries.set(
		"activationEvents onLanguage:*",
		strings(field(json, "activationEvents")).filter((event) =>
			event.startsWith("onLanguage:"),
		),
	);
	return entries;
};

/**
 * @param {string} text
 * @param {import("./manifest-table.mjs").ManifestRow[]} manifests
 * @returns {Entries}
 */
const configKeysEntries = (text, manifests) => {
	/** @type {Entries} */
	const entries = new Map();
	for (const provider of perProvider(manifests, "languages").keys()) {
		const row = new RegExp(
			`\\[\\s*"${provider}",\\s*"${provider}\\.files",\\s*(\\[[^\\]]*\\])`,
			"u",
		).exec(text);
		entries.set(
			`${provider} languages`,
			row?.[1] === undefined ? [] : strings(parseJson(row[1])),
		);
	}
	return entries;
};

/**
 * @param {string} text
 * @returns {Entries}
 */
const supportMatrixEntries = (text) => {
	/** @type {Entries} */
	const entries = new Map();
	for (const entry of items(field(parseJson(text), "entries"))) {
		const provider = String(field(entry, "provider"));
		for (const [index, manifest] of items(
			field(entry, "manifests"),
		).entries()) {
			const key = `${provider} manifests[${index}]`;
			entries.set(
				`${key} defaultGlob`,
				globsOf(field(manifest, "defaultGlob")),
			);
			entries.set(`${key} languages`, strings(field(manifest, "languages")));
		}
	}
	return entries;
};

/**
 * @param {string} text
 * @returns {Entries}
 */
const neovimEntries = (text) =>
	blockEntries(
		text,
		NEOVIM_VARIABLES,
		(variable) =>
			new RegExp(`local ${variable} = \\{\\n((?: {2}.*\\n)*)\\}\\n`, "u"),
	);

/**
 * @param {string} text
 * @returns {Entries}
 */
const neovimTargetEntries = (text) =>
	blockEntries(
		text,
		["triples"],
		() => /local triples = \{\n((?: {2}.*\n)*)\}\n/u,
	);

/**
 * @param {string} text
 * @returns {Entries}
 */
const jetbrainsEntries = (text) =>
	new Map([
		...blockEntries(
			text,
			JETBRAINS_VARIABLES,
			(variable) =>
				new RegExp(
					`private val ${variable} = (?:setOf|listOf<[^(]*>)\\(\\n((?: {12}.*\\n)*) {8}\\)`,
					"u",
				),
		),
		...blockEntries(
			text,
			["releaseTargets"],
			() => /private val releaseTargets = setOf\(\n((?: {12}.*\n)*) {8}\)/u,
		),
	]);

/**
 * @param {string} text
 * @returns {Entries}
 */
const zedEntries = (text) => {
	const languages = /languages = \[\n([\s\S]*?)\n\]/u.exec(text);
	const ids =
		/\[language_servers\.versionlens\.language_ids\]\n([\s\S]*)$/u.exec(text);
	return new Map([
		["languages", lines(languages?.[1])],
		["language_servers.versionlens.language_ids", lines(ids?.[1])],
	]);
};

/**
 * @param {string[]} list
 * @returns {string}
 */
const preview = (list) =>
	list.length > 6 ? `${list.slice(0, 6).join(", ")}, ...` : list.join(", ");

/**
 * @param {string[]} want
 * @param {string[]} have
 * @returns {string}
 */
const describeDifference = (want, have) => {
	const missing = want.filter((item) => !have.includes(item));
	const extra = have.filter((item) => !want.includes(item));
	if (missing.length + extra.length === 0) {
		return "order differs";
	}
	return [
		...(missing.length > 0 ? [`missing: ${preview(missing)}`] : []),
		...(extra.length > 0 ? [`unexpected: ${preview(extra)}`] : []),
	].join("; ");
};

/**
 * One message per generated entry that differs from the committed one.
 * @param {GeneratedFile} file
 * @returns {string[]}
 */
const findDrift = ({ path, current, next, entriesOf }) => {
	const actual = entriesOf(current);
	/** @type {string[]} */
	const drift = [];
	for (const [name, want] of entriesOf(next)) {
		const have = actual.get(name) ?? [];
		if (JSON.stringify(want) !== JSON.stringify(have)) {
			drift.push(
				`${path}: ${name}\n    ${describeDifference(want, have)}\n    expected: ${preview(want)}\n    actual:   ${preview(have)}`,
			);
		}
	}
	if (drift.length === 0 && next !== current) {
		drift.push(`${path}: formatting differs from the generated output`);
	}
	return drift;
};

export {
	configKeysEntries,
	findDrift,
	jetbrainsEntries,
	neovimEntries,
	neovimTargetEntries,
	packageJsonEntries,
	supportMatrixEntries,
	zedEntries,
};
