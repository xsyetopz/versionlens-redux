import { expect, it } from "../../runtime.ts";
import { packageFileFixture } from "./fixture.ts";
import { commandState, documentStub } from "./state.ts";

import {
  appliedEdits,
  applyTestState,
  registeredCommand,
  reset,
} from "./support.ts";

it("sort command bypasses CodeLens replacement gate like upstream", async (): Promise<void> => {
  const { registerCommands } = await import("../../../commands/register.ts");
  reset();
  const applyInputs: unknown[] = [];
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
            newText: '"a":"1.0.0",\n"b":"1.0.0"',
            range: {
              end: { character: 41, line: 0 },
              start: { character: 17, line: 0 },
            },
          },
        ],
        vulnerableUpdateCount: 0,
      };
    },
  };

  const state = commandState(session, {
    flags: {
      providerBusy: 0,
      providerError: false,
      codeLensReplace: false,
      showPrereleases: false,
      showSuggestionStats: false,
      showVersionLenses: true,
    },
  });
  applyTestState.activeTextEditor = { document: documentStub("b") };
  registerCommands(state as never);
  await registeredCommand("versionlens.editor.onSortDependencies")();

  expect(applyInputs[0]).toMatchObject({ command: "sort" });
  expect(appliedEdits).toEqual([
    expect.objectContaining({ newText: expect.stringContaining('"a"') }),
  ]);
  expect(state.flags.codeLensReplace).toBe(false);
});

it("single update leaves CodeLens replacement disabled after applying like upstream", async (): Promise<void> => {
  const { registerCommands } = await import("../../../commands/register.ts");
  reset();
  const document = documentStub("left-pad");
  const state = commandState({
    applyCommand: (): {
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
    } => ({
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
    }),
  });

  applyTestState.activeTextEditor = { document };
  registerCommands(state as never);
  await registeredCommand("versionlens.suggestion.onUpdateDependency")(
    "left-pad",
  );

  expect(appliedEdits).toHaveLength(1);
  expect(state.flags.codeLensReplace).toBe(false);
});

it("bulk update leaves CodeLens replacement disabled when applyEdit rejects like upstream", async (): Promise<void> => {
  const { registerCommands } = await import("../../../commands/register.ts");
  reset();
  const document = documentStub("left-pad");
  applyTestState.applyEditBlocker = Promise.reject(new Error("apply failed"));
  const state = commandState({
    applyCommand: (): {
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
    } => ({
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
    }),
  });

  applyTestState.activeTextEditor = { document };
  registerCommands(state as never);
  await expect(
    registeredCommand("versionlens.editor.onUpdateDependenciesLatest")(),
  ).rejects.toThrow("apply failed");

  expect(state.flags.codeLensReplace).toBe(false);
});

it("workspace applyEdit false is reported as a failed edit", async (): Promise<void> => {
  const { registerCommands } = await import("../../../commands/register.ts");
  reset();
  const document = documentStub("left-pad");
  applyTestState.applyEditBlocker = Promise.resolve(false);
  const state = commandState({
    applyCommand: (): {
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
    } => ({
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
    }),
  });

  applyTestState.activeTextEditor = { document };
  registerCommands(state as never);
  await expect(
    registeredCommand("versionlens.editor.onUpdateDependenciesLatest")(),
  ).rejects.toThrow("could not apply");
  expect(state.flags.codeLensReplace).toBe(false);
});

it("vulnerability confirmation rejects edits after the document changes", async (): Promise<void> => {
  const { registerCommands } = await import("../../../commands/register.ts");
  reset();
  let text = packageFileFixture("package-left-pad-template.json").replace(
    "__PACKAGE__",
    "left-pad",
  );
  let version = 1;
  const document = {
    getText: (): string => text,
    get version(): number {
      return version;
    },
    languageId: "json",
    uri: { toString: (): string => "file:///package.json" },
  };
  let confirm: ((choice: string) => void) | undefined;
  applyTestState.warningChoice = new Promise((resolve): void => {
    confirm = resolve;
  });
  applyTestState.activeTextEditor = { document };
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
              end: { character: 35, line: 0 },
              start: { character: 30, line: 0 },
            },
          },
        ],
        vulnerableUpdateCount: 1,
      }),
    }) as never,
  );
  const pending = registeredCommand(
    "versionlens.suggestion.onUpdateDependency",
  )("left-pad");
  await Promise.resolve();
  text = text.replace("1.0.0", "1.0.1");
  version += 1;
  confirm?.("Update Anyway");
  await pending;

  expect(appliedEdits).toEqual([]);
});
