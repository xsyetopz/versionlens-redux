import { expect, it } from "../runtime.ts";

import "./support.ts";
import {
  type AnalyzeOutput,
  authContext,
  authorizationSecret,
  createExtensionState,
  diagnosticState,
  documentStub,
  outputFor,
  type ResolveOutput,
  registryUrl,
  reset,
  type SessionStub,
} from "./state.ts";

function authorizationSession(
  documentUri: string,
  onResolve: () => void,
): SessionStub {
  return {
    analyzeDocument: (): AnalyzeOutput => outputFor(documentUri),
    disposeSession: (): undefined => undefined,
    resolveDocument: (): ResolveOutput => {
      onResolve();
      return {
        authorizationRequiredCount: 1,
        authorizationRequiredRequests: [
          {
            authUrl: registryUrl,
            requestUrl: `${registryUrl}/private-package`,
          },
        ],
        edits: [],
        suggestions: [],
        vulnerableUpdateCount: 0,
      };
    },
  };
}

it("diagnostic refresh suppresses repeated auth prompts after cancellation", async (): Promise<void> => {
  const { refreshDiagnostics } = await import("../../diagnostics/refresh.ts");
  reset();
  Object.assign(
    diagnosticState.configurationAuth.workspaceValues,
    Object.fromEntries([
      [
        "UrlAuthenticationStore",
        {
          [registryUrl]: {
            protocol: "https:",
            scheme: "NotSet",
            status: "User cancelled",
            url: registryUrl,
          },
        },
      ],
    ]),
  );
  const document = documentStub("file:///workspace/package.json");
  let resolveCount = 0;
  await refreshDiagnostics(
    createExtensionState({
      context: authContext(),
      session: authorizationSession(document.uri.toString(), (): void => {
        resolveCount += 1;
      }),
    }) as never,
    document as never,
  );

  expect(resolveCount).toBe(1);
  expect(diagnosticState.userInteraction.warningMessages).toEqual([]);
  expect(diagnosticState.userInteraction.inputPrompts).toEqual([]);
  expect(diagnosticState.configurationAuth.updatedSettings).toEqual([]);
});

it("diagnostic refresh offers authentication when registry auth is required", async (): Promise<void> => {
  const { refreshDiagnostics } = await import("../../diagnostics/refresh.ts");
  reset();
  diagnosticState.userInteraction.warningChoice = "Add Authentication";
  diagnosticState.userInteraction.inputValues.push(registryUrl, "Bearer token");
  diagnosticState.userInteraction.quickPickValues.push({
    label: "Custom Value",
    providerScheme: "Custom",
  });
  let initialResolveCount = 0;
  const document = documentStub("file:///workspace/package.json");
  await refreshDiagnostics(
    createExtensionState({
      context: authContext(),
      session: authorizationSession(document.uri.toString(), (): void => {
        initialResolveCount += 1;
      }),
    }) as never,
    document as never,
  );

  expect(diagnosticState.userInteraction.warningMessages).toHaveLength(1);
  expect(diagnosticState.userInteraction.inputPrompts[0]).toMatchObject({
    value: registryUrl,
  });
  expect(diagnosticState.configurationAuth.storedSecrets).toEqual([
    {
      key: authorizationSecret,
      value: "Bearer token",
    },
  ]);
  expect(diagnosticState.configurationAuth.updatedSettings[0]).toMatchObject({
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
  expect(
    diagnosticState.diagnosticSession.createdSessionConfigs[0],
  ).toMatchObject({
    http: {
      authHeaders: [
        {
          name: "Authorization",
          url: registryUrl,
          value: "Bearer token",
        },
      ],
    },
  });
  expect(initialResolveCount).toBe(1);
  expect(diagnosticState.diagnosticSession.reloadedResolveCount).toBe(1);
});
