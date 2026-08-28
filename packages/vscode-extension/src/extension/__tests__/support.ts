import { expect } from "./runtime.ts";

interface CommandRegistry {
  [command: string]: (...args: unknown[]) => unknown;
}
interface CommandsMock {
  executeCommand: () => undefined;
  registerCommand: (
    command: string,
    callback: (...args: unknown[]) => unknown,
  ) => { dispose: () => undefined };
}
interface AuthContext {
  extensionPath: string;
  secrets: {
    get: (key: string) => string | undefined | Promise<string | undefined>;
    store: (key: string, value: string) => void;
  };
  storageUri: { path: string };
  workspaceState: {
    get: (key: string, fallback: unknown) => unknown;
    update: (key: string, value: unknown) => void;
  };
}
interface TestGlobals {
  __versionLensAppliedEdits?: unknown[];
  __versionLensRegisteredCommands?: Record<
    string,
    (...args: unknown[]) => unknown
  >;
}
interface AuthContextState {
  secretValues: Record<string, string | undefined>;
  storedSecrets: { key: string; value: string }[];
  updatedSettings: { key: string; target: boolean; value: unknown }[];
  workspaceValues: Record<string, unknown>;
}

function createCommandsMock(registry: CommandRegistry): CommandsMock {
  return {
    executeCommand: (): undefined => undefined,
    registerCommand(
      command: string,
      callback: (...args: unknown[]) => unknown,
    ): { dispose: () => undefined } {
      registry[command] = async (...args: unknown[]): Promise<unknown> =>
        callback(...args);
      return { dispose: (): undefined => undefined };
    },
  };
}

function createAuthContext(state: AuthContextState): AuthContext {
  return {
    extensionPath: "/test/extension",
    secrets: {
      get: (key: string): string | undefined => state.secretValues[key],
      store(key: string, value: string): void {
        state.secretValues[key] = value;
        state.storedSecrets.push({ key, value });
      },
    },
    storageUri: { path: "/workspace/.vscode" },
    workspaceState: {
      get: (key: string, fallback: unknown): unknown =>
        state.workspaceValues[key] ?? fallback,
      update: (key: string, value: unknown): void => {
        state.workspaceValues[key] = value;
        state.updatedSettings.push({ key, target: false, value });
      },
    },
  };
}

function expectCustomAuthenticationSetting(
  setting: unknown,
  registryUrl: string,
): void {
  expect(setting).toMatchObject({
    key: "UrlAuthenticationStore",
    target: false,
    value: {
      [registryUrl]: {
        label: "Custom Value",
        protocol: "https:",
        scheme: "Custom",
        status: "NoStatus",
        url: registryUrl,
      },
    },
  });
}

const testGlobals = globalThis as typeof globalThis & TestGlobals;
function ensureRegisteredCommands(): Record<
  string,
  (...args: unknown[]) => unknown
> {
  const existing = testGlobals.__versionLensRegisteredCommands;
  if (existing) {
    return existing;
  }
  const created: Record<string, (...args: unknown[]) => unknown> = {};
  testGlobals.__versionLensRegisteredCommands = created;
  return created;
}

function ensureAppliedEdits(): unknown[] {
  const existing = testGlobals.__versionLensAppliedEdits;
  if (existing) {
    return existing;
  }
  const created: unknown[] = [];
  testGlobals.__versionLensAppliedEdits = created;
  return created;
}

const registeredCommands: Record<string, (...args: unknown[]) => unknown> =
  ensureRegisteredCommands();
const appliedEdits: unknown[] = ensureAppliedEdits();

export type { CommandsMock };
export {
  appliedEdits,
  createAuthContext,
  createCommandsMock,
  expectCustomAuthenticationSetting,
  registeredCommands,
};
