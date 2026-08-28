import { expect, it } from "../runtime.ts";

import "./support.ts";
import {
  analyzeDocumentStub,
  createExtensionState,
  diagnosticState,
  documentStub,
  type outputFor,
  reset,
} from "./state.ts";

function expectDirtyDocumentOutdated(
  state: ReturnType<typeof createExtensionState>,
  document: { uri: { toString: () => string } },
): void {
  const uri = document.uri.toString();
  expect(state.snapshots.editedDependencies.get(uri)).toBe(uri);
  expect(state.flags.showOutdated).toBe(true);
}

it("document refresh renders diagnostics without status bar side effects", async (): Promise<void> => {
  const { refreshDiagnostics } = await import("../../diagnostics/refresh.ts");
  reset();
  const active = documentStub("file:///workspace/package.json");
  const background = documentStub("file:///workspace/other/package.json");
  diagnosticState.diagnosticSession.activeTextEditor = { document: active };

  await refreshDiagnostics(createExtensionState() as never, active as never);
  await refreshDiagnostics(
    createExtensionState() as never,
    background as never,
  );

  expect(
    diagnosticState.diagnosticSession.diagnosticsSets.map(
      (entry): unknown => entry.uri,
    ),
  ).toEqual([active.uri, background.uri]);
});

it("dirty diagnostic refresh marks documents outdated when dependencies changed", async (): Promise<void> => {
  const { refreshDiagnostics } = await import("../../diagnostics/refresh.ts");
  reset();
  const document = {
    ...documentStub("file:///workspace/package.json"),
    isDirty: true,
  };
  const currentState = createExtensionState();
  currentState.snapshots.savedDependencies.set(
    document.uri.toString(),
    "previous-signature",
  );

  await refreshDiagnostics(currentState as never, document as never);

  expectDirtyDocumentOutdated(currentState, document);
  expect(currentState.flags.showOutdated).toBe(true);
});

it("dirty diagnostic refresh without saved baseline marks non-empty dependencies outdated", async (): Promise<void> => {
  const { refreshDiagnostics } = await import("../../diagnostics/refresh.ts");
  reset();
  const document = {
    ...documentStub("file:///workspace/package.json"),
    isDirty: true,
  };
  const currentState = createExtensionState();

  await refreshDiagnostics(currentState as never, document as never);

  expectDirtyDocumentOutdated(currentState, document);
  expect(currentState.flags.showOutdated).toBe(true);
});

it("diagnostic refresh is gated by visible version lenses", async (): Promise<void> => {
  const { refreshDiagnostics } = await import("../../diagnostics/refresh.ts");
  reset();
  const document = documentStub("file:///workspace/package.json");
  let analyzeCount = 0;

  await refreshDiagnostics(
    createExtensionState({
      flags: {
        codeLensReplace: true,
        providerBusy: 0,
        providerError: false,
        showPrereleases: false,
        showSuggestionStats: false,
        showVersionLenses: false,
      },
      session: {
        analyzeDocument: (): ReturnType<typeof outputFor> =>
          analyzeDocumentStub(document.uri.toString(), (): void => {
            analyzeCount += 1;
          }),
        resolveDocument: (): {
          authorizationRequiredCount: number;
          authorizationRequiredRequests: never[];
          edits: never[];
          suggestions: never[];
          vulnerableUpdateCount: number;
        } => ({
          authorizationRequiredCount: 0,
          authorizationRequiredRequests: [],
          edits: [],
          suggestions: [],
          vulnerableUpdateCount: 0,
        }),
      },
    }) as never,
    document as never,
  );

  expect(analyzeCount).toBe(0);
  expect(diagnosticState.diagnosticSession.diagnosticsSets).toEqual([]);
});
