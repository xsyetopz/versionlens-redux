#!/usr/bin/env bun

import fs from "node:fs";
import path from "node:path";

const roots = ["packages/vscode-extension/src", "tests"];
const allowedExtensionSourceDirs = new Set(["extension", "schema"]);
const allowedRootSourceFiles = new Set(["extension.ts"]);
const allowedBareImports = new Set(["bun:test", "vscode"]);
const forbiddenNodeImports = new Set(["node:child_process"]);
const preservedPackageFields = [
	"author",
	"categories",
	"description",
	"displayName",
	"engines",
	"icon",
	"license",
	"preview",
	"publisher",
	"repository",
	"version",
];
const importPattern =
	/\b(?:import|export)\b(?:[^"'`]*?\bfrom\s*)?["']([^"']+)["']|createRequire\([^)]*\)\(["']([^"']+)["']\)/g;
const jsonBridgePattern = /\bJSON\.(?:parse|stringify)\s*\(/;
const configReadPattern =
	/\b(?:configuredValue(?:<[^>]+>)?|get(?:<[^>]+>)?)\(\s*"([^"]+)"/g;
const configPairPattern = /\[\s*"[^"]+"\s*,\s*"([^"]+)"/g;
const configTuplePattern = /\[\s*"([^"]+)"\s*,\s*"([^"]+)"/g;
const configKeyPattern = /^[a-z][a-z.]*\.[A-Za-z]/;
const packageAssetPattern = /^(images|schemas|src\/schema)\//;
const markdownImagePattern = /!\[[^\]]*\]\(([^)]+)\)/g;
const imageAssetPattern = /^images\//;
const absoluteUrlPattern = /^[a-z]+:/iu;
const packagedImageBudgetBytes = 147_000;
const offenders = [];

function walk(dir) {
	if (!fs.existsSync(dir)) {
		return [];
	}

	const files = [];
	for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
		const filePath = path.join(dir, entry.name);
		if (entry.isDirectory()) {
			files.push(...walk(filePath));
			continue;
		}

		if (entry.name.endsWith(".ts")) {
			files.push(filePath);
		}
	}
	return files;
}

for (const entry of fs.readdirSync("packages/vscode-extension/src", {
	withFileTypes: true,
})) {
	if (entry.isDirectory() && !allowedExtensionSourceDirs.has(entry.name)) {
		offenders.push(
			`packages/vscode-extension/src/${entry.name}/ is not adapter-owned source`,
		);
	}
	if (entry.isFile() && !allowedRootSourceFiles.has(entry.name)) {
		offenders.push(
			`packages/vscode-extension/src/${entry.name} must live under extension/ or be an entrypoint`,
		);
	}
}

function isAllowed(specifier) {
	return (
		specifier.startsWith(".") ||
		specifier.startsWith("node:") ||
		allowedBareImports.has(specifier)
	);
}

function constSection(source, name) {
	const start = source.indexOf(`export const ${name}`);
	if (start < 0) {
		return "";
	}

	const end = source.indexOf("] as const;", start);
	return end < 0 ? "" : source.slice(start, end);
}

function configTuples(source) {
	return [...source.matchAll(configTuplePattern)]
		.map((match) => [match[1], match[2]])
		.filter(([, key]) => configKeyPattern.test(key));
}

function filePatternDetails(source) {
	return [
		...source.matchAll(/\[\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*\[([^\]]*)\]/g),
	].map((match) => [
		match[2],
		{
			ecosystem: match[1],
			languages: [...match[3].matchAll(/"([^"]+)"/g)].map((entry) => entry[1]),
		},
	]);
}

function expectSuperset(label, current, upstream) {
	const currentSet = new Set(current);
	for (const value of upstream) {
		if (!currentSet.has(value)) {
			offenders.push(`${label} missing upstream contribution ${value}`);
		}
	}
}

function expectArraySuperset(label, current, upstream) {
	if (!Array.isArray(upstream)) {
		return;
	}

	if (!Array.isArray(current)) {
		offenders.push(`${label} is no longer an array`);
		return;
	}

	expectSuperset(label, current, upstream);
}

function keywordTokens(keywords) {
	return keywords.flatMap((keyword) =>
		typeof keyword === "string" ? keyword.split(",").filter(Boolean) : [],
	);
}

function expectPackageKeywords(current, upstream) {
	if (!(Array.isArray(current) && Array.isArray(upstream))) {
		offenders.push("package.json keywords must be arrays");
		return;
	}

	const currentTokens = new Set(keywordTokens(current));
	const upstreamTokens = new Set(keywordTokens(upstream));
	const allowedAdditions = new Set(["cargo", "rust"]);

	for (const token of upstreamTokens) {
		if (!currentTokens.has(token)) {
			offenders.push(`package.json keywords missing upstream keyword ${token}`);
		}
	}

	for (const token of currentTokens) {
		if (!(upstreamTokens.has(token) || allowedAdditions.has(token))) {
			offenders.push(`package.json keywords has unverified keyword ${token}`);
		}
	}
}

function rustStringArray(filePath, name) {
	const source = fs.readFileSync(filePath, "utf8");
	const pattern = new RegExp(
		`const\\s+${name}:\\s*&\\[&str\\]\\s*=\\s*&\\[(?<body>[\\s\\S]*?)\\];`,
		"u",
	);
	const match = source.match(pattern);
	if (!match?.groups?.body) {
		offenders.push(`${filePath} missing Rust array ${name}`);
		return [];
	}

	return [...match.groups.body.matchAll(/"([^"]+)"/g)].map((entry) => entry[1]);
}

function dependencyPropertiesDefault(key) {
	const setting =
		extensionPackage.contributes?.configuration?.properties?.[
			`versionlens.${key}.dependencyProperties`
		];
	if (!Array.isArray(setting?.default)) {
		offenders.push(
			`versionlens.${key}.dependencyProperties default is not an array`,
		);
		return [];
	}

	return setting.default;
}

function menuCommands(packageJson, menu) {
	return (packageJson.contributes?.menus?.[menu] ?? []).map(
		(item) => item.command,
	);
}

function menuPlacements(packageJson, menu) {
	return (packageJson.contributes?.menus?.[menu] ?? []).map(
		(item) => `${item.command}\0${item.group ?? ""}`,
	);
}

function jsonValidationFileMatches(packageJson) {
	return (packageJson.contributes?.jsonValidation ?? []).map(
		(item) => item.fileMatch,
	);
}

function rustVersionLensCommandStrings(filePath) {
	const source = fs.readFileSync(filePath, "utf8");
	return [...source.matchAll(/"(?<command>versionlens\.[^"]+)"/g)].map(
		(match) => match.groups.command,
	);
}

