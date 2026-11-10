import { items, parseJson } from "../lib/json.mjs";
import { derivedSettings, languageIds } from "./manifest-table.mjs";

const PACKAGE_JSON = "packages/vscode-extension/package.json";

/**
 * @param {string} text
 * @param {number} index
 * @returns {string}
 */
const indentOf = (text, index) => {
	const start = text.lastIndexOf("\n", index) + 1;
	return /^[ \t]*/u.exec(text.slice(start))?.[0] ?? "";
};

/**
 * @param {unknown[]} values
 * @param {string} indent
 * @returns {string}
 */
const jsonArray = (values, indent) => {
	const inner = `${indent}  `;
	return `[\n${values.map((value) => `${inner}${JSON.stringify(value)}`).join(",\n")}\n${indent}]`;
};

/**
 * Where the value of `"default":` ends, for a string or array value.
 * @param {string} text
 * @param {number} start
 * @returns {number}
 */
const valueEnd = (text, start) => {
	if (text[start] === "[") {
		return text.indexOf("]", start) + 1;
	}
	let index = start + 1;
	while (text[index] !== '"') {
		index += text[index] === "\\" ? 2 : 1;
	}
	return index + 1;
};

/**
 * @param {string} text
 * @param {string} key
 * @param {string | string[]} value
 * @returns {string}
 */
const patchDefault = (text, key, value) => {
	const keyIndex = text.indexOf(`"${key}": {`);
	if (keyIndex === -1) {
		throw new Error(`${PACKAGE_JSON} has no setting ${key}`);
	}
	const blockEnd = text.indexOf(`\n${indentOf(text, keyIndex)}}`, keyIndex);
	const marker = text.indexOf('"default": ', keyIndex);
	if (marker === -1 || marker > blockEnd) {
		throw new Error(`${key} has no default`);
	}
	const start = marker + '"default": '.length;
	const end = valueEnd(text, start);
	const replacement = Array.isArray(value)
		? jsonArray(value, indentOf(text, marker))
		: JSON.stringify(value);
	return text.slice(0, start) + replacement + text.slice(end);
};

/**
 * @param {string} text
 * @param {import("./manifest-table.mjs").ManifestRow[]} manifests
 * @returns {string}
 */
const patchActivationEvents = (text, manifests) => {
	const marker = '"activationEvents": [';
	const start = text.indexOf(marker);
	if (start === -1) {
		throw new Error(`${PACKAGE_JSON} lacks activationEvents`);
	}
	const end = text.indexOf("]", start);
	const existing = items(
		parseJson(text.slice(start + marker.length - 1, end + 1)),
	).map(String);
	const languages = new Set(languageIds(manifests));
	const kept = existing.filter(
		(event) =>
			event.startsWith("onLanguage:") &&
			languages.has(event.slice("onLanguage:".length)),
	);
	const added = [...languages]
		.map((language) => `onLanguage:${language}`)
		.filter((event) => !kept.includes(event));
	const others = existing.filter((event) => !event.startsWith("onLanguage:"));
	return (
		text.slice(0, start + marker.length - 1) +
		jsonArray([...kept, ...added, ...others], indentOf(text, start)) +
		text.slice(end + 1)
	);
};

/**
 * @param {string} text
 * @param {import("./manifest-table.mjs").ManifestRow[]} manifests
 * @returns {string}
 */
const generatePackageJson = (text, manifests) => {
	let result = text;
	const present = new Set(
		[...text.matchAll(/\n[ \t]*"(versionlens\.[^"]+)": \{/gu)].map(
			(match) => match[1],
		),
	);
	for (const [key, value] of derivedSettings(manifests)) {
		if (!present.has(key)) {
			throw new Error(
				`${PACKAGE_JSON} lacks ${key}; add its schema by hand, then rerun`,
			);
		}
		result = patchDefault(result, key, value);
	}
	return patchActivationEvents(result, manifests);
};

export { generatePackageJson, PACKAGE_JSON };
