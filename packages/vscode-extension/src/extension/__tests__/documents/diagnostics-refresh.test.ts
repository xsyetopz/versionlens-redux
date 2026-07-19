import { expect, it } from "../runtime.ts";

import {
  type analysisOutput,
  codeLensProviders,
  diagnosticsSets,
  registeredCommands,
  setActiveTextEditor,
  testDocument,
} from "./support.ts";

const vulnerabilityStart = 5;
const vulnerabilityEnd = 20;
const commandLensOutput: ReturnType<typeof analysisOutput> = {
  canSortDependencies: true,
  codeLenses: [
    {
      arguments: [
        "left-pad",
        "left-pad\u001f0:30,0:35",
        "updateMajor",
        "2.0.0",
      ],
      command: "versionlens.suggestion.onUpdateDependency",
      range: {
        end: { character: vulnerabilityEnd, line: 0 },
        start: { character: vulnerabilityStart, line: 0 },
      },
      title: "left-pad major 2.0.0",
    },
  ],
  dependencies: [],
  dependencySignature: "left-pad@1.0.0",
  diagnostics: [],
  isSupportedManifest: true,
  status: { text: "Version Lens", tooltip: "1 update", visible: true },
};
const vulnerabilityOutput: ReturnType<typeof analysisOutput> & {
  suggestions: never[];
} = {
  canSortDependencies: true,
  codeLenses: [],
  dependencies: [],
  dependencySignature: "left-pad@1.0.0",
  diagnostics: [
    {
      code: "OSV-1",
      codeDescriptionUrl: "https://osv.dev/vulnerability/OSV-1",
      message: "Vulnerability found in left-pad@1.0.0:\nOSV-1",
      range: {
        end: { character: vulnerabilityEnd, line: 0 },
        start: { character: vulnerabilityStart, line: 0 },
      },
      severity: 0,
      source: "VersionLens",
    },
  ],
  isSupportedManifest: true,
  status: { text: "Version Lens", tooltip: "1 issue", visible: true },
  suggestions: [],
};

interface CommandSession {
  analyzeDocument: () => typeof commandLensOutput;
  applyCommand: (input: unknown) => {
    authorizationRequiredCount: number;
    authorizationRequiredRequests: never[];
    edits: never[];
    vulnerableUpdateCount: number;
  };
  resolveDocument: () => { edits: never[]; suggestions: never[] };
}

interface DiagnosticResolution {
  authorizationRequiredCount: number;
  authorizationRequiredRequests: never[];
  edits: never[];
  suggestions: never[];
}

interface VulnerabilityDiagnosticsState {
  flags: {
    providerBusy: number;
    providerError: boolean;
    showVersionLenses: boolean;
  };
  sessions: Map<
    string,
    {
      resource: undefined;
      session: {
        analyzeDocument: () => typeof vulnerabilityOutput;
        resolveDocument: () => DiagnosticResolution;
      };
    }
  >;
  snapshots: {
    editedDependencies: Map<unknown, unknown>;
    savedDependencies: Map<string, string>;
  };
  ui: {
    diagnostics: {
      set: (uri: unknown, diagnostics: unknown[]) => void;
    };
  };
}

function vulnerabilityDiagnosticsState(
  onResolve: () => void,
): VulnerabilityDiagnosticsState {
  return {
    flags: {
      providerBusy: 0,
      providerError: false,
      showVersionLenses: true,
    },
    sessions: new Map([
      [
        "global",
        {
          resource: undefined,
          session: {
            resolveDocument: (): DiagnosticResolution => {
              onResolve();
              return {
                authorizationRequiredCount: 0,
                authorizationRequiredRequests: [],
                edits: [],
                suggestions: [],
              };
            },
            analyzeDocument: (): typeof vulnerabilityOutput =>
              vulnerabilityOutput,
          },
        },
      ],
    ]),
    snapshots: { editedDependencies: new Map(), savedDependencies: new Map() },
    ui: {
      diagnostics: {
        set(uri: unknown, diagnostics: unknown[]): void {
          diagnosticsSets.push({ diagnostics, uri });
        },
      },
    },
  };
}

