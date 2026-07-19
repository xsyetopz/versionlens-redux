import { expect, it } from "../../runtime.ts";

import { authContext } from "./auth.ts";

import { commandState, documentStub } from "./state.ts";

import {
  appliedEdits,
  applyTestState,
  authorizationSecret,
  createdNativeSessions,
  createdSessionConfigs,
  inputPrompts,
  inputValues,
  quickPickValues,
  registeredCommand,
  registryUrl,
  reset,
  storedSecrets,
  updatedConfig,
  warningMessages,
} from "./support.ts";

interface AuthorizationResult {
  authorizationRequiredCount: number;
  authorizationRequiredRequests: Array<{
    authUrl: string;
    requestUrl: string;
  }>;
  edits: never[];
  vulnerableUpdateCount: number;
}
interface SuccessfulResult {
  authorizationRequiredCount: number;
  authorizationRequiredRequests: never[];
  edits: Array<{
    newText: string;
    range: {
      end: { character: number; line: number };
      start: { character: number; line: number };
    };
  }>;
  vulnerableUpdateCount: number;
}

interface SuccessfulReloadedSession {
  analyzeDocument: () => undefined;
  applyCommand: (input: unknown) => SuccessfulResult;
  clearCache: () => undefined;
  disposeSession: () => undefined;
  resolveDocument: () => undefined;
}
interface AuthorizationRequiredSession {
  applyCommand: (input: unknown) => AuthorizationResult;
  disposeSession: () => undefined;
}

function authorizationRequiredSession(
  applyInputs: unknown[],
): AuthorizationRequiredSession {
  const recordedInputs = applyInputs;
  return {
    applyCommand: (input: unknown): AuthorizationResult => {
      recordedInputs.push(input);
      return {
        authorizationRequiredCount: 1,
        authorizationRequiredRequests: [
          {
            authUrl: registryUrl,
            requestUrl: `${registryUrl}/private-package`,
          },
        ],
        edits: [],
        vulnerableUpdateCount: 0,
      };
    },
    disposeSession: (): undefined => undefined,
  };
}

function successfulReloadedSession(
  applyInputs: unknown[],
): SuccessfulReloadedSession {
  return {
    analyzeDocument: (): undefined => undefined,
    applyCommand: (input: unknown): SuccessfulResult => {
      applyInputs.push(input);
      return {
        authorizationRequiredCount: 0,
        authorizationRequiredRequests: [],
        edits: [
          {
            newText: "1.1.0",
            range: {
              end: { character: 42, line: 0 },
              start: { character: 37, line: 0 },
            },
          },
        ],
        vulnerableUpdateCount: 0,
      };
    },
    clearCache: (): undefined => undefined,
    disposeSession: (): undefined => undefined,
    resolveDocument: (): undefined => undefined,
  };
}

it("resolve command offers authentication when registry auth is required", async (): Promise<void> => {
  const { registerCommands } = await import("../../../commands/register.ts");
  reset();
  applyTestState.warningChoice = "Add Authentication";
  inputValues.push(registryUrl, "Bearer token");
  quickPickValues.push({ label: "Custom Value", providerScheme: "Custom" });
  const document = documentStub("private-package");
  const session = {
    disposeSession: (): undefined => undefined,
    applyCommand: (): {
      authorizationRequiredCount: number;
      authorizationRequiredRequests: Array<{
        authUrl: string;
        requestUrl: string;
      }>;
      edits: never[];
      vulnerableUpdateCount: number;
    } => ({
      authorizationRequiredCount: 1,
      authorizationRequiredRequests: [
        {
          authUrl: registryUrl,
          requestUrl: `${registryUrl}/private-package`,
        },
      ],
      edits: [],
      vulnerableUpdateCount: 0,
    }),
  };

  applyTestState.activeTextEditor = { document };
  registerCommands(
    commandState(session, {
      context: authContext(),
    }) as never,
  );
  await registeredCommand("versionlens.suggestion.onUpdateDependency")(
    "private-package",
  );

  expect(warningMessages).toHaveLength(1);
  expect(inputPrompts[0]).toMatchObject({ value: registryUrl });
  expect(storedSecrets).toEqual([
    {
      key: authorizationSecret,
      value: "Bearer token",
    },
  ]);
  expect(updatedConfig[0]).toMatchObject({
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
  expect(appliedEdits).toEqual([]);
});

it("resolve command retries and applies edits after adding authentication", async (): Promise<void> => {
  const { registerCommands } = await import("../../../commands/register.ts");
  reset();
  applyTestState.warningChoice = "Add Authentication";
  inputValues.push(registryUrl, "Bearer token");
  quickPickValues.push({ label: "Custom Value", providerScheme: "Custom" });
  const document = documentStub("private-package");
  const applyInputs: unknown[] = [];
  const session = authorizationRequiredSession(applyInputs);
  createdNativeSessions.push(successfulReloadedSession(applyInputs));

  applyTestState.activeTextEditor = { document };
  registerCommands(
    commandState(session, {
      context: authContext(),
    }) as never,
  );
  await registeredCommand("versionlens.suggestion.onUpdateDependency")(
    "private-package",
  );

  expect(applyInputs).toHaveLength(2);
  expect(applyInputs[1]).toEqual(applyInputs[0]);
  expect(appliedEdits).toEqual([expect.objectContaining({ newText: "1.1.0" })]);
});

it("add auth command reloads the native session with stored headers", async (): Promise<void> => {
  const { registerCommands } = await import("../../../commands/register.ts");
  reset();
  inputValues.push(registryUrl, "secret-token");
  quickPickValues.push({ label: "Custom Value", providerScheme: "Custom" });
  let clearCacheCount = 0;

  registerCommands(
    commandState(
      {
        clearCache: (): void => {
          clearCacheCount += 1;
        },
        disposeSession: (): undefined => undefined,
      },
      {
        context: authContext(),
      },
    ) as never,
  );
  await registeredCommand("versionlens.editor.onAddUrlAuthentication")();

  expect(storedSecrets).toEqual([
    {
      key: authorizationSecret,
      value: "secret-token",
    },
  ]);
  expect(createdSessionConfigs[0]).toMatchObject({
    http: {
      authHeaders: [
        {
          name: "Authorization",
          url: registryUrl,
          value: "secret-token",
        },
      ],
    },
  });
  expect(clearCacheCount).toBe(1);
  expect(applyTestState.codeLensRefreshCount).toBe(1);
});
