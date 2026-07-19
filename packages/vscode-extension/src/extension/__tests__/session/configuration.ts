import { configured } from "./support.ts";

const sessionCacheMinutes = 3;
const cacheTtlSeconds = 42;
const httpTimeoutMs = 15_000;
const npmDependencyProperties = [
  "version",
  "packageManager",
  "devEngines.packageManager",
  "dependencies",
  "devDependencies",
  "peerDependencies",
  "optionalDependencies",
  "overrides",
  "overrides.*",
  "jspm.dependencies",
  "jspm.devDependencies",
  "jspm.peerDependencies",
  "jspm.optionalDependencies",
  "pnpm.overrides",
  "pnpm.overrides.*",
  "workspaces.catalog",
  "workspaces.catalogs.*",
  "customDependencies",
];
const providerCacheEntries = [
  { cacheDurationMinutes: 9, ecosystem: "cargo" },
  { cacheDurationMinutes: 10, ecosystem: "composer" },
  { cacheDurationMinutes: 11, ecosystem: "deno" },
  { cacheDurationMinutes: 12, ecosystem: "docker" },
  { cacheDurationMinutes: 13, ecosystem: "dotnet" },
  { cacheDurationMinutes: 14, ecosystem: "dub" },
  { cacheDurationMinutes: 15, ecosystem: "golang" },
  { cacheDurationMinutes: 16, ecosystem: "maven" },
  { cacheDurationMinutes: 17, ecosystem: "npm" },
  { cacheDurationMinutes: 19, ecosystem: "pub" },
  { cacheDurationMinutes: 20, ecosystem: "pypi" },
  { cacheDurationMinutes: 21, ecosystem: "ruby" },
] as const;
const providerHttpEntries = [
  { ecosystem: "composer", strictSsl: true },
  { ecosystem: "deno", strictSsl: false },
  { ecosystem: "docker", strictSsl: true },
  { ecosystem: "dotnet", strictSsl: false },
  { ecosystem: "dub", strictSsl: true },
  { ecosystem: "golang", strictSsl: false },
  { ecosystem: "maven", strictSsl: true },
  { ecosystem: "pub", strictSsl: false },
  { ecosystem: "pypi", strictSsl: false },
  { ecosystem: "ruby", strictSsl: true },
] as const;
const workspaceAuth: Record<string, unknown> = Object.fromEntries([
  [
    "UrlAuthenticationStore",
    Object.fromEntries([
      [
        "https://registry.example.test",
        {
          label: "Custom Value",
          protocol: "https:",
          scheme: "Custom",
          status: "NoStatus",
          url: "https://registry.example.test",
        },
      ],
    ]),
  ],
]);
const registryUrls = [
  { ecosystem: "cargo", url: "https://mirror.test/crates" },
  { ecosystem: "composer", url: "https://composer.test" },
  { ecosystem: "dub", url: "https://dub.test" },
  { ecosystem: "golang", url: "https://proxy.test" },
  { ecosystem: "maven", url: "https://maven.test" },
  { ecosystem: "pypi", url: "https://pypi.test/simple" },
  { ecosystem: "pub", url: "https://pub.test" },
  { ecosystem: "ruby", url: "https://ruby.test" },
  { ecosystem: "dotnet", url: "https://nuget.test/v3/index.json" },
  { ecosystem: "dotnet", url: "" },
] as const;
const prereleaseTagFilters = [
  { ecosystem: "cargo", tags: ["alpha"] },
  { ecosystem: "composer", tags: ["dev"] },
  { ecosystem: "deno", tags: ["canary"] },
  { ecosystem: "dotnet", tags: ["preview"] },
  { ecosystem: "dub", tags: ["pre"] },
  { ecosystem: "golang", tags: ["beta"] },
  { ecosystem: "maven", tags: ["milestone"] },
  { ecosystem: "npm", tags: [] },
  { ecosystem: "pypi", tags: ["dev"] },
  { ecosystem: "pub", tags: ["beta"] },
  { ecosystem: "ruby", tags: ["pre"] },
] as const;