function collectPackageAssetPaths(value, paths = []) {
	if (!value) {
		return paths;
	}

	if (typeof value === "string") {
		if (packageAssetPattern.test(value)) {
			paths.push(value);
		}
		return paths;
	}

	if (Array.isArray(value)) {
		for (const item of value) {
			collectPackageAssetPaths(item, paths);
		}
		return paths;
	}

	if (typeof value === "object") {
		for (const item of Object.values(value)) {
			collectPackageAssetPaths(item, paths);
		}
	}

	return paths;
}

function markdownAssetPath(markdownPath, assetPath) {
	if (absoluteUrlPattern.test(assetPath)) {
		return undefined;
	}

	const packageRelativePath = path.relative(
		"packages/vscode-extension",
		path.normalize(path.join(path.dirname(markdownPath), assetPath)),
	);
	return packageRelativePath.startsWith("..") ? undefined : packageRelativePath;
}

function collectMarkdownAssetPaths(markdownPath) {
	const source = fs.readFileSync(markdownPath, "utf8");
	return [...source.matchAll(markdownImagePattern)]
		.map((match) => markdownAssetPath(markdownPath, match[1]))
		.filter(Boolean);
}

function packageFiles(dir) {
	return fs.readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
		const filePath = path.join(dir, entry.name);
		return entry.isDirectory() ? packageFiles(filePath) : [filePath];
	});
}

