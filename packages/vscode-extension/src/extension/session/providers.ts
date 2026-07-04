import * as vscode from "vscode";
import { providerCacheKeys } from "../config/keys/cache.ts";
import { dependencyPropertyKeys } from "../config/keys/dependency-properties.ts";
import { filePatternKeys } from "../config/keys/files.ts";
import { providerStrictSslKeys } from "../config/keys/http.ts";
import { prereleaseTagKeys } from "../config/keys/prerelease.ts";
import { registryUrlKeys } from "../config/keys/registry.ts";
import type {
	NativeDependencyPropertyConfig,
	NativeFilePatternConfig,
	NativePrereleaseTagFilter,
	NativeProviderCacheConfig,
	NativeProviderHttpConfig,
	NativeRegistryUrl,
} from "../native/config.ts";
import { configuredValue } from "./configured.ts";

export function dependencyProperties(): NativeDependencyPropertyConfig[] {
	const config = vscode.workspace.getConfiguration("versionlens");

	return dependencyPropertyKeys.flatMap(([ecosystem, key]) => {
		const properties = configuredValue<string[] | undefined>(key, config);
		return properties === undefined ? [] : [{ ecosystem, properties }];
	});
}

export function filePatterns(): NativeFilePatternConfig[] {
	const config = vscode.workspace.getConfiguration("versionlens");

	return filePatternKeys.flatMap(([ecosystem, key]) => {
		const pattern = configuredValue<string | undefined>(key, config);
		return pattern === undefined ? [] : [{ ecosystem, pattern }];
	});
}

export function registryUrls(): NativeRegistryUrl[] {
	const config = vscode.workspace.getConfiguration("versionlens");
	const registryUrls = registryUrlKeys.flatMap(([ecosystem, key]) => {
		const url = configuredValue<string | undefined>(key, config);
		return url === undefined ? [] : [{ ecosystem, url }];
	});

	for (const url of configuredValue<(string | undefined)[] | undefined>(
		"dotnet.nuget.sources",
		config,
	) ?? []) {
		if (url !== undefined) {
			registryUrls.push({ ecosystem: "dotnet", url });
		}
	}

	return registryUrls;
}

export function prereleaseTagFilters(): NativePrereleaseTagFilter[] {
	const config = vscode.workspace.getConfiguration("versionlens");

	return prereleaseTagKeys.flatMap(([ecosystem, key]) => {
		const tags = configuredValue<string[] | undefined>(key, config);
		return tags === undefined ? [] : [{ ecosystem, tags }];
	});
}

export function providerHttpConfigs(): NativeProviderHttpConfig[] {
	const config = vscode.workspace.getConfiguration("versionlens");

	return providerStrictSslKeys.flatMap(([ecosystem, key]) => {
		const strictSsl = configuredValue<boolean | null | undefined>(key, config);
		return strictSsl === undefined || strictSsl === null
			? []
			: [{ ecosystem, strictSsl }];
	});
}

export function providerCacheConfigs(): NativeProviderCacheConfig[] {
	const config = vscode.workspace.getConfiguration("versionlens");

	return providerCacheKeys.flatMap(([ecosystem, key]) => {
		const cacheDurationMinutes = configuredValue<number | null | undefined>(
			key,
			config,
		);
		return cacheDurationMinutes === undefined || cacheDurationMinutes === null
			? []
			: [{ cacheDurationMinutes, ecosystem }];
	});
}