function configureBasics(): void {
  configured.cacheTtlSeconds = cacheTtlSeconds;
  configured["caching.duration"] = sessionCacheMinutes;
  configured.enabledProviders = ["cargo"];
  configured["suggestions.showOnStartup"] = true;
  configured.showPrereleases = true;
  configured["deno.dependencyProperties"] = ["imports"];
  configured["npm.dependencyProperties"] = npmDependencyProperties;
  configured["pypi.dependencyProperties"] = ["tool.uv.sources"];
  configured["composer.files"] = "**/acme.composer.json";
}

function configureRegistryUrls(): void {
  configured["cargo.apiUrl"] = "https://mirror.test/crates";
  configured["composer.apiUrl"] = "https://composer.test";
  configured["dub.apiUrl"] = "https://dub.test";
  configured["golang.apiUrl"] = "https://proxy.test";
  configured["maven.apiUrl"] = "https://maven.test";
  configured["pypi.apiUrl"] = "https://pypi.test/simple";
  configured["pub.apiUrl"] = "https://pub.test";
  configured["ruby.apiUrl"] = "https://ruby.test";
  configured["dotnet.nuget.sources"] = ["https://nuget.test/v3/index.json", ""];
}

function configurePrereleases(): void {
  for (const [provider, tags] of [
    ["cargo", ["alpha"]],
    ["composer", ["dev"]],
    ["deno", ["canary"]],
    ["dotnet", ["preview"]],
    ["dub", ["pre"]],
    ["golang", ["beta"]],
    ["maven", ["milestone"]],
    ["npm", []],
    ["pypi", ["dev"]],
    ["pub", ["beta"]],
    ["ruby", ["pre"]],
  ] as const) {
    configured[`${provider}.prereleaseTagFilter`] = [...tags];
  }
}

function authenticationContext(): unknown {
  return {
    secrets: {
      get: async (): Promise<string> => "Bearer token",
    },
    storageUri: { path: "/workspace/.vscode" },
    workspaceState: {
      get: (key: string, fallback: unknown): unknown =>
        workspaceAuth[key] ?? fallback,
    },
  };
}

function authenticatedSessionState(): unknown {
  return {
    context: {
      extensionPath: "/test/extension",
      ...(authenticationContext() as object),
    },
    flags: { showPrereleases: true, showSuggestionStats: false },
    lifecycle: { sessionGenerations: new Map() },
    sessions: new Map(),
  };
}

function configureProviderPolicies(): void {
  for (const { cacheDurationMinutes, ecosystem } of providerCacheEntries) {
    configured[`${ecosystem}.caching.duration`] = cacheDurationMinutes;
  }
  configured["cargo.http.strictSSL"] = null;
  for (const { ecosystem, strictSsl } of providerHttpEntries) {
    configured[`${ecosystem}.http.strictSSL`] = strictSsl;
  }
}

function configureHttpAndIndicators(): void {
  configured.proxy = "http://localhost:8080";
  Object.assign(configured, Object.fromEntries([["proxyStrictSSL", false]]));
  configured["http.strictSSL"] = null;
  configured["http.caFile"] = "/tmp/versionlens-ca.pem";
  configured["http.timeoutMs"] = httpTimeoutMs;
  configured["suggestions.showVulnerabilities"] = false;
  configured["suggestions.indicators"] = Object.fromEntries([
    ["Build", "B"],
    ["Directory", "D"],
    ["Error", "E"],
    ["Latest", "L"],
    ["Match", "M"],
    ["NoMatch", "N"],
    ["SatisfiesLatest", "S"],
    ["Updateable", "U"],
    ["UpdateableVulnerable", "V"],
  ]);
}

function configureSession(): void {
  configureBasics();
  configureRegistryUrls();
  configurePrereleases();
  configureProviderPolicies();
  configureHttpAndIndicators();
}

export {
  authenticatedSessionState,
  authenticationContext,
  configureSession,
  npmDependencyProperties,
  prereleaseTagFilters,
  providerCacheEntries,
  providerHttpEntries,
  registryUrls,
  sessionCacheMinutes,
  workspaceAuth,
};