for (const root of roots) {
	for (const filePath of walk(root)) {
		const source = fs.readFileSync(filePath, "utf8");
		for (const [index, line] of source.split("\n").entries()) {
			if (jsonBridgePattern.test(line)) {
				offenders.push(
					`${filePath}:${index + 1} uses JSON bridge in TypeScript adapter`,
				);
			}
		}
		for (const match of source.matchAll(importPattern)) {
			const specifier = match[1] ?? match[2];
			if (!isAllowed(specifier)) {
				offenders.push(`${filePath} imports ${specifier}`);
			}
			if (forbiddenNodeImports.has(specifier)) {
				offenders.push(
					`${filePath} imports ${specifier}; Rust owns CLI/domain discovery`,
				);
			}
		}
	}
}

const extensionPackage = JSON.parse(
	fs.readFileSync("packages/vscode-extension/package.json", "utf8"),
);
const upstreamPackage = JSON.parse(
	fs.readFileSync(
		"external/versionlens/vscode-versionlens/package.json",
		"utf8",
	),
);
for (const field of preservedPackageFields) {
	if (
		JSON.stringify(extensionPackage[field]) !==
		JSON.stringify(upstreamPackage[field])
	) {
		offenders.push(`package.json ${field} differs from upstream extension`);
	}
}
expectPackageKeywords(extensionPackage.keywords, upstreamPackage.keywords);
if (extensionPackage.dependencies) {
	offenders.push(
		"packages/vscode-extension/package.json must not have runtime dependencies",
	);
}
if (extensionPackage.devDependencies) {
	offenders.push(
		"packages/vscode-extension/package.json must not have package-local devDependencies; use the root Bun workspace",
	);
}
if (extensionPackage.main !== "./dist/extension.js") {
	offenders.push(
		"packages/vscode-extension/package.json main must point at ./dist/extension.js",
	);
}
const packageAssetPaths = collectPackageAssetPaths({
	icon: extensionPackage.icon,
	contributes: extensionPackage.contributes,
});
const markdownAssetPaths = packageFiles("packages/vscode-extension")
	.filter((filePath) => filePath.endsWith(".md"))
	.flatMap(collectMarkdownAssetPaths);
const referencedAssetPaths = new Set([
	...packageAssetPaths,
	...markdownAssetPaths,
]);

for (const assetPath of referencedAssetPaths) {
	if (!fs.existsSync(path.join("packages/vscode-extension", assetPath))) {
		offenders.push(`package asset ${assetPath} does not exist`);
	}
}
const packagedImagePaths = packageFiles("packages/vscode-extension/images");
for (const imagePath of packagedImagePaths) {
	const packageRelativePath = path.relative(
		"packages/vscode-extension",
		imagePath,
	);
	if (
		imageAssetPattern.test(packageRelativePath) &&
		!referencedAssetPaths.has(packageRelativePath)
	) {
		offenders.push(`${packageRelativePath} is packaged but unreferenced`);
	}
}
const packagedImageBytes = packagedImagePaths.reduce(
	(total, imagePath) => total + fs.statSync(imagePath).size,
	0,
);
if (packagedImageBytes > packagedImageBudgetBytes) {
	offenders.push(
		`packaged image assets use ${packagedImageBytes} bytes; budget is ${packagedImageBudgetBytes} bytes`,
	);
}

const packageIgnorePath = "packages/vscode-extension/.vscodeignore";
const packageIgnoreLines = fs.existsSync(packageIgnorePath)
	? fs.readFileSync(packageIgnorePath, "utf8").split("\n")
	: [];
const requiredPackageFiles = [
	"README.md",
	"LICENSE",
	"dist/extension.js",
	"native/versionlens_napi.node",
	"src/schema/versionlens.multi-registries.json",
];
for (const requiredFile of requiredPackageFiles) {
	if (!fs.existsSync(path.join("packages/vscode-extension", requiredFile))) {
		offenders.push(`packages/vscode-extension missing ${requiredFile}`);
	}
}
for (const requiredIgnore of [
	"src/**",
	"dist/*.node",
	"*.vsix",
	"*.tsbuildinfo",
	"tsconfig.json",
	"node_modules/**",
]) {
	if (!packageIgnoreLines.includes(requiredIgnore)) {
		offenders.push(`${packageIgnorePath} missing ${requiredIgnore}`);
	}
}
for (const forbiddenIgnore of [
	"native/**",
	"native/*.node",
	"*.node",
	"**/*.node",
]) {
	if (packageIgnoreLines.includes(forbiddenIgnore)) {
		offenders.push(
			`${packageIgnorePath} must not ignore packaged native module via ${forbiddenIgnore}`,
		);
	}
}

