import { expect, it } from "../runtime.ts";

import {
  configured,
  configuredByResource,
  createdSessionConfigs,
  createdSessionCount,
  disposedNativeSessions,
  resetCreatedSessionCount,
  setWorkspaceFolders,
  uri,
} from "./support.ts";

type RegistryModule = typeof import("../../session/registry.ts");
type TestUri = ReturnType<typeof uri>;

interface SessionRecord {
  resource?: unknown;
  session: { disposeSession: () => void };
}

interface RegistryState {
  context: Record<string, unknown>;
  flags: { showPrereleases: boolean; showSuggestionStats: boolean };
  lifecycle: { sessionGenerations: Map<unknown, unknown> };
  sessions: Map<string, SessionRecord>;
}

interface RecreationResult {
  changed: boolean;
  firstSession: unknown;
  recreatedFirstSession: unknown;
  secondSession: unknown;
  unchangedSecondSession: unknown;
}

const workspaceSessionCount = 3;

function resetRegistry(): void {
  for (const key of Object.keys(configured)) {
    delete configured[key];
  }
  configuredByResource.clear();
  createdSessionConfigs.length = 0;
  disposedNativeSessions.clear();
}

function configureWorkspaceRoots(): { first: TestUri } {
  const first = uri("file:///workspace/first");
  const second = uri("file:///workspace/second");
  setWorkspaceFolders([{ uri: first }, { uri: second }]);
  configuredByResource.set(first.toString(), {
    "composer.apiUrl": "https://registry.first.test",
    "composer.caching.duration": 11,
    "composer.files": "**/first-composer.json",
    "composer.http.strictSSL": false,
    enabledProviders: ["composer"],
    proxy: "http://proxy.first.test",
  });
  configuredByResource.set(second.toString(), {
    "cargo.apiUrl": "https://registry.second.test",
    "cargo.caching.duration": 22,
    "cargo.files": "**/Second.toml",
    "cargo.http.strictSSL": true,
    enabledProviders: ["cargo"],
    proxy: "http://proxy.second.test",
  });
  return { first };
}

function registryState(): RegistryState {
  return {
    context: { extensionPath: "/test/extension" },
    flags: { showPrereleases: false, showSuggestionStats: false },
    lifecycle: { sessionGenerations: new Map() },
    sessions: new Map(),
  };
}

const expectedFirstWorkspaceConfiguration = {
  enabledProviders: ["composer"],
  http: { proxy: "http://proxy.first.test" },
  providers: {
    dependencyProperties: [],
    filePatterns: [
      { ecosystem: "composer", pattern: "**/first-composer.json" },
    ],
    prereleaseTagFilters: [],
    providerCache: [{ cacheDurationMinutes: 11, ecosystem: "composer" }],
    providerHttp: [{ ecosystem: "composer", strictSsl: false }],
    registryUrls: [
      { ecosystem: "composer", url: "https://registry.first.test" },
    ],
  },
};
const expectedSecondWorkspaceConfiguration = {
  enabledProviders: ["cargo"],
  http: { proxy: "http://proxy.second.test" },
  providers: {
    dependencyProperties: [],
    filePatterns: [{ ecosystem: "cargo", pattern: "**/Second.toml" }],
    prereleaseTagFilters: [],
    providerCache: [{ cacheDurationMinutes: 22, ecosystem: "cargo" }],
    providerHttp: [{ ecosystem: "cargo", strictSsl: true }],
    registryUrls: [{ ecosystem: "cargo", url: "https://registry.second.test" }],
  },
};

