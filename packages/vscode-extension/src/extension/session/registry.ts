import {
  type ConfigurationChangeEvent,
  type Uri,
  type WorkspaceFolder,
  workspace,
} from "#vscode-host";
import { optionalProperty } from "../config/optional.ts";
import type { NativeSessionConfig } from "../native/config.ts";
import type { NativeSession } from "../native/module.ts";
import { loadNative } from "../native/module.ts";
import type { ExtensionState, ResourceSession } from "../state.ts";
import { cacheDurationMinutes } from "./cache.ts";
import {
  configuredEnabledProviders,
  configuredShowVulnerabilities,
} from "./flags.ts";
import { httpConfig } from "./http.ts";
import { suggestionIndicators } from "./indicators.ts";
import {
  dependencyProperties,
  filePatterns,
  prereleaseTagFilters,
  providerCacheConfigs,
  providerHttpConfigs,
  registryUrls,
} from "./providers.ts";

const globalSessionKey = "global";
const workspaceSessionPrefix = "workspace:";

type SessionRecreation = Promise<boolean>;

interface SessionIdentity {
  key: string;
  resource: Uri | undefined;
}

function sessionForResource(
  state: ExtensionState,
  resource: Uri | undefined,
): NativeSession | undefined {
  return state.sessions.get(sessionIdentityForResource(resource).key)?.session;
}

async function recreateSessions(state: ExtensionState): SessionRecreation {
  const identities = synchronizeSessionIdentities(state);
  const results = await Promise.all(
    identities.map(
      (identity): SessionRecreation => recreateSession(state, identity),
    ),
  );
  return results.every(Boolean);
}

async function recreateAffectedSessions(
  state: ExtensionState,
  event: ConfigurationChangeEvent,
): SessionRecreation {
  const identities = synchronizeSessionIdentities(state);
  const affected = identities.filter(
    (identity): boolean =>
      !state.sessions.has(identity.key) ||
      event.affectsConfiguration("versionlens", identity.resource) ||
      event.affectsConfiguration("http", identity.resource),
  );
  const results = await Promise.all(
    affected.map(
      (identity): SessionRecreation => recreateSession(state, identity),
    ),
  );
  return results.every(Boolean);
}

async function synchronizeWorkspaceSessions(
  state: ExtensionState,
): SessionRecreation {
  const identities = synchronizeSessionIdentities(state);
  const missing = identities.filter(
    ({ key }): boolean => !state.sessions.has(key),
  );
  const results = await Promise.all(
    missing.map(
      (identity): Promise<boolean> => recreateSession(state, identity),
    ),
  );
  return results.every(Boolean);
}

function disposeSessions(state: ExtensionState): void {
  ensureSessionState(state);
  for (const key of state.lifecycle.sessionGenerations.keys()) {
    incrementGeneration(state, key);
  }
  for (const { session } of state.sessions.values()) {
    session.disposeSession();
  }
  state.sessions.clear();
}

function sessionIdentities(): SessionIdentity[] {
  return [
    { key: globalSessionKey, resource: undefined },
    ...(workspace.workspaceFolders ?? []).map(
      ({ uri }): { key: string; resource: Uri } => ({
        key: workspaceSessionKey(uri),
        resource: uri,
      }),
    ),
  ];
}

function sessionIdentityForResource(
  resource: Uri | undefined,
): SessionIdentity {
  let workspaceFolder: WorkspaceFolder | undefined;
  if (resource) {
    workspaceFolder = workspace.getWorkspaceFolder(resource);
  }
  if (workspaceFolder) {
    return {
      key: workspaceSessionKey(workspaceFolder.uri),
      resource: workspaceFolder.uri,
    };
  }
  return { key: globalSessionKey, resource: undefined };
}

function workspaceSessionKey(resource: Uri): string {
  return `${workspaceSessionPrefix}${resource.toString()}`;
}

function synchronizeSessionIdentities(
  state: ExtensionState,
): SessionIdentity[] {
  ensureSessionState(state);
  const identities = sessionIdentities();
  const desiredKeys = new Set(identities.map(({ key }): string => key));
  for (const [key, entry] of state.sessions) {
    if (!desiredKeys.has(key)) {
      incrementGeneration(state, key);
      entry.session.disposeSession();
      state.sessions.delete(key);
    }
  }
  return identities;
}

async function recreateSession(
  state: ExtensionState,
  identity: SessionIdentity,
): SessionRecreation {
  ensureSessionState(state);
  const generation = incrementGeneration(state, identity.key);
  const previous = state.sessions.get(identity.key);
  state.sessions.delete(identity.key);
  previous?.session.disposeSession();
  const extensionPath = state.context?.extensionPath;
  if (!extensionPath) {
    return false;
  }

  const config = await sessionConfig(state, identity.resource);
  if (
    !ownsGeneration(state, identity.key, generation) ||
    state.context?.extensionPath !== extensionPath ||
    !sessionIdentities().some(({ key }): boolean => key === identity.key)
  ) {
    return false;
  }

  const session = loadNative(extensionPath).createSession(config);
  if (!ownsGeneration(state, identity.key, generation)) {
    session.disposeSession();
    return false;
  }
  const entry: ResourceSession = { resource: identity.resource, session };
  state.sessions.set(identity.key, entry);
  return true;
}

async function sessionConfig(
  state: ExtensionState,
  resource: Uri | undefined,
): Promise<NativeSessionConfig> {
  const indicators = suggestionIndicators(resource);
  const enabledProviders = configuredEnabledProviders(resource);
  const cacheDuration = cacheDurationMinutes(resource);
  const showVulnerabilities = configuredShowVulnerabilities(resource);
  return {
    ...optionalProperty("cacheDurationMinutes", cacheDuration),
    ...optionalProperty("enabledProviders", enabledProviders),
    http: await httpConfig(state, resource),
    providers: {
      dependencyProperties: dependencyProperties(resource),
      filePatterns: filePatterns(resource),
      prereleaseTagFilters: prereleaseTagFilters(resource),
      providerCache: providerCacheConfigs(resource),
      providerHttp: providerHttpConfigs(resource),
      registryUrls: registryUrls(resource),
    },
    showPrereleases: state.flags.showPrereleases,
    showSuggestionStats: state.flags.showSuggestionStats,
    ...optionalProperty("showVulnerabilities", showVulnerabilities),
    ...optionalProperty("suggestionIndicators", indicators),
  };
}

function incrementGeneration(state: ExtensionState, key: string): number {
  const generation = (state.lifecycle.sessionGenerations.get(key) ?? 0) + 1;
  state.lifecycle.sessionGenerations.set(key, generation);
  return generation;
}

function ownsGeneration(
  state: ExtensionState,
  key: string,
  generation: number,
): boolean {
  return state.lifecycle.sessionGenerations.get(key) === generation;
}

function ensureSessionState(state: ExtensionState): void {
  state.sessions ??= new Map();
  state.lifecycle.sessionGenerations ??= new Map();
}

export {
  disposeSessions,
  globalSessionKey,
  recreateAffectedSessions,
  recreateSessions,
  sessionForResource,
  synchronizeWorkspaceSessions,
};