const commandSource = fs.readFileSync(
	"packages/vscode-extension/src/extension/commands.ts",
	"utf8",
);
const registeredCommands = new Set(
	[
		...commandSource.matchAll(/registerCommand\(\s*"([^"]+)"/g),
		...commandSource.matchAll(/\[\s*"([^"]+)"\s*,\s*"[^"]+"\s*,?\s*\]/g),
	].map((match) => match[1]),
);
const contributedCommands = new Set(
	(extensionPackage.contributes?.commands ?? []).map(
		(command) => command.command,
	),
);
const upstreamCommands = (upstreamPackage.contributes?.commands ?? []).map(
	(command) => command.command,
);
const upstreamInternalRegisteredCommands = new Set([
	"versionlens.suggestion.onChooseBuild",
	"versionlens.suggestion.onFileLink",
	"versionlens.suggestion.onUpdateDependency",
]);
const contributedCommandById = new Map(
	(extensionPackage.contributes?.commands ?? []).map((command) => [
		command.command,
		command,
	]),
);
const upstreamActivationEvents = upstreamPackage.activationEvents ?? [];
if (
	JSON.stringify([...contributedCommands]) !== JSON.stringify(upstreamCommands)
) {
	offenders.push("commands must match upstream package.json exactly");
}
expectSuperset("commands", contributedCommands, upstreamCommands);
for (const command of upstreamPackage.contributes?.commands ?? []) {
	if (
		JSON.stringify(contributedCommandById.get(command.command)) !==
		JSON.stringify(command)
	) {
		offenders.push(`${command.command} contribution differs from upstream`);
	}
}
const localActivationEvents = extensionPackage.activationEvents ?? [];
if (
	!upstreamActivationEvents.every((event) =>
		localActivationEvents.includes(event),
	)
) {
	offenders.push("activationEvents must include upstream package.json events");
}
for (const command of contributedCommands) {
	if (!localActivationEvents.includes(`onCommand:${command}`)) {
		offenders.push(`${command} is contributed but has no onCommand activation`);
	}
}
for (const event of localActivationEvents) {
	if (
		!(
			upstreamActivationEvents.includes(event) ||
			(event.startsWith("onCommand:") &&
				contributedCommands.has(event.slice("onCommand:".length)))
		)
	) {
		offenders.push(`activationEvents has unsupported extra event ${event}`);
	}
}
for (const menu of Object.keys(upstreamPackage.contributes?.menus ?? {})) {
	if (
		JSON.stringify(extensionPackage.contributes?.menus?.[menu] ?? []) !==
		JSON.stringify(upstreamPackage.contributes?.menus?.[menu] ?? [])
	) {
		offenders.push(`${menu} menu must match upstream package.json exactly`);
	}
	expectSuperset(
		`${menu} menu`,
		menuCommands(extensionPackage, menu),
		menuCommands(upstreamPackage, menu),
	);
	expectSuperset(
		`${menu} menu placements`,
		menuPlacements(extensionPackage, menu),
		menuPlacements(upstreamPackage, menu),
	);
}
expectSuperset(
	"jsonValidation",
	jsonValidationFileMatches(extensionPackage),
	jsonValidationFileMatches(upstreamPackage),
);
for (const item of extensionPackage.contributes?.jsonValidation ?? []) {
	if (!fs.existsSync(path.join("packages/vscode-extension", item.url))) {
		offenders.push(`jsonValidation URL ${item.url} does not exist`);
	}
}

