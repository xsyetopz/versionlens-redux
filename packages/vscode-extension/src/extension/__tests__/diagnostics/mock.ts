import { DiagnosticMock } from "./constructors/diagnostic.ts";
import { RangeMock } from "./constructors/range.ts";
import type { TextDocumentStub } from "./state.ts";
import { diagnosticState } from "./state.ts";

type MockModule = Record<string, unknown>;

function workspaceConfiguration(): MockModule {
  return {
    get(key: string, fallback?: unknown): unknown {
      if (
        Object.hasOwn(diagnosticState.configurationAuth.workspaceConfig, key)
      ) {
        return diagnosticState.configurationAuth.workspaceConfig[key];
      }
      return fallback;
    },
    inspect(key: string): { workspaceValue: unknown | null } | undefined {
      const value = diagnosticState.configurationAuth.workspaceConfig[key];
      let inspection: { workspaceValue: unknown | null } | undefined;
      if (value !== undefined) {
        inspection = { workspaceValue: value };
      }
      return inspection;
    },
    update(key: string, value: unknown, target: boolean): void {
      diagnosticState.configurationAuth.workspaceConfig[key] = value;
      diagnosticState.configurationAuth.updatedSettings.push({
        key,
        target,
        value,
      });
    },
  };
}

function createVscodeMock(): MockModule {
  return Object.fromEntries([
    ["Diagnostic", DiagnosticMock],
    ["Range", RangeMock],
    [
      "Uri",
      {
        parse: (
          value: string,
        ): { scheme: string | undefined; value: string } => ({
          scheme: value.split(":")[0],
          value,
        }),
      },
    ],
    ["commands", { executeCommand: (): undefined => undefined }],
    [
      "window",
      {
        get activeTextEditor(): { document: TextDocumentStub } | undefined {
          return diagnosticState.diagnosticSession.activeTextEditor;
        },
        showInputBox: (options: unknown): string | undefined => {
          diagnosticState.userInteraction.inputPrompts.push(options);
          return diagnosticState.userInteraction.inputValues.shift();
        },
        showQuickPick: (): unknown =>
          diagnosticState.userInteraction.quickPickValues.shift(),
        showWarningMessage: (...args: unknown[]): string | undefined => {
          diagnosticState.userInteraction.warningMessages.push(args);
          return diagnosticState.userInteraction.warningChoice;
        },
      },
    ],
    [
      "workspace",
      {
        getConfiguration: workspaceConfiguration,
        getWorkspaceFolder: (): undefined => undefined,
      },
    ],
  ]);
}

export { createVscodeMock };
