import { expect, it } from "../runtime.ts";

import {
  authenticatedSessionState,
  authenticationContext,
  configureSession,
  prereleaseTagFilters as expectedPrereleaseTagFilters,
  registryUrls as expectedRegistryUrls,
  npmDependencyProperties,
  providerCacheEntries,
  providerHttpEntries,
  sessionCacheMinutes,
} from "./configuration.ts";
import {
  configured,
  configuredByResource,
  createdSessionConfig,
  setWorkspaceFolders,
} from "./support.ts";

interface SessionModules {
  cache: typeof import("../../session/cache.ts");
  flags: typeof import("../../session/flags.ts");
  http: typeof import("../../session/http.ts");
  indicators: typeof import("../../session/indicators.ts");
  providers: typeof import("../../session/providers.ts");
  registry: typeof import("../../session/registry.ts");
}

interface FeatureFlags {
  providerBusy: number;
  providerError: boolean;
  showPrereleases: boolean;
  showSuggestionStats: boolean;
  showVersionLenses: boolean;
}

function resetConfiguration(): void {
  setWorkspaceFolders(undefined);
  configuredByResource.clear();
  for (const key of Object.keys(configured)) {
    delete configured[key];
  }
}

async function loadSessionModules(): Promise<SessionModules> {
  const [cache, flags, http, indicators, providers, registry] =
    await Promise.all([
      import("../../session/cache.ts"),
      import("../../session/flags.ts"),
      import("../../session/http.ts"),
      import("../../session/indicators.ts"),
      import("../../session/providers.ts"),
      import("../../session/registry.ts"),
    ]);
  return { cache, flags, http, indicators, providers, registry };
}

function featureFlags(): FeatureFlags {
  return {
    providerBusy: 0,
    providerError: false,
    showPrereleases: false,
    showSuggestionStats: false,
    showVersionLenses: false,
  };
}

it("leaves parser and provider defaults in Rust unless configured", async (): Promise<void> => {
  resetConfiguration();
  const modules = await loadSessionModules();
  expect(modules.cache.cacheDurationMinutes()).toBeUndefined();
  expect(modules.flags.configuredEnabledProviders()).toBeUndefined();
  expect(modules.flags.configuredShowVulnerabilities()).toBeUndefined();
  expect(modules.providers.dependencyProperties()).toEqual([]);
  expect(modules.providers.prereleaseTagFilters()).toEqual([]);
  expect(modules.providers.registryUrls()).toEqual([]);
  expect(modules.indicators.suggestionIndicators()).toBeUndefined();
  configured.enabledProviders = [];
  configured["suggestions.indicators"] = {};
  expect(modules.flags.configuredEnabledProviders()).toEqual([]);
  expect(modules.indicators.suggestionIndicators()).toEqual({});
  expect(await modules.http.httpConfig({} as never)).toEqual({});
  const flags = featureFlags();
  modules.flags.reloadConfigurationState({ flags } as never);
  expect(flags).toMatchObject({
    showPrereleases: false,
    showSuggestionStats: false,
    showVersionLenses: false,
  });
});

it("maps configured provider policies into native session inputs", async (): Promise<void> => {
  resetConfiguration();
  configureSession();
  const modules = await loadSessionModules();
  const flags = featureFlags();
  expect(modules.cache.cacheDurationMinutes()).toBe(sessionCacheMinutes);
  expect(modules.flags.configuredEnabledProviders()).toEqual(["cargo"]);
  expect(modules.flags.configuredShowVulnerabilities()).toBe(false);
  modules.flags.reloadConfigurationState({ flags } as never);
  expect(flags.showVersionLenses).toBe(true);
  expect(flags.showPrereleases).toBe(false);
  configured["suggestions.showPrereleasesOnStartup"] = true;
  modules.flags.reloadConfigurationState({ flags } as never);
  expect(flags.showPrereleases).toBe(true);
  configured["suggestions.showPrereleasesOnStartup"] = false;
  modules.flags.reloadConfigurationState({ flags } as never);
  expect(flags.showPrereleases).toBe(false);
  expect(modules.providers.dependencyProperties()).toContainEqual({
    ecosystem: "deno",
    properties: ["imports"],
  });
  expect(modules.providers.dependencyProperties()).toContainEqual({
    ecosystem: "npm",
    properties: npmDependencyProperties,
  });
  expect(modules.providers.dependencyProperties()).toContainEqual({
    ecosystem: "pypi",
    properties: ["tool.uv.sources"],
  });
  expect(modules.providers.registryUrls()).toEqual(
    expectedRegistryUrls.map(({ ecosystem, url }) => ({ ecosystem, url })),
  );
  expect(modules.providers.prereleaseTagFilters()).toEqual(
    expectedPrereleaseTagFilters.map(({ ecosystem, tags }) => ({
      ecosystem,
      tags: [...tags],
    })),
  );
});

it("maps authentication, HTTP, and indicator configuration", async (): Promise<void> => {
  resetConfiguration();
  configureSession();
  const modules = await loadSessionModules();
  await modules.registry.recreateSessions(authenticatedSessionState() as never);
  expect(createdSessionConfig).toMatchObject({
    providers: {
      filePatterns: [
        { ecosystem: "composer", pattern: "**/acme.composer.json" },
      ],
      providerCache: providerCacheEntries,
      providerHttp: providerHttpEntries,
    },
  });
  expect(modules.indicators.suggestionIndicators()).toEqual({
    build: "B",
    directory: "D",
    error: "E",
    latest: "L",
    matched: "M",
    noMatch: "N",
    satisfiesLatest: "S",
    updateable: "U",
    updateableVulnerable: "V",
  });
  expect(
    await modules.http.httpConfig({
      context: authenticationContext(),
    } as never),
  ).toEqual({
    authHeaders: [
      {
        name: "Authorization",
        url: "https://registry.example.test",
        value: "Bearer token",
      },
    ],
    proxy: "http://localhost:8080",
  });
  configured["http.strictSSL"] = false;
  expect(await modules.http.httpConfig({} as never)).toMatchObject({
    strictSsl: false,
  });
});