for (const command of registeredCommands) {
	if (
		!(
			contributedCommands.has(command) ||
			upstreamInternalRegisteredCommands.has(command)
		)
	) {
		offenders.push(`${command} is registered but not contributed`);
	}
}

for (const command of contributedCommands) {
	if (!registeredCommands.has(command)) {
		offenders.push(`${command} is contributed but not registered`);
	}
}
for (const command of rustVersionLensCommandStrings(
	"crates/versionlens-core/src/presentation.rs",
)) {
	if (
		!(
			contributedCommands.has(command) ||
			upstreamInternalRegisteredCommands.has(command)
		)
	) {
		offenders.push(`${command} is emitted by Rust but not contributed`);
	}
	if (!registeredCommands.has(command)) {
		offenders.push(`${command} is emitted by Rust but not registered`);
	}
}

const contributedSettings = new Set(
	Object.keys(extensionPackage.contributes?.configuration?.properties ?? {}),
);
expectSuperset(
	"configuration",
	contributedSettings,
	Object.keys(upstreamPackage.contributes?.configuration?.properties ?? {}),
);
const contributedConfiguration =
	extensionPackage.contributes?.configuration?.properties ?? {};
const upstreamConfiguration =
	upstreamPackage.contributes?.configuration?.properties ?? {};
for (const [key, setting] of Object.entries(upstreamConfiguration)) {
	const current = contributedConfiguration[key];
	expectArraySuperset(`${key} default`, current?.default, setting.default);
	expectArraySuperset(`${key} enum`, current?.items?.enum, setting.items?.enum);
}

const dependencyDefaultSpecs = [
	[
		"cargo",
		"crates/versionlens-parsers/src/cargo_toml/paths.rs",
		"CARGO_DEPENDENCY_PATHS",
	],
	[
		"composer",
		"crates/versionlens-parsers/src/json_manifest/paths.rs",
		"COMPOSER_DEPENDENCY_PATHS",
	],
	[
		"deno",
		"crates/versionlens-parsers/src/json_manifest/paths.rs",
		"DENO_DEPENDENCY_PATHS",
	],
	[
		"dotnet",
		"crates/versionlens-parsers/src/dotnet_xml.rs",
		"DOTNET_DEPENDENCY_PATHS",
	],
	[
		"dub",
		"crates/versionlens-parsers/src/json_manifest/paths.rs",
		"DUB_DEPENDENCY_PATHS",
	],
	[
		"maven",
		"crates/versionlens-parsers/src/maven_xml.rs",
		"MAVEN_DEPENDENCY_PATHS",
	],
	[
		"npm",
		"crates/versionlens-parsers/src/json_manifest/paths.rs",
		"NPM_DEPENDENCY_PATHS",
	],
	[
		"pnpm",
		"crates/versionlens-parsers/src/pnpm_yaml/paths.rs",
		"PNPM_DEPENDENCY_PATHS",
	],
	[
		"pub",
		"crates/versionlens-parsers/src/pubspec_yaml/paths.rs",
		"PUBSPEC_DEPENDENCY_PATHS",
	],
	[
		"pypi",
		"crates/versionlens-parsers/src/pyproject_toml/paths/defaults.rs",
		"PYPI_DEPENDENCY_PATHS",
	],
];
for (const [key, rustPath, rustConst] of dependencyDefaultSpecs) {
	const rustDefaults = new Set(rustStringArray(rustPath, rustConst));
	for (const path of dependencyPropertiesDefault(key)) {
		if (!rustDefaults.has(path)) {
			offenders.push(
				`versionlens.${key}.dependencyProperties default ${path} is not recognized by ${rustConst}`,
			);
		}
	}
}
const configKeysSource = [
	"cache",
	"dependency-properties",
	"files",
	"http",
	"prerelease",
	"registry",
]
	.map((name) =>
		fs.readFileSync(
			`packages/vscode-extension/src/extension/config/keys/${name}.ts`,
			"utf8",
		),
	)
	.join("\n");
const configKeyPairs = [...configKeysSource.matchAll(configPairPattern)]
	.map((match) => match[1])
	.filter((key) => configKeyPattern.test(key));
