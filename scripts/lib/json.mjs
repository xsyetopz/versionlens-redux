/**
 * @param {unknown} value
 * @returns {value is Record<string, unknown>}
 */
const isRecord = (value) =>
	typeof value === "object" && value !== null && !Array.isArray(value);

/**
 * Parse JSON text without trusting its shape.
 * @param {string} text
 * @returns {unknown}
 */
const parseJson = (text) => {
	/** @type {unknown} */
	const value = JSON.parse(text);
	return value;
};

/**
 * Read the string `version` field of a parsed `package.json`.
 * @param {unknown} manifest
 * @returns {string}
 */
const manifestVersion = (manifest) => {
	if (isRecord(manifest) && typeof manifest["version"] === "string") {
		return manifest["version"];
	}
	throw new Error("package.json has no string version");
};

/**
 * @param {unknown} value
 * @param {string} key
 * @returns {unknown}
 */
const field = (value, key) => (isRecord(value) ? value[key] : undefined);

/**
 * @param {unknown} value
 * @returns {unknown[]}
 */
const items = (value) => (Array.isArray(value) ? value : []);

/**
 * @param {unknown} value
 * @returns {string[]}
 */
const strings = (value) =>
	items(value).filter((entry) => typeof entry === "string");

export { field, isRecord, items, manifestVersion, parseJson, strings };
