import { expect, it } from "../runtime.ts";

import "./support.ts";
import {
  createExtensionState,
  diagnosticState,
  documentStub,
  outputFor,
  reset,
} from "./state.ts";

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

  expect(
    currentState.snapshots.editedDependencies.get(document.uri.toString()),
  ).toBe(document.uri.toString());
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

  expect(
    currentState.snapshots.editedDependencies.get(document.uri.toString()),
  ).toBe(document.uri.toString());
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
        analyzeDocument(): {
          canSortDependencies: boolean;
          codeLenses: never[];
          dependencies: never[];
          dependencySignature: string;
          diagnostics: never[];
          installTaskConfigKey: undefined;
          isSupportedManifest: boolean;
          status: {
            dependencyCount: number;
            errorCount: number;
            noMatchCount: number;
            text: string;
            tooltip: string;
            updateCount: number;
            visible: boolean;
            vulnerabilityCount: number;
          };
        } {
          analyzeCount += 1;
          return outputFor(document.uri.toString());
        },
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