const configReads = walk("packages/vscode-extension/src")
	.filter((filePath) => !filePath.endsWith(".test.ts"))
	.flatMap((filePath) =>
		[...fs.readFileSync(filePath, "utf8").matchAll(configReadPattern)].map(
			(match) => match[1],
		),
	);
const versionlensConfigKeys = new Set(
	[...configKeyPairs, ...configReads].filter(
		(key) => !["proxy", "proxyStrictSSL"].includes(key),
	),
);

for (const key of versionlensConfigKeys) {
	if (!contributedSettings.has(`versionlens.${key}`)) {
		offenders.push(`versionlens.${key} is read but not contributed`);
	}
}

const filePatternKeys = new Map(
	configTuples(constSection(configKeysSource, "filePatternKeys")),
);

const rustSupportedFileDefaults = new Map([
	["deno.files", "**/{deno.json,deno.jsonc}"],
	[
		"docker.files",
		"**/{dockerfile,*.dockerfile,Dockerfile,*.Dockerfile,compose.yaml,compose.yml,*.compose.yaml,*.compose.yml,compose.*.yaml,compose.*.yml,docker-compose.yaml,docker-compose.yml,docker-compose.*.yaml,docker-compose.*.yml}",
	],
	[
		"dotnet.files",
		"**/{*.csproj,*.fsproj,*.vbproj,project.json,*.targets,*.props}",
	],
	[
		"pnpm.files",
		"**/{pnpm-workspace.yaml,pnpm-workspace.yml,.yarnrc.yaml,.yarnrc.yml}",
	],
	["pypi.files", "**/{Pipfile,pyproject.toml,*requirements*.txt}"],
	["pub.files", "**/{pubspec.yaml,pubspec.yml}"],
]);
const rustSupportedFileLanguages = new Map([
	["dotnet.files", ["xml", "json", "jsonc"]],
]);
for (const [key, expectedDefault] of rustSupportedFileDefaults) {
	const contributedDefault =
		contributedConfiguration[`versionlens.${key}`]?.default;
	if (contributedDefault !== expectedDefault) {
		offenders.push(
			`versionlens.${key} default does not cover Rust-supported manifest variants`,
		);
	}
}
const filePatternDetailsByKey = new Map(filePatternDetails(configKeysSource));
for (const [key, expectedLanguages] of rustSupportedFileLanguages) {
	const languages = filePatternDetailsByKey.get(key)?.languages ?? [];
	for (const language of expectedLanguages) {
		if (!languages.includes(language)) {
			offenders.push(
				`${key} selectors missing ${language} for Rust-supported manifests`,
			);
		}
	}
}

const providerSettingLists = [
	[
		".apiUrl",
		new Set(
			configTuples(constSection(configKeysSource, "registryUrlKeys")).map(
				([, key]) => key,
			),
		),
	],
	[".files", new Set(filePatternKeys.values())],
	[
		".prereleaseTagFilter",
		new Set(
			configTuples(constSection(configKeysSource, "prereleaseTagKeys")).map(
				([, key]) => key,
			),
		),
	],
	[
		".http.strictSSL",
		new Set(
			configTuples(constSection(configKeysSource, "providerStrictSslKeys")).map(
				([, key]) => key,
			),
		),
	],
	[
		".caching.duration",
		new Set(
			configTuples(constSection(configKeysSource, "providerCacheKeys")).map(
				([, key]) => key,
			),
		),
	],
	[
		".dependencyProperties",
		new Set(
			configTuples(
				constSection(configKeysSource, "dependencyPropertyKeys"),
			).map(([, key]) => key),
		),
	],
];

for (const key of contributedSettings) {
	const shortKey = key.replace(/^versionlens\./, "");
	for (const [suffix, knownKeys] of providerSettingLists) {
		if (shortKey.endsWith(suffix) && !knownKeys.has(shortKey)) {
			offenders.push(
				`${key} is contributed but missing from extension/config/keys/*.ts`,
			);
		}
	}
}

if (offenders.length > 0) {
	console.error(offenders.join("\n"));
	process.exit(1);
}

console.log("checked VS Code adapter; no TS domain/runtime dependency drift");
