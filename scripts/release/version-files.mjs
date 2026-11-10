import { isRecord, parseJson } from "../lib/json.mjs";
import { capture } from "../lib/spawn.mjs";
import { fail, parseSemver, replaceExactly } from "./semver.mjs";

const ROOT_VERSION_PATTERN =
	/"name": "@versionlens\/workspace",[\s\S]*?"version": "(?<version>[^"]+)"/u;
const ROOT_REPLACEMENT_PATTERN =
	/(?<prefix>"name": "@versionlens\/workspace",[\s\S]*?"version": ")[^"]+(?<suffix>")/u;
const EXTENSION_REPLACEMENT_PATTERN =
	/(?<prefix>"name": "versionlens-redux",[\s\S]*?"version": ")[^"]+(?<suffix>")/u;
const CARGO_REPLACEMENT_PATTERN =
	/(?<prefix>\[workspace\.package\][\s\S]*?^version = ")[^"]+(?<suffix>")/mu;
const BUN_LOCK_REPLACEMENT_PATTERN =
	/(?<prefix>"packages\/vscode-extension": \{\n\s+"name": "versionlens-redux",\n\s+"version": ")[^"]+(?<suffix>")/u;
const ZED_CARGO_REPLACEMENT_PATTERN =
	/(?<prefix>name = "versionlens-zed-extension"\nversion = ")[^"]+(?<suffix>")/u;
const VERSION_LINE_PATTERN = /^version = "[^"]+"$/mu;
const NEOVIM_VERSION_PATTERN = /return "(?<version>[^"]+)"/u;
const NEOVIM_VERSION_REPLACEMENT_PATTERN =
	/(?<prefix>return ")[^"]+(?<suffix>")/u;

/** @type {[string, RegExp][]} */
const manifests = [
	[
		"Cargo.toml",
		/\[workspace\.package\][\s\S]*?^version = "(?<version>[^"]+)"/mu,
	],
	[
		"packages/vscode-extension/package.json",
		/"name": "versionlens-redux",[\s\S]*?"version": "(?<version>[^"]+)"/u,
	],
	[
		"packages/zed-extension/Cargo.toml",
		/name = "versionlens-zed-extension"\nversion = "(?<version>[^"]+)"/u,
	],
	[
		"packages/zed-extension/extension.toml",
		/^version = "(?<version>[^"]+)"$/mu,
	],
	[
		"packages/jetbrains-plugin/build.gradle.kts",
		/^version = "(?<version>[^"]+)"$/mu,
	],
	[
		"packages/neovim-plugin/lua/versionlens/version.lua",
		NEOVIM_VERSION_PATTERN,
	],
];

/**
 * @param {string} path
 * @param {RegExp} pattern
 * @returns {Promise<string>}
 */
const currentVersion = async (path, pattern) => {
	const match = pattern.exec(await Bun.file(path).text());
	const version = match?.groups?.["version"];
	if (version === undefined || version === "") {
		return fail(`${path}: could not read the current version`);
	}
	parseSemver(version);
	return version;
};

/** @returns {Promise<string>} */
const readRepositoryVersion = () =>
	currentVersion("package.json", ROOT_VERSION_PATTERN);

/**
 * @param {unknown} value
 * @returns {string}
 */
const stringField = (value) => {
	if (typeof value !== "string") {
		return fail("cargo metadata returned an unexpected shape");
	}
	return value;
};

/** @returns {Promise<string[]>} */
const rustWorkspacePackages = async () => {
	const metadata = await capture([
		"cargo",
		"metadata",
		"--no-deps",
		"--format-version",
		"1",
	]);
	if (metadata.exitCode !== 0) {
		fail(`cargo metadata failed: ${metadata.stderr}`);
	}
	const workspace = parseJson(metadata.stdout);
	if (!(
		isRecord(workspace) &&
		Array.isArray(workspace["workspace_members"]) &&
		Array.isArray(workspace["packages"])
	)) {
		return fail("cargo metadata returned an unexpected shape");
	}
	/** @type {unknown[]} */
	const members = workspace["workspace_members"];
	/** @type {unknown[]} */
	const packages = workspace["packages"];
	const workspaceMembers = new Set(members);
	// Released crates live in crates/; tooling such as xtask keeps its own version.
	const cratesDir = `${stringField(workspace["workspace_root"])}/crates/`;
	/** @type {string[]} */
	const names = [];
	for (const entry of packages) {
		if (!isRecord(entry)) {
			return fail("cargo metadata returned an unexpected shape");
		}
		if (
			workspaceMembers.has(entry["id"]) &&
			stringField(entry["manifest_path"]).startsWith(cratesDir)
		) {
			names.push(stringField(entry["name"]));
		}
	}
	return names.toSorted((left, right) => left.localeCompare(right));
};

/**
 * @param {string} previousVersion
 * @param {string[]} rustPackages
 * @returns {Promise<void>}
 */
