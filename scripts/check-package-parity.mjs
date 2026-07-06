#!/usr/bin/env bun

import fs from "node:fs";

const upstreamPath = "external/versionlens/vscode-versionlens/package.json";
const localPath = "packages/vscode-extension/package.json";
const metadataFields = [
	"name",
	"displayName",
	"description",
	"publisher",
	"preview",
	"version",
	"repository",
	"author",
	"license",
	"icon",
	"keywords",
	"categories",
	"engines",
];
const contributionFields = [
	"contributes.commands",
	"contributes.menus",
	"contributes.jsonValidation",
	"contributes.languages",
	"contributes.grammars",
	"contributes.keybindings",
	"contributes.walkthroughs",
];
const configKeyFiles = [
	"cache",
	"dependency-properties",
	"files",
	"http",
	"prerelease",
	"registry",
];
const configPairPattern = /\[\s*"([^"]+)"\s*,\s*"([^"]+)"/g;
const configKeyPattern = /^[a-z][a-z.]*\.[A-Za-z]/;
const locallyExpandedDefaultSuffixes = [".files", ".dependencyProperties"];
const approvedMetadataOverrides = new Map([
	["name", "versionlens-redux"],
	["displayName", "VersionLens Redux"],
	["publisher", "xsyetopz"],
	["version", "0.1.0"],
	["engines", { vscode: ">=1.75.0" }],
	[
		"repository",
		{ type: "git", url: "https://github.com/xsyetopz/versionlens-redux.git" },
	],
]);
const localCommandCategory = "VersionLens Redux";
const upstreamCommandCategory = "VersionLens";
const approvedLocalActivationEvents = new Set([
	"onLanguage:groovy",
	"onLanguage:kotlin",
	"onLanguage:properties",
]);
const failures = [];

function localConfigurationKeys() {
	const keys = new Set();
	for (const name of configKeyFiles) {
		const source = fs.readFileSync(
			`packages/vscode-extension/src/extension/config/keys/${name}.ts`,
			"utf8",
		);
		for (const match of source.matchAll(configPairPattern)) {
			const ecosystem = match[1];
			const key = match[2];
			if (configKeyPattern.test(key)) {
				keys.add(`versionlens.${key}`);
			}
			if (key.endsWith(".files")) {
				keys.add(`versionlens.${ecosystem}.onSaveChanges`);
			}
		}
	}
	return keys;
}

function readJson(filePath) {
	return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function stable(value) {
	if (Array.isArray(value)) {
		return value.map(stable);
	}
	if (value && typeof value === "object") {
		return Object.fromEntries(
			Object.keys(value)
				.sort()
				.map((key) => [key, stable(value[key])]),
		);
	}
	return value;
}

function getPath(source, keyPath) {
	return keyPath
		.split(".")
		.reduce(
			(value, key) => (value === undefined ? undefined : value[key]),
			source,
		);
}

function sameValue(left, right) {
	return JSON.stringify(stable(left)) === JSON.stringify(stable(right));
}

function compareField(upstream, local, keyPath) {
	const upstreamValue = getPath(upstream, keyPath);
	const localValue = getPath(local, keyPath);
	if (approvedMetadataOverrides.has(keyPath)) {
		if (!sameValue(localValue, approvedMetadataOverrides.get(keyPath))) {
			failures.push(`${keyPath} differs from approved local rebrand`);
		}
		return;
	}
	if (!sameValue(upstreamValue, localValue)) {
		failures.push(`${keyPath} differs from upstream package.json`);
	}
}

function configurationPropertyKeys(configuration) {
	return Object.keys(configuration?.properties ?? {}).sort();
}

function compareConfigurationKeys(upstreamKeys, localKeys) {
	const upstreamKeySet = new Set(upstreamKeys);
	const localKeySet = new Set(localKeys);
	const supportedLocalKeys = localConfigurationKeys();
	for (const key of upstreamKeys) {
		if (!localKeySet.has(key)) {
			failures.push(
				`contributes.configuration is missing upstream property ${key}`,
			);
		}
	}
	for (const key of localKeys) {
		if (!(upstreamKeySet.has(key) || supportedLocalKeys.has(key))) {
			failures.push(
				`contributes.configuration has unsupported extra property ${key}`,
			);
		}
	}
}

function removeApprovedExtraConfiguration(
	normalizedLocal,
	upstreamKeys,
	localKeys,
) {
	const upstreamKeySet = new Set(upstreamKeys);
	for (const key of localKeys) {
		if (!upstreamKeySet.has(key)) {
			delete normalizedLocal.properties[key];
		}
	}
}

function normalizeLocalConfigurationRebrand(normalizedLocal) {
	if (normalizedLocal?.title === localCommandCategory) {
		normalizedLocal.title = upstreamCommandCategory;
	}
}

function normalizeLocallyExpandedDefaults(normalizedUpstream, normalizedLocal) {
	for (const key of Object.keys(normalizedLocal?.properties ?? {})) {
		if (
			!locallyExpandedDefaultSuffixes.some((suffix) => key.endsWith(suffix))
		) {
			continue;
		}
		const upstreamSetting = normalizedUpstream?.properties?.[key];
		const localSetting = normalizedLocal?.properties?.[key];
		if (!(upstreamSetting && localSetting)) {
			continue;
		}
		delete upstreamSetting.default;
		delete localSetting.default;
	}
}

function compareConfiguration(upstream, local) {
	const upstreamConfig = upstream.contributes?.configuration;
	const localConfig = local.contributes?.configuration;
	const upstreamKeys = configurationPropertyKeys(upstreamConfig);
	const localKeys = configurationPropertyKeys(localConfig);
	compareConfigurationKeys(upstreamKeys, localKeys);
	if (failures.length > 0) {
		return;
	}

	const normalizedLocal = stable(localConfig);
	const normalizedUpstream = stable(upstreamConfig);
	removeApprovedExtraConfiguration(normalizedLocal, upstreamKeys, localKeys);
	normalizeLocallyExpandedDefaults(normalizedUpstream, normalizedLocal);
	normalizeLocalConfigurationRebrand(normalizedLocal);

	if (!sameValue(normalizedUpstream, normalizedLocal)) {
		failures.push(
			"contributes.configuration differs from upstream package.json",
		);
	}
}

function compareActivationEvents(upstream, local) {
	const upstreamEvents = upstream.activationEvents ?? [];
	const localEvents = local.activationEvents ?? [];
	const upstreamSet = new Set(upstreamEvents);
	const localSet = new Set(localEvents);
	for (const event of upstreamEvents) {
		if (!localSet.has(event)) {
			failures.push(`activationEvents is missing upstream event ${event}`);
		}
	}

	for (const event of localEvents) {
		if (!(upstreamSet.has(event) || approvedLocalActivationEvents.has(event))) {
			failures.push(`activationEvents has unsupported extra event ${event}`);
		}
	}
}

const upstream = readJson(upstreamPath);
const local = readJson(localPath);

for (const field of metadataFields) {
	compareField(upstream, local, field);
}
function normalizeLocalContributions(value) {
	if (Array.isArray(value)) {
		return value.map(normalizeLocalContributions);
	}
	if (value && typeof value === "object") {
		return Object.fromEntries(
			Object.entries(value).map(([key, entry]) => [
				key,
				key === "category" && entry === localCommandCategory
					? upstreamCommandCategory
					: normalizeLocalContributions(entry),
			]),
		);
	}
	return value;
}

for (const field of contributionFields) {
	const upstreamValue = getPath(upstream, field);
	const localValue = normalizeLocalContributions(getPath(local, field));
	if (!sameValue(upstreamValue, localValue)) {
		failures.push(`${field} differs from upstream package.json`);
	}
}
compareConfiguration(upstream, local);
compareActivationEvents(upstream, local);

if (failures.length > 0) {
	console.error(failures.join("\n"));
	process.exit(1);
}
