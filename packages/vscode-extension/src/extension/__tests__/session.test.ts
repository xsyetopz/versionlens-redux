import { expect, mock, test } from "bun:test";

const configured: Record<
	string,
	| boolean
	| number
	| null
	| Record<string, string>
	| { name: string; secret: string; url?: string }[]
	| string
	| string[]
	| undefined
> = {};
let createdSessionConfig: unknown;

mock.module("vscode", () => ({
	workspace: {
		getConfiguration() {
			return {
				get(key: string, fallback?: unknown) {
					return Object.hasOwn(configured, key) ? configured[key] : fallback;
				},
				inspect(key: string) {
					const value = configured[key];
					return value === undefined ? undefined : { workspaceValue: value };
				},
			};
		},
	},
}));

mock.module("../native/module.ts", () => ({
	loadNative() {
		return {
			createSession(config: unknown) {
				createdSessionConfig = config;
				return {
					analyzeDocument: () => undefined,
					applyCommand: () => undefined,
					clearCache: () => undefined,
					disposeSession: () => undefined,
					resolveDocument: () => undefined,
				};
			},
		};
	},
}));

test("dependencyProperties leaves parser defaults in Rust unless configured", async () => {
	for (const key of Object.keys(configured)) {
		delete configured[key];
	}
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

	const {
		cacheDurationMinutes,
		configuredEnabledProviders,
		configuredShowVulnerabilities,
		dependencyProperties,
		httpConfig,
		prereleaseTagFilters,
		recreateSession,
		reloadConfigurationState,
		registryUrls,
		suggestionIndicators,
	} = await import("../session.ts");

	expect(cacheDurationMinutes()).toBeUndefined();
	expect(configuredEnabledProviders()).toBeUndefined();
	expect(configuredShowVulnerabilities()).toBeUndefined();
	expect(dependencyProperties()).toEqual([]);
	expect(prereleaseTagFilters()).toEqual([]);
	expect(registryUrls()).toEqual([]);
	expect(suggestionIndicators()).toBeUndefined();
	configured["enabledProviders"] = [];
	configured["suggestions.indicators"] = {};
	expect(configuredEnabledProviders()).toEqual([]);
	expect(suggestionIndicators()).toEqual({});
	expect(await httpConfig({} as never)).toEqual({});
	const flags = {
		providerBusy: 0,
		providerError: false,
		showPrereleases: false,
		showSuggestionStats: false,
		showVersionLenses: false,
	};
	reloadConfigurationState({ flags } as never);
	expect(flags.showVersionLenses).toBe(false);
	expect(flags.showPrereleases).toBe(false);
	expect(flags.showSuggestionStats).toBe(false);

	configured["cacheTtlSeconds"] = 42;
	configured["caching.duration"] = 3;
	configured["enabledProviders"] = ["cargo"];
	configured["suggestions.showOnStartup"] = true;
	configured["showPrereleases"] = true;
	configured["deno.dependencyProperties"] = ["imports"];
	configured["npm.dependencyProperties"] = npmDependencyProperties;
	configured["pypi.dependencyProperties"] = ["tool.uv.sources"];
	configured["composer.files"] = "**/acme.composer.json";
	configured["cargo.apiUrl"] = "https://mirror.test/crates";
	configured["composer.apiUrl"] = "https://composer.test";
	configured["dub.apiUrl"] = "https://dub.test";
	configured["golang.apiUrl"] = "https://proxy.test";
	configured["maven.apiUrl"] = "https://maven.test";
	configured["pypi.apiUrl"] = "https://pypi.test/simple";
	configured["pub.apiUrl"] = "https://pub.test";
	configured["ruby.apiUrl"] = "https://ruby.test";
	configured["dotnet.nuget.sources"] = ["https://nuget.test/v3/index.json", ""];
	configured["cargo.prereleaseTagFilter"] = ["alpha"];
	configured["composer.prereleaseTagFilter"] = ["dev"];
	configured["deno.prereleaseTagFilter"] = ["canary"];
	configured["dotnet.prereleaseTagFilter"] = ["preview"];
	configured["dub.prereleaseTagFilter"] = ["pre"];
	configured["golang.prereleaseTagFilter"] = ["beta"];
	configured["maven.prereleaseTagFilter"] = ["milestone"];
	configured["npm.prereleaseTagFilter"] = [];
	configured["pypi.prereleaseTagFilter"] = ["dev"];
	configured["pub.prereleaseTagFilter"] = ["beta"];
	configured["ruby.prereleaseTagFilter"] = ["pre"];
	configured["cargo.caching.duration"] = 9;
	configured["composer.caching.duration"] = 10;
	configured["deno.caching.duration"] = 11;
	configured["docker.caching.duration"] = 12;
	configured["dotnet.caching.duration"] = 13;
	configured["dub.caching.duration"] = 14;
	configured["golang.caching.duration"] = 15;
	configured["maven.caching.duration"] = 16;
	configured["npm.caching.duration"] = 17;
	configured["pub.caching.duration"] = 19;
	configured["pypi.caching.duration"] = 20;
	configured["ruby.caching.duration"] = 21;
	configured["cargo.http.strictSSL"] = null;
	configured["composer.http.strictSSL"] = true;
	configured["deno.http.strictSSL"] = false;
	configured["docker.http.strictSSL"] = true;
	configured["dotnet.http.strictSSL"] = false;
	configured["dub.http.strictSSL"] = true;
	configured["golang.http.strictSSL"] = false;
	configured["maven.http.strictSSL"] = true;
	configured["pub.http.strictSSL"] = false;
	configured["pypi.http.strictSSL"] = false;
	configured["ruby.http.strictSSL"] = true;
	configured["proxy"] = "http://localhost:8080";
	configured["proxyStrictSSL"] = false;
	configured["http.strictSSL"] = null;
	configured["http.caFile"] = "/tmp/versionlens-ca.pem";
	configured["http.timeoutMs"] = 15_000;
	const workspaceAuth = {
		UrlAuthenticationStore: {
			"https://registry.example.test": {
				label: "Custom Value",
				protocol: "https:",
				scheme: "Custom",
				status: "NoStatus",
				url: "https://registry.example.test",
			},
		},
	};
	configured["suggestions.showVulnerabilities"] = false;
	configured["suggestions.indicators"] = {
		Build: "B",
		Directory: "D",
		Error: "E",
		Latest: "L",
		Match: "M",
		NoMatch: "N",
		SatisfiesLatest: "S",
		Updateable: "U",
		UpdateableVulnerable: "V",
	};

	expect(cacheDurationMinutes()).toBe(3);
	expect(configuredEnabledProviders()).toEqual(["cargo"]);
	expect(configuredShowVulnerabilities()).toBe(false);
	reloadConfigurationState({ flags } as never);
	expect(flags.showVersionLenses).toBe(true);
	expect(flags.showPrereleases).toBe(false);
	configured["suggestions.showPrereleasesOnStartup"] = true;
	reloadConfigurationState({ flags } as never);
	expect(flags.showPrereleases).toBe(true);
	configured["suggestions.showPrereleasesOnStartup"] = false;
	reloadConfigurationState({ flags } as never);
	expect(flags.showPrereleases).toBe(false);
	expect(dependencyProperties()).toContainEqual({
		ecosystem: "deno",
		properties: ["imports"],
	});
	expect(dependencyProperties()).toContainEqual({
		ecosystem: "npm",
		properties: npmDependencyProperties,
	});
	expect(dependencyProperties()).toContainEqual({
		ecosystem: "pypi",
		properties: ["tool.uv.sources"],
	});
	expect(registryUrls()).toEqual([
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
	]);
	expect(prereleaseTagFilters()).toEqual([
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
	]);
	await recreateSession({
		context: {
			extensionPath: "/test/extension",
			secrets: {
				get: async () => "Bearer token",
			},
			storageUri: { path: "/workspace/.vscode" },
			workspaceState: {
				get: (key: string, fallback: unknown) =>
					workspaceAuth[key as keyof typeof workspaceAuth] ?? fallback,
			},
		},
		flags: { showPrereleases: true, showSuggestionStats: false },
	} as never);
	expect(createdSessionConfig).toMatchObject({
		providers: {
			filePatterns: [
				{ ecosystem: "composer", pattern: "**/acme.composer.json" },
			],
			providerCache: [
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
			],
			providerHttp: [
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
			],
		},
	});
	expect(suggestionIndicators()).toEqual({
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
		await httpConfig({
			context: {
				secrets: {
					get: async () => "Bearer token",
				},
				storageUri: { path: "/workspace/.vscode" },
				workspaceState: {
					get: (key: string, fallback: unknown) =>
						workspaceAuth[key as keyof typeof workspaceAuth] ?? fallback,
				},
			},
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
	expect(await httpConfig({} as never)).toMatchObject({ strictSsl: false });
});