const assertSynchronizedVersions = async (previousVersion, rustPackages) => {
	const manifestVersions = await Promise.all(
		manifests.map(([path, pattern]) => currentVersion(path, pattern)),
	);
	for (const [index, version] of manifestVersions.entries()) {
		if (version !== previousVersion) {
			fail(
				`${manifests[index]?.[0]}: version ${version} is not synchronized with ${previousVersion}`,
			);
		}
	}

	const lockVersions = await Promise.all(
		rustPackages.map((name) =>
			currentVersion(
				"Cargo.lock",
				new RegExp(`name = "${name}"\\nversion = "(?<version>[^"]+)"`, "u"),
			),
		),
	);
	for (const [index, version] of lockVersions.entries()) {
		if (version !== previousVersion) {
			fail(
				`Cargo.lock: ${rustPackages[index]} version ${version} is not synchronized with ${previousVersion}`,
			);
		}
	}
};

/**
 * @typedef {(match: string, prefix: string, suffix: string) => string} ReplaceVersion
 */

/**
 * @typedef {object} Update
 * @property {string} path
 * @property {RegExp} pattern
 * @property {string | ReplaceVersion} replacement
 */

/** @typedef {(path: string, pattern: RegExp, replacement: Update["replacement"]) => void} ApplyUpdate */

/**
 * @param {ApplyUpdate} applyUpdate
 * @param {ReplaceVersion} replaceVersion
 * @returns {void}
 */
const applyCoreManifestUpdates = (applyUpdate, replaceVersion) => {
	applyUpdate("package.json", ROOT_REPLACEMENT_PATTERN, replaceVersion);
	applyUpdate(
		"packages/vscode-extension/package.json",
		EXTENSION_REPLACEMENT_PATTERN,
		replaceVersion,
	);
	applyUpdate("Cargo.toml", CARGO_REPLACEMENT_PATTERN, replaceVersion);
};

/**
 * @param {ApplyUpdate} applyUpdate
 * @param {ReplaceVersion} replaceVersion
 * @param {string[]} rustPackages
 * @returns {void}
 */
const applyRustLockUpdates = (applyUpdate, replaceVersion, rustPackages) => {
	for (const name of rustPackages) {
		applyUpdate(
			"Cargo.lock",
			new RegExp(
				`(?<prefix>name = "${name}"\\nversion = ")[^"]+(?<suffix>")`,
				"u",
			),
			replaceVersion,
		);
	}
};

/**
 * @param {ApplyUpdate} applyUpdate
 * @param {ReplaceVersion} replaceVersion
 * @param {string} nextVersion
 * @returns {void}
 */
const applyPackageManifestUpdates = (
	applyUpdate,
	replaceVersion,
	nextVersion,
) => {
	applyUpdate("bun.lock", BUN_LOCK_REPLACEMENT_PATTERN, replaceVersion);
	applyUpdate(
		"packages/zed-extension/Cargo.toml",
		ZED_CARGO_REPLACEMENT_PATTERN,
		replaceVersion,
	);
	applyUpdate(
		"packages/zed-extension/Cargo.lock",
		ZED_CARGO_REPLACEMENT_PATTERN,
		replaceVersion,
	);
	applyUpdate(
		"packages/zed-extension/extension.toml",
		VERSION_LINE_PATTERN,
		`version = "${nextVersion}"`,
	);
	applyUpdate(
		"packages/jetbrains-plugin/build.gradle.kts",
		VERSION_LINE_PATTERN,
		`version = "${nextVersion}"`,
	);
	applyUpdate(
		"packages/neovim-plugin/lua/versionlens/version.lua",
		NEOVIM_VERSION_REPLACEMENT_PATTERN,
		replaceVersion,
	);
};

/**
 * @param {string} nextVersion
 * @param {string[]} rustPackages
 * @returns {Promise<number>}
 */
const updateRepositoryVersions = async (nextVersion, rustPackages) => {
	/** @type {Update[]} */
	const updates = [];
	/** @type {ApplyUpdate} */
	const applyUpdate = (path, pattern, replacement) => {
		updates.push({ path, pattern, replacement });
	};

	/** @type {ReplaceVersion} */
	const replaceVersion = (_match, prefix, suffix) =>
		prefix + nextVersion + suffix;
	applyCoreManifestUpdates(applyUpdate, replaceVersion);
	applyRustLockUpdates(applyUpdate, replaceVersion, rustPackages);
	applyPackageManifestUpdates(applyUpdate, replaceVersion, nextVersion);

	const paths = [...new Set(updates.map(({ path }) => path))];
	const sources = await Promise.all(paths.map((path) => Bun.file(path).text()));
	/** @type {Map<string, string>} */
	const changes = new Map(
		paths.map((path, index) => [path, sources[index] ?? ""]),
	);
	for (const { path, pattern, replacement } of updates) {
		changes.set(
			path,
			replaceExactly({
				expectedCount: 1,
				path,
				pattern,
				replacement,
				source: changes.get(path) ?? "",
			}),
		);
	}

	await Promise.all(
		[...changes].map(([path, contents]) => Bun.write(path, contents)),
	);
	return changes.size;
};

export {
	assertSynchronizedVersions,
	readRepositoryVersion,
	rustWorkspacePackages,
	updateRepositoryVersions,
};
