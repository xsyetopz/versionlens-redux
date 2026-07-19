import {
  secretValues,
  storedSecrets,
  updatedConfig,
  workspaceValues,
} from "./support.ts";

function authContext(): {
  extensionPath: string;
  secrets: {
    get: (key: string) => Promise<string | undefined>;
    store: (key: string, value: string) => void;
  };
  storageUri: { path: string };
  workspaceState: {
    get: (key: string, fallback: unknown) => unknown;
    update: (key: string, value: unknown) => void;
  };
} {
  return {
    extensionPath: "/test/extension",
    secrets: {
      get: async (key: string): Promise<string | undefined> =>
        secretValues[key],
      store(key: string, value: string): void {
        secretValues[key] = value;
        storedSecrets.push({ key, value });
      },
    },
    storageUri: { path: "/workspace/.vscode" },
    workspaceState: {
      get: (key: string, fallback: unknown): unknown =>
        workspaceValues[key] ?? fallback,
      update: (key: string, value: unknown): void => {
        workspaceValues[key] = value;
        updatedConfig.push({ key, target: false, value });
      },
    },
  };
}

export { authContext };
