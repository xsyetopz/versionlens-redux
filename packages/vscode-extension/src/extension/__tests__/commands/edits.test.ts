import { readFileSync } from "node:fs";
import process from "node:process";
import { expect, it } from "../runtime.ts";

import { commandState } from "./command-state.ts";
import {
  appliedEdits,
  clearRegisteredCommands,
  completeTask,
  executedTasks,
  openedExternalUris,
  quickPickItems,
  quickPickOptions,
  registeredCommands,
  shownTextDocuments,
  smokeTaskLabel,
  testState,
  warningMessages,
} from "./support.ts";

const bulkCommandCount = 3;

interface BuildEditResult {
  edits: Array<{
    newText: string;
    range: {
      end: { character: number; line: number };
      start: { character: number; line: number };
    };
  }>;
  vulnerableUpdateCount: number;
}
interface BuildSelectionSession {
  applyCommand: (input: unknown) => BuildEditResult;
}

function buildSelectionSession(applyInputs: unknown[]): BuildSelectionSession {
  const recordedInputs = applyInputs;
  return {
    applyCommand(input: unknown): BuildEditResult {
      recordedInputs.push(input);
      return {
        edits: [
          {
            newText: "1.0.0+build.2",
            range: {
              end: { character: 43, line: 0 },
              start: { character: 30, line: 0 },
            },
          },
        ],
        vulnerableUpdateCount: 0,
      };
    },
  };
}

function buildDocument(): {
  getText: () => string;
  languageId: string;
  uri: { toString: () => string };
} {
  return {
    getText: (): string => packageFileFixture("package-left-pad-build.json"),
    languageId: "json",
    uri: { toString: (): string => "file:///package.json" },
  };
}

it("save task can retry while install is running or failed", async (): Promise<void> => {
  const { handleDidSaveTextDocument } = await import("../../tasks.ts");
  executedTasks.length = 0;
  testState.taskCompletionMode = "manual";
  const state = {
    flags: { showOutdated: false },
    snapshots: {
      editedDependencies: new Map<string, string>(),
      savedDependencies: new Map<string, string>(),
    },
  };
  const document = {
    uri: {
      scheme: "file",
      toString: (): string => "file:///retry-package.json",
    },
  };
  const key = "file:///retry-package.json";

  testState.analyzed = {
    activeProviderName: "npm",
    canSortDependencies: true,
    installTaskConfigKey: "npm.onSaveChanges",
    isSupportedManifest: true,
  };
  testState.dependencySnapshotValue = "left-pad@1";

  await handleDidSaveTextDocument(state as never, document as never);
  expect(state.snapshots.savedDependencies.get(key)).toBe("left-pad@1");

  state.snapshots.editedDependencies.set(key, "left-pad@2");
  const running = handleDidSaveTextDocument(state as never, document as never);
  await Promise.resolve();
  expect(executedTasks).toEqual([smokeTaskLabel]);
  expect(state.snapshots.savedDependencies.get(key)).toBe("left-pad@1");

  await handleDidSaveTextDocument(state as never, document as never);
  expect(executedTasks).toEqual([smokeTaskLabel]);
  expect(state.snapshots.savedDependencies.get(key)).toBe("left-pad@1");

  completeTask(smokeTaskLabel, 1);
  await running;
  expect(state.snapshots.savedDependencies.get(key)).toBe("left-pad@1");

  testState.taskCompletionMode = "auto";
  await handleDidSaveTextDocument(state as never, document as never);
  expect(executedTasks).toEqual([smokeTaskLabel, smokeTaskLabel]);
  expect(state.snapshots.savedDependencies.get(key)).toBe("left-pad@2");
});

it("resolve command applies Rust-produced edits", async (): Promise<void> => {
  const { registerCommands } = await import("../../commands/register.ts");
  const applyInputs: unknown[] = [];
  const startingRefreshCount = testState.refreshCount;
  appliedEdits.length = 0;
  clearRegisteredCommands();
  const document = {
    getText: (): string => packageFileFixture("package-left-pad.json"),
    languageId: "json",
    uri: { toString: (): string => "file:///package.json" },
  };
  const session = {
    applyCommand: (
      input: unknown,
    ): {
      edits: Array<{
        newText: string;
        range: {
          end: { character: number; line: number };
          start: { character: number; line: number };
        };
      }>;
    } => {
      applyInputs.push(input);
      return {
        edits: [
          {
            newText: "1.1.0",
            range: {
              end: { character: 35, line: 0 },
              start: { character: 30, line: 0 },
            },
          },
        ],
      };
    },
  };

  testState.activeTextEditor = { document };
  const state = commandState(session);
  registerCommands(state as never);
  await registeredCommands["versionlens.suggestion.onUpdateDependency"]?.(
    "left-pad",
    "left-pad\u001f0:30,0:35",
  );
  await registeredCommands["versionlens.editor.onSortDependencies"]?.();
  state.flags.codeLensReplace = true;
  await registeredCommands["versionlens.editor.onUpdateDependenciesMinor"]?.();

  expect(applyInputs).toMatchObject([
    { dependencyName: "left-pad\u001f0:30,0:35" },
    { command: "sort" },
    { command: "updateMinor" },
  ]);
  expect(appliedEdits).toHaveLength(bulkCommandCount);
  expect(appliedEdits[0]).toMatchObject({ newText: "1.1.0" });
  expect(testState.refreshCount).toBe(startingRefreshCount + bulkCommandCount);
});

