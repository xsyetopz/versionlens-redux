import { isRecord, parseJson } from "../lib/json.mjs";

/**
 * @typedef {object} PackageTarget
 * @property {string} vscode
 * @property {string} rust
 */

/**
 * Read `targets.json` next to this file.
 * @returns {Promise<PackageTarget[]>}
 */
const readTargets = async () => {
	const file = parseJson(
		await Bun.file(new URL("targets.json", import.meta.url)).text(),
	);
	if (!(isRecord(file) && Array.isArray(file["targets"]))) {
		throw new Error("targets.json has no targets array");
	}
	/** @type {unknown[]} */
	const entries = file["targets"];
	/** @type {PackageTarget[]} */
	const targets = [];
	for (const entry of entries) {
		if (
			isRecord(entry) &&
			typeof entry["vscode"] === "string" &&
			typeof entry["rust"] === "string"
		) {
			targets.push({ rust: entry["rust"], vscode: entry["vscode"] });
		} else {
			throw new Error("targets.json has a malformed target");
		}
	}
	return targets;
};

export { readTargets };
