import { expect, it } from "../../runtime.ts";
import { packageFileFixture } from "./fixture.ts";
import { commandState, documentStub } from "./state.ts";

import {
  appliedEdits,
  applyResult,
  applyTestState,
  registeredCommand,
  reset,
} from "./support.ts";

const sortStartCharacter = 17;
const sortEndCharacter = 41;
const updateStartCharacter = 30;
const updateEndCharacter = 35;

it("sort command bypasses CodeLens replacement gate like upstream", async (): Promise<void> => {
  const { registerCommands } = await import("../../../commands/register.ts");
  reset();
  const applyInputs: unknown[] = [];
  const session = {
    applyCommand(input: unknown): ReturnType<typeof applyResult> {
      applyInputs.push(input);
      return applyResult(
        '"a":"1.0.0",\n"b":"1.0.0"',
        sortStartCharacter,
        sortEndCharacter,
      );
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
    applyCommand: (): ReturnType<typeof applyResult> => applyResult(),
  });

  applyTestState.activeTextEditor = { document };
  registerCommands(state as never);
  await registeredCommand("versionlens.suggestion.onUpdateDependency")(
    "left-pad",
  );

  expect(appliedEdits).toHaveLength(1);
  expect(state.flags.codeLensReplace).toBe(false);
});

for (const testCase of [
  {
    name: "bulk update leaves CodeLens replacement disabled when applyEdit rejects like upstream",
    blocker: (): Promise<never> => Promise.reject(new Error("apply failed")),
    expectedError: "apply failed",
  },
  {
    name: "workspace applyEdit false is reported as a failed edit",
    blocker: (): Promise<boolean> => Promise.resolve(false),
    expectedError: "could not apply",
  },
] as const) {
  it(testCase.name, async (): Promise<void> => {
    const { registerCommands } = await import("../../../commands/register.ts");
    reset();
    applyTestState.applyEditBlocker = testCase.blocker();
    const state = commandState({
      applyCommand: (): ReturnType<typeof applyResult> => applyResult(),
    });

    applyTestState.activeTextEditor = {
      document: documentStub("left-pad"),
    };
    registerCommands(state as never);
    await expect(
      registeredCommand("versionlens.editor.onUpdateDependenciesLatest")(),
    ).rejects.toThrow(testCase.expectedError);
    expect(state.flags.codeLensReplace).toBe(false);
  });
}

it("vulnerability confirmation rejects edits after the document changes", async (): Promise<void> => {
  const { registerCommands } = await import("../../../commands/register.ts");
  reset();
  let text = packageFileFixture("left-pad-template.json").replace(
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
      applyCommand: (): ReturnType<typeof applyResult> =>
        applyResult("1.1.0", updateStartCharacter, updateEndCharacter, 1),
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
