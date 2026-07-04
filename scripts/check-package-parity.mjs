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
const expandedFileDefaults = new Map([
	["versionlens.deno.files", "**/{deno.json,deno.jsonc}"],
	[
		"versionlens.docker.files",
		"**/{dockerfile,*.dockerfile,Dockerfile,*.Dockerfile,compose.yaml,compose.yml,*.compose.yaml,*.compose.yml,compose.*.yaml,compose.*.yml,docker-compose.yaml,docker-compose.yml,docker-compose.*.yaml,docker-compose.*.yml}",
	],
	[
		"versionlens.dotnet.files",
		"**/{*.csproj,*.fsproj,*.vbproj,project.json,*.targets,*.props}",
	],
	[
		"versionlens.pnpm.files",
		"**/{pnpm-workspace.yaml,pnpm-workspace.yml,.yarnrc.yaml,.yarnrc.yml}",
	],
	["versionlens.pypi.files", "**/{Pipfile,pyproject.toml,*requirements*.txt}"],
	["versionlens.pub.files", "**/{pubspec.yaml,pubspec.yml}"],
]);
const failures = [];

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
	if (!sameValue(upstreamValue, localValue)) {
		failures.push(`${keyPath} differs from upstream package.json`);
	}
}

function compareConfiguration(upstream, local) {
	const upstreamConfig = upstream.contributes?.configuration;
	const localConfig = local.contributes?.configuration;
	const upstreamPropertyKeys = Object.keys(
		upstreamConfig?.properties ?? {},
	).sort();
	const localPropertyKeys = Object.keys(localConfig?.properties ?? {}).sort();
	if (!sameValue(upstreamPropertyKeys, localPropertyKeys)) {
		failures.push(
			"contributes.configuration properties differ from upstream package.json",
		);
		return;
	}

	const normalizedLocal = stable(localConfig);
	const normalizedUpstream = stable(upstreamConfig);
	for (const [key, expectedDefault] of expandedFileDefaults) {
		const localSetting = normalizedLocal?.properties?.[key];
		const upstreamSetting = normalizedUpstream?.properties?.[key];
		if (!(localSetting && upstreamSetting)) {
			failures.push(`${key} missing from configuration parity inputs`);
			continue;
		}
		if (localSetting.default !== expectedDefault) {
			failures.push(
				`${key} default must include Rust-supported manifest variants`,
			);
		}
		localSetting.default = upstreamSetting.default;
	}

	if (!sameValue(normalizedUpstream, normalizedLocal)) {
		failures.push(
			"contributes.configuration differs from upstream package.json outside approved file-default expansions",
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

	const contributedCommands = new Set(
		(local.contributes?.commands ?? []).map((command) => command.command),
	);
	for (const command of contributedCommands) {
		const event = `onCommand:${command}`;
		if (!localSet.has(event)) {
			failures.push(`activationEvents is missing command activation ${event}`);
		}
	}

	for (const event of localEvents) {
		if (
			!(
				upstreamSet.has(event) ||
				(event.startsWith("onCommand:") &&
					contributedCommands.has(event.slice("onCommand:".length)))
			)
		) {
			failures.push(`activationEvents has unsupported extra event ${event}`);
		}
	}
}

const upstream = readJson(upstreamPath);
const local = readJson(localPath);

for (const field of metadataFields) {
	compareField(upstream, local, field);
}
for (const field of contributionFields) {
	compareField(upstream, local, field);
}
compareConfiguration(upstream, local);
compareActivationEvents(upstream, local);

if (failures.length > 0) {
	console.error("Package parity check failed:");
	for (const failure of failures) {
		console.error(`- ${failure}`);
	}
	process.exit(1);
}
