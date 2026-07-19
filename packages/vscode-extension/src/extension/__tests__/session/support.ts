import { mock, mockVscodeHost } from "../runtime.ts";

type MockModule = Record<string, unknown>;

type ConfigurationValue =
  | boolean
  | number
  | null
  | Record<string, string>
  | { name: string; secret: string; url?: string }[]
  | string
  | string[]
  | undefined;
interface SessionConfiguration extends Record<string, ConfigurationValue> {
  cacheTtlSeconds?: number;
  enabledProviders?: string[];
  proxy?: string;
  showPrereleases?: boolean;
}
interface TestUri {
  fsPath: string;
  path: string;
  scheme: string | undefined;
  toString: () => string;
}

const configured: SessionConfiguration = {};
let createdSessionConfig: unknown;
let createdSessionCount = 0;
const createdSessionConfigs: unknown[] = [];
const disposedNativeSessions = new Set<unknown>();
const configuredByResource = new Map<string, typeof configured>();
let workspaceFolders: { uri: ReturnType<typeof uri> }[] | undefined;

function resetCreatedSessionCount(): void {
  createdSessionCount = 0;
}

function setWorkspaceFolders(
  value: { uri: ReturnType<typeof uri> }[] | undefined,
): void {
  workspaceFolders = value;
}

function uri(value: string): TestUri {
  const uriValue = value;
  return {
    fsPath: uriValue.replace("file://", ""),
    path: uriValue.replace("file://", ""),
    scheme: uriValue.split(":")[0],
    toString: (): string => uriValue,
  };
}

mockVscodeHost(
  (): MockModule => ({
    workspace: {
      getConfiguration(
        _section?: string,
        resource?: { toString: () => string },
      ): {
        get: (key: string, fallback?: unknown) => unknown;
        inspect: (key: string) =>
          | {
              workspaceValue:
                | string
                | number
                | boolean
                | string[]
                | Record<string, string>
                | Array<{ name: string; secret: string; url?: string }>
                | null;
            }
          | undefined;
      } {
        let values = configured;
        if (resource) {
          values = configuredByResource.get(resource.toString()) ?? configured;
        }
        return {
          get(key: string, fallback?: unknown): unknown {
            if (Object.hasOwn(values, key)) {
              return values[key];
            }
            return fallback;
          },
          inspect(key: string):
            | {
                workspaceValue:
                  | string
                  | number
                  | boolean
                  | string[]
                  | Record<string, string>
                  | Array<{ name: string; secret: string; url?: string }>
                  | null;
              }
            | undefined {
            const value = values[key];
            if (value === undefined) {
              return;
            }
            return { workspaceValue: value };
          },
        };
      },
      get workspaceFolders():
        | Array<{ uri: ReturnType<typeof uri> }>
        | undefined {
        return workspaceFolders;
      },
      getWorkspaceFolder(resource: {
        fsPath?: string;
      }): { uri: ReturnType<typeof uri> } | undefined {
        return workspaceFolders?.find(({ uri: folder }): boolean | undefined =>
          resource.fsPath?.startsWith(`${folder.fsPath}/`),
        );
      },
    },
  }),
);

mock.module(
  "../../native/module.ts",
  (): MockModule => ({
    loadNative(): {
      createSession: (config: unknown) => {
        analyzeDocument: () => undefined;
        applyCommand: () => undefined;
        clearCache: () => undefined;
        disposeSession: () => Set<unknown>;
        resolveDocument: () => undefined;
      };
    } {
      return {
        createSession(config: unknown): {
          analyzeDocument: () => undefined;
          applyCommand: () => undefined;
          clearCache: () => undefined;
          disposeSession: () => Set<unknown>;
          resolveDocument: () => undefined;
        } {
          createdSessionConfig = config;
          createdSessionConfigs.push(config);
          createdSessionCount += 1;
          const session = {
            analyzeDocument: (): undefined => undefined,
            applyCommand: (): undefined => undefined,
            clearCache: (): undefined => undefined,
            disposeSession: (): Set<unknown> =>
              disposedNativeSessions.add(session),
            resolveDocument: (): undefined => undefined,
          };
          return session;
        },
      };
    },
  }),
);

export {
  configured,
  configuredByResource,
  createdSessionConfig,
  createdSessionConfigs,
  createdSessionCount,
  disposedNativeSessions,
  resetCreatedSessionCount,
  setWorkspaceFolders,
  uri,
};
