import { expect, it } from "../../runtime.ts";

import { commandState, documentStub } from "./state.ts";

import {
  appliedEdits,
  applyTestState,
  outputLines,
  registeredCommand,
  reset,
} from "./support.ts";

let releaseApplyEdit: (() => void) | undefined;

it("vulnerability confirmation rejects invalid native edit ranges", async (): Promise<void> => {
  const { registerCommands } = await import("../../../commands/register.ts");
  reset();
  applyTestState.warningChoice = "Update Anyway";
  applyTestState.activeTextEditor = { document: documentStub("left-pad") };
  registerCommands(
    commandState({
      applyCommand: (): {
        edits: Array<{
          newText: string;
          range: {
            end: { character: number; line: number };
            start: { character: number; line: number };
          };
        }>;
        vulnerableUpdateCount: number;
      } => ({
        edits: [
          {
            newText: "1.1.0",
            range: {
              end: { character: 999, line: 0 },
              start: { character: 30, line: 0 },
            },
          },
        ],
        vulnerableUpdateCount: 1,
      }),
    }) as never,
  );
  await registeredCommand("versionlens.suggestion.onUpdateDependency")(
    "left-pad",
  );

  expect(appliedEdits).toEqual([]);
});

it("resolve command ignores reentry while an edit is pending", async (): Promise<void> => {
  const { registerCommands } = await import("../../../commands/register.ts");
  reset();
  const document = documentStub("left-pad");
  const applyInputs: unknown[] = [];
  applyTestState.applyEditBlocker = new Promise((resolve): void => {
    releaseApplyEdit = (): void => resolve(true);
  });
  const session = {
    applyCommand: (
      input: unknown,
    ): {
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
    } => {
      applyInputs.push(input);
      return {
        authorizationRequiredCount: 0,
        authorizationRequiredRequests: [],
        edits: [
          {
            newText: "1.1.0",
            range: {
              end: { character: 35, line: 0 },
              start: { character: 30, line: 0 },
            },
          },
        ],
        vulnerableUpdateCount: 0,
      };
    },
  };

  applyTestState.activeTextEditor = { document };
  registerCommands(commandState(session) as never);
  const first = registeredCommand("versionlens.suggestion.onUpdateDependency")(
    "left-pad",
  );
  await Promise.resolve();
  const second = registeredCommand("versionlens.suggestion.onUpdateDependency")(
    "left-pad",
  );
  await Promise.resolve();

  expect(applyInputs).toHaveLength(1);

  releaseApplyEdit?.();
  await first;
  await second;
});

it("resolve command logs native failures without applying edits", async (): Promise<void> => {
  const { registerCommands } = await import("../../../commands/register.ts");
  reset();
  const consoleError = console.error;
  console.error = (): undefined => undefined;
  const document = documentStub("left-pad");
  const session = {
    applyCommand: (): never => {
      throw new Error("native exploded");
    },
  };

  applyTestState.activeTextEditor = { document };
  registerCommands(
    commandState(session, {
      ui: {
        outputChannel: {
          appendLine(value: string): void {
            outputLines.push(value);
          },
        },
      },
    }) as never,
  );
  try {
    await registeredCommand("versionlens.suggestion.onUpdateDependency")(
      "left-pad",
    );

    expect(outputLines[0]).toContain("native exploded");
    expect(appliedEdits).toEqual([]);
  } finally {
    console.error = consoleError;
  }
});

it("resolve command forwards CodeLens-selected update commands", async (): Promise<void> => {
  const { registerCommands } = await import("../../../commands/register.ts");
  reset();
  const applyInputs: unknown[] = [];
  const selector = "1.2.3\u001f0:12,0:17";
  const session = {
    applyCommand: (
      input: unknown,
    ): {
      authorizationRequiredCount: number;
      authorizationRequiredRequests: never[];
      edits: never[];
      vulnerableUpdateCount: number;
    } => {
      applyInputs.push(input);
      return {
        authorizationRequiredCount: 0,
        authorizationRequiredRequests: [],
        edits: [],
        vulnerableUpdateCount: 0,
      };
    },
  };

  applyTestState.activeTextEditor = { document: documentStub("version") };
  registerCommands(commandState(session) as never);
  await registeredCommand("versionlens.suggestion.onUpdateDependency")(
    "1.2.3",
    selector,
    "updateMajor",
    "2.0.0",
  );

  expect(applyInputs[0]).toMatchObject({
    command: "updateMajor",
    dependencyName: selector,
    selectedVersion: "2.0.0",
  });
});

it("bulk update native failure decrements provider busy once like upstream", async (): Promise<void> => {
  const { registerCommands } = await import("../../../commands/register.ts");
  reset();
  const document = documentStub("left-pad");
  const state = commandState(
    {
      applyCommand(): never {
        throw new Error("native exploded");
      },
    },
    {
      flags: {
        providerBusy: 2,
        providerError: false,
        codeLensReplace: true,
        showPrereleases: false,
        showSuggestionStats: false,
        showVersionLenses: true,
      },
      ui: {
        codeLensRefresh: undefined,
        diagnostics: undefined,
        outputChannel: { appendLine: (): undefined => undefined },
      },
    },
  );

  applyTestState.activeTextEditor = { document };
  registerCommands(state as never);
  await registeredCommand("versionlens.editor.onUpdateDependenciesLatest")();

  expect(state.flags.providerBusy).toBe(2);
  expect(state.flags.providerError).toBe(true);
});