function commandSession(applyInputs: unknown[]): CommandSession {
  const recordedInputs = applyInputs;
  return {
    analyzeDocument: (): typeof commandLensOutput => commandLensOutput,
    applyCommand: (
      input: unknown,
    ): {
      authorizationRequiredCount: number;
      authorizationRequiredRequests: never[];
      edits: never[];
      vulnerableUpdateCount: number;
    } => {
      recordedInputs.push(input);
      return {
        authorizationRequiredCount: 0,
        authorizationRequiredRequests: [],
        edits: [],
        vulnerableUpdateCount: 0,
      };
    },
    resolveDocument: () => ({ edits: [], suggestions: [] }),
  };
}

it("code lens command argument carries native payload through the CodeLens object", async (): Promise<void> => {
  const { registerCommands } = await import("../../commands/register.ts");
  const { registerCodeLensProvider } = await import(
    "../../commands/codelens.ts"
  );
  codeLensProviders.length = 0;
  for (const command of Object.keys(registeredCommands)) {
    delete registeredCommands[command];
  }
  const applyInputs: unknown[] = [];
  const document = testDocument();
  setActiveTextEditor({ document });
  const state = {
    flags: {
      codeLensReplace: true,
      providerBusy: 0,
      providerError: false,
      showVersionLenses: true,
    },
    sessions: new Map([
      [
        "global",
        {
          resource: undefined,
          session: commandSession(applyInputs),
        },
      ],
    ]),
    snapshots: { editedDependencies: new Map(), savedDependencies: new Map() },
    ui: { codeLensRefresh: undefined },
  };

  registerCommands(state as never);
  registerCodeLensProvider(state as never);
  const lenses = await Promise.resolve(
    codeLensProviders[0]?.provideCodeLenses(document),
  );
  const lens = lenses?.[0];
  if (!lens) {
    throw new Error("expected a CodeLens");
  }
  const { command } = lens as {
    command?: { arguments?: unknown[]; command: string };
  };
  if (!command) {
    throw new Error("expected a CodeLens command");
  }

  expect(command.arguments).toEqual([lens]);
  await registeredCommands[command.command]?.(command.arguments?.[0]);

  expect(applyInputs[0]).toMatchObject({
    command: "updateMajor",
    dependencyName: "left-pad\u001f0:30,0:35",
    selectedVersion: "2.0.0",
  });
});

it("code lens provider exposes a refresh event", async (): Promise<void> => {
  const { refreshCodeLenses, registerCodeLensProvider } = await import(
    "../../commands/codelens.ts"
  );
  codeLensProviders.length = 0;
  const state = {
    flags: { showVersionLenses: true },
    sessions: new Map(),
    ui: { codeLensRefresh: undefined },
  };

  registerCodeLensProvider(state as never);
  let refreshCount = 0;
  codeLensProviders[0]?.onDidChangeCodeLenses?.((): void => {
    refreshCount += 1;
  });
  refreshCodeLenses(state as never);

  expect(refreshCount).toBe(1);
});

it("refreshDiagnostics renders upstream vulnerability diagnostics without status UI", async (): Promise<void> => {
  const { refreshDiagnostics } = await import("../../diagnostics/refresh.ts");
  diagnosticsSets.length = 0;
  let resolveDocumentCount = 0;
  const document = {
    ...testDocument(),
    isDirty: false,
  };
  setActiveTextEditor({ document });
  const state = vulnerabilityDiagnosticsState((): void => {
    resolveDocumentCount += 1;
  });

  await refreshDiagnostics(state as never, document as never);

  expect(diagnosticsSets).toHaveLength(1);
  expect(diagnosticsSets[0]?.diagnostics[0]).toMatchObject({
    code: {
      target: { value: "https://osv.dev/vulnerability/OSV-1" },
      value: "OSV-1",
    },
    message: "Vulnerability found in left-pad@1.0.0:\nOSV-1",
    range: { values: [0, vulnerabilityStart, 0, vulnerabilityEnd] },
    severity: 0,
    source: "VersionLens",
  });
  expect(diagnosticsSets[0]?.diagnostics[0]).not.toHaveProperty(
    "codeDescription",
  );
  expect(resolveDocumentCount).toBe(1);
  expect(state.snapshots.savedDependencies.get("file:///package.json")).toBe(
    "left-pad@1.0.0",
  );
});
