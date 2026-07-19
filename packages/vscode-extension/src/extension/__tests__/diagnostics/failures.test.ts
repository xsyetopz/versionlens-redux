import { expect, it } from "../runtime.ts";

import "./support.ts";
import {
  type AnalyzeOutput,
  createExtensionState,
  diagnosticState,
  documentStub,
  outputFor,
  type ResolveOutput,
  reset,
} from "./state.ts";

it("analyze failure decrements only its provider busy operation", async (): Promise<void> => {
  const { analyzeDocument } = await import("../../diagnostics/analyze.ts");
  reset();
  const document = documentStub("file:///workspace/package.json");
  const currentState = createExtensionState({
    flags: {
      codeLensReplace: true,
      providerBusy: 2,
      providerError: false,
      showOutdated: false,
      showPrereleases: false,
      showSuggestionStats: false,
      showVersionLenses: true,
    },
    session: {
      analyzeDocument(): never {
        throw new Error("provider failed");
      },
    },
  });

  analyzeDocument(currentState as never, document as never);

  expect(currentState.flags.providerBusy).toBe(2);
  expect(currentState.flags.providerError).toBe(true);
});

it("resolve failure decrements only its provider busy operation", async (): Promise<void> => {
  const { resolveDocumentForDiagnostics } = await import(
    "../../diagnostics/resolve.ts"
  );
  reset();
  const document = documentStub("file:///workspace/package.json");
  const currentState = createExtensionState({
    flags: {
      codeLensReplace: true,
      providerBusy: 2,
      providerError: false,
      showOutdated: false,
      showPrereleases: false,
      showSuggestionStats: false,
      showVersionLenses: true,
    },
    session: {
      resolveDocument(): never {
        throw new Error("provider failed");
      },
    },
  });

  await resolveDocumentForDiagnostics(currentState as never, document as never);

  expect(currentState.flags.providerBusy).toBe(2);
  expect(currentState.flags.providerError).toBe(true);
});

it("stale session resolutions cannot publish diagnostics", async (): Promise<void> => {
  const { refreshDiagnostics } = await import("../../diagnostics/refresh.ts");
  reset();
  let finishResolution: ((value: unknown) => void) | undefined;
  const pendingResolution = new Promise((resolve): void => {
    finishResolution = resolve;
  });
  const document = documentStub("file:///workspace/race-package.json");
  const olderSession = {
    analyzeDocument: (): AnalyzeOutput => outputFor(document.uri.toString()),
    resolveDocument: (): Promise<unknown> => pendingResolution,
  };
  const currentState = createExtensionState({ session: olderSession });

  const refresh = refreshDiagnostics(currentState as never, document as never);
  await Promise.resolve();
  currentState.sessions.set("global", {
    resource: undefined,
    session: {
      analyzeDocument: (): AnalyzeOutput => outputFor(document.uri.toString()),
      resolveDocument: (): ResolveOutput => ({
        authorizationRequiredCount: 0,
        authorizationRequiredRequests: [],
        edits: [],
        suggestions: [],
        vulnerableUpdateCount: 0,
      }),
    },
  });
  finishResolution?.({
    authorizationRequiredCount: 0,
    authorizationRequiredRequests: [],
    edits: [],
    suggestions: [],
  });
  await refresh;

  expect(diagnosticState.diagnosticSession.diagnosticsSets).toEqual([]);
});