async function recreateAffectedWorkspace(
  registry: RegistryModule,
  state: RegistryState,
  first: TestUri,
): Promise<RecreationResult> {
  const firstDocument = { uri: uri("file:///workspace/first/package.json") };
  const secondDocument = { uri: uri("file:///workspace/second/Cargo.toml") };
  const firstSession = registry.sessionForResource(
    state as never,
    firstDocument.uri as never,
  );
  const secondSession = registry.sessionForResource(
    state as never,
    secondDocument.uri as never,
  );
  const changed = await registry.recreateAffectedSessions(
    state as never,
    {
      affectsConfiguration: (
        _section: string,
        resource?: { toString: () => string },
      ): boolean => resource?.toString() === first.toString(),
    } as never,
  );
  return {
    changed,
    firstSession,
    recreatedFirstSession: registry.sessionForResource(
      state as never,
      firstDocument.uri as never,
    ),
    secondSession,
    unchangedSecondSession: registry.sessionForResource(
      state as never,
      secondDocument.uri as never,
    ),
  };
}

function raceState(
  globalSessionKey: string,
  secretResolvers: Array<(value: string) => void>,
  disposeCounts: { initial: number },
): RegistryState {
  const registryUrl = "https://registry.race.test";
  return {
    context: {
      extensionPath: "/test/extension",
      secrets: {
        get: (): Promise<string> =>
          new Promise<string>((resolve): number =>
            secretResolvers.push(resolve),
          ),
      },
      storageUri: { path: "/workspace/.vscode" },
      workspaceState: {
        get: (): Record<string, unknown> => ({
          [registryUrl]: {
            label: "Custom Value",
            protocol: "https:",
            scheme: "Custom",
            status: "NoStatus",
            url: registryUrl,
          },
        }),
      },
    },
    flags: { showPrereleases: false, showSuggestionStats: false },
    lifecycle: { sessionGenerations: new Map() },
    sessions: new Map([
      [
        globalSessionKey,
        {
          resource: undefined,
          session: {
            disposeSession(): void {
              disposeCounts.initial += 1;
            },
          },
        },
      ],
    ]),
  };
}

it("two workspace roots isolate sessions and recreate only affected configuration", async (): Promise<void> => {
  const registry = await import("../../session/registry.ts");
  resetRegistry();
  const { first } = configureWorkspaceRoots();
  const state = registryState();
  expect(await registry.recreateSessions(state as never)).toBe(true);
  expect(state.sessions.size).toBe(workspaceSessionCount);
  expect(createdSessionConfigs).toContainEqual(
    expect.objectContaining(expectedFirstWorkspaceConfiguration),
  );
  expect(createdSessionConfigs).toContainEqual(
    expect.objectContaining(expectedSecondWorkspaceConfiguration),
  );
  const result = await recreateAffectedWorkspace(registry, state, first);
  expect(result.changed).toBe(true);
  expect(result.firstSession).toBeDefined();
  expect(result.secondSession).toBeDefined();
  expect(result.firstSession).not.toBe(result.secondSession);
  expect(result.recreatedFirstSession).not.toBe(result.firstSession);
  expect(result.unchangedSecondSession).toBe(result.secondSession);
  expect(disposedNativeSessions.has(result.firstSession)).toBe(true);
  expect(disposedNativeSessions.has(result.secondSession)).toBe(false);
  setWorkspaceFolders(undefined);
  configuredByResource.clear();
});

it("overlapping session recreation commits only the newest generation", async (): Promise<void> => {
  setWorkspaceFolders(undefined);
  resetRegistry();
  const { globalSessionKey, recreateSessions } = await import(
    "../../session/registry.ts"
  );
  resetCreatedSessionCount();
  const secretResolvers: Array<(value: string) => void> = [];
  const disposeCounts = { initial: 0 };
  const state = raceState(globalSessionKey, secretResolvers, disposeCounts);
  const older = recreateSessions(state as never);
  await Promise.resolve();
  const newer = recreateSessions(state as never);
  await Promise.resolve();
  expect(secretResolvers).toHaveLength(2);
  secretResolvers[1]?.("new-token");
  expect(await newer).toBe(true);
  const winningSession = state.sessions.get(globalSessionKey)?.session;
  secretResolvers[0]?.("old-token");
  expect(await older).toBe(false);
  expect(disposeCounts.initial).toBe(1);
  expect(createdSessionCount).toBe(1);
  expect(state.sessions.get(globalSessionKey)?.session).toBe(winningSession);
});