it("open dependency command opens Rust-produced local paths", async (): Promise<void> => {
  const { registerCommands } = await import("../../commands/register.ts");
  openedExternalUris.length = 0;
  shownTextDocuments.length = 0;
  testState.dependencyFileType = 2;
  clearRegisteredCommands();

  registerCommands(commandState(undefined) as never);
  await registeredCommands["versionlens.suggestion.onFileLink"]?.(
    "/repo/local",
  );

  expect(openedExternalUris).toEqual([{ path: "/repo/local", scheme: "file" }]);
  expect(shownTextDocuments).toEqual([]);
});

it("choose build command applies the selected Rust build edit", async (): Promise<void> => {
  const { registerCommands } = await import("../../commands/register.ts");
  const applyInputs: unknown[] = [];
  appliedEdits.length = 0;
  quickPickItems.length = 0;
  quickPickOptions.length = 0;
  clearRegisteredCommands();

  testState.activeTextEditor = { document: buildDocument() };
  registerCommands(commandState(buildSelectionSession(applyInputs)) as never);
  await registeredCommands["versionlens.suggestion.onChooseBuild"]?.(
    "left-pad\u001f0:30,0:43",
    "left-pad",
    "1.0.0+build.1",
    "1.0.0+build.2",
    "1.0.0+build.1",
  );

  expect(quickPickOptions).toContainEqual({
    placeHolder: "Choose a build or press escape to cancel",
    title: "Choose a build for left-pad",
  });
  expect(quickPickItems).toContainEqual({
    label: "1.0.0+build.1",
    picked: true,
  });
  expect(quickPickItems).toContainEqual({
    label: "1.0.0+build.2",
    picked: false,
  });
  expect(applyInputs[0]).toMatchObject({
    dependencyName: "left-pad\u001f0:30,0:43",
    selectedVersion: "1.0.0+build.2",
  });
  expect(appliedEdits[0]).toMatchObject({ newText: "1.0.0+build.2" });
});

it("resolve command confirms vulnerable updates before applying edits", async (): Promise<void> => {
  const { registerCommands } = await import("../../commands/register.ts");
  appliedEdits.length = 0;
  warningMessages.length = 0;
  testState.warningChoice = undefined;
  clearRegisteredCommands();
  const document = {
    getText: (): string => packageFileFixture("package-left-pad.json"),
    languageId: "json",
    uri: { toString: (): string => "file:///package.json" },
  };
  const session = {
    applyCommand: (): {
      vulnerableUpdatePackage: string;
      vulnerableUpdateVersion: string;
      edits: Array<{
        newText: string;
        range: {
          end: { character: number; line: number };
          start: { character: number; line: number };
        };
      }>;
      vulnerableUpdateCount: number;
    } => ({
      vulnerableUpdatePackage: "left-pad",
      vulnerableUpdateVersion: "1.1.0",
      edits: [
        {
          newText: "1.1.0",
          range: {
            end: { character: 35, line: 0 },
            start: { character: 30, line: 0 },
          },
        },
      ],
      vulnerableUpdateCount: 1,
    }),
  };

  testState.activeTextEditor = { document };
  registerCommands(commandState(session) as never);
  await registeredCommands["versionlens.suggestion.onUpdateDependency"]?.(
    "left-pad",
  );

  expect(warningMessages).toHaveLength(1);
  expect(appliedEdits).toEqual([]);

  testState.warningChoice = "Update Anyway";
  await registeredCommands["versionlens.suggestion.onUpdateDependency"]?.(
    "left-pad",
  );

  expect(warningMessages).toHaveLength(2);
  expect(warningMessages[1]).toEqual([
    "Vulnerabilities found in left-pad@1.1.0. Do you want to continue?",
    { modal: true },
    "Update Anyway",
  ]);
  expect(appliedEdits).toHaveLength(1);
  testState.warningChoice = undefined;
});

function packageFileFixture(name: string): string {
  return readFileSync(
    `${process.cwd()}/tests/fixtures/vscode-extension/${name}`,
    "utf8",
  );
}
