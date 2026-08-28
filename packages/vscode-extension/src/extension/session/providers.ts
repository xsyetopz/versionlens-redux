import { type Uri, workspace } from "#vscode-host";
import {
  dependencyPropertyKeys,
  filePatternKeys,
  prereleaseTagKeys,
  providerCacheKeys,
  providerStrictSslKeys,
  registryUrlKeys,
} from "../config/keys.ts";
import type {
  NativeDependencyPropertyConfig,
  NativeFilePatternConfig,
  NativePrereleaseTagFilter,
  NativeProviderCacheConfig,
  NativeProviderHttpConfig,
  NativeRegistryUrl,
} from "../native.ts";
import { configuredValue } from "./configured.ts";

export function dependencyProperties(
  resource?: Uri,
): NativeDependencyPropertyConfig[] {
  const config = workspace.getConfiguration("versionlens", resource);

  return dependencyPropertyKeys.flatMap(
    ([ecosystem, key]): Array<{
      ecosystem: string;
      properties: string[];
    }> => {
      const properties = configuredValue<string[] | undefined>(key, config);
      if (properties === undefined) {
        return [];
      }
      return [{ ecosystem, properties }];
    },
  );
}

export function filePatterns(resource?: Uri): NativeFilePatternConfig[] {
  const config = workspace.getConfiguration("versionlens", resource);

  return filePatternKeys.flatMap(
    ([ecosystem, key]): Array<{ ecosystem: string; pattern: string }> => {
      const pattern = configuredValue<string | undefined>(key, config);
      if (pattern === undefined) {
        return [];
      }
      return [{ ecosystem, pattern }];
    },
  );
}

export function registryUrls(resource?: Uri): NativeRegistryUrl[] {
  const config = workspace.getConfiguration("versionlens", resource);
  const configuredRegistryUrls = registryUrlKeys.flatMap(
    ([ecosystem, key]): Array<{ ecosystem: string; url: string }> => {
      const url = configuredValue<string | undefined>(key, config);
      if (url === undefined) {
        return [];
      }
      return [{ ecosystem, url }];
    },
  );

  for (const url of configuredValue<(string | undefined)[] | undefined>(
    "dotnet.nuget.sources",
    config,
  ) ?? []) {
    if (url !== undefined) {
      configuredRegistryUrls.push({ ecosystem: "dotnet", url });
    }
  }

  return configuredRegistryUrls;
}

export function prereleaseTagFilters(
  resource?: Uri,
): NativePrereleaseTagFilter[] {
  const config = workspace.getConfiguration("versionlens", resource);

  return prereleaseTagKeys.flatMap(
    ([ecosystem, key]): Array<{ ecosystem: string; tags: string[] }> => {
      const tags = configuredValue<string[] | undefined>(key, config);
      if (tags === undefined) {
        return [];
      }
      return [{ ecosystem, tags }];
    },
  );
}

export function providerHttpConfigs(
  resource?: Uri,
): NativeProviderHttpConfig[] {
  const config = workspace.getConfiguration("versionlens", resource);

  return providerStrictSslKeys.flatMap(
    ([ecosystem, key]): Array<{ ecosystem: string; strictSsl: boolean }> => {
      const strictSsl = configuredValue<boolean | null | undefined>(
        key,
        config,
      );
      if (strictSsl === undefined || strictSsl === null) {
        return [];
      }
      return [{ ecosystem, strictSsl }];
    },
  );
}

export function providerCacheConfigs(
  resource?: Uri,
): NativeProviderCacheConfig[] {
  const config = workspace.getConfiguration("versionlens", resource);

  return providerCacheKeys.flatMap(
    ([ecosystem, key]): Array<{
      cacheDurationMinutes: number;
      ecosystem: string;
    }> => {
      const cacheDurationMinutes = configuredValue<number | null | undefined>(
        key,
        config,
      );
      if (cacheDurationMinutes === undefined || cacheDurationMinutes === null) {
        return [];
      }
      return [{ cacheDurationMinutes, ecosystem }];
    },
  );
}
