import { expect, it } from "../runtime.ts";

import { commandState } from "./command-state.ts";
import {
  clearRegisteredCommands,
  createdSessionConfigs,
  executedTasks,
  quickPickItems,
  registeredCommands,
  smokeTaskLabel,
  testState,
} from "./support.ts";

interface TimerHandle {
  [Symbol.dispose]: () => undefined;
  [Symbol.toPrimitive]: () => number;
  hasRef: () => boolean;
  ref: () => TimerHandle;
  refresh: () => TimerHandle;
  unref: () => TimerHandle;
}

const toggleRefreshCount = 3;
const refreshIntervalMs = 123;
const saveRefreshCount = 3;

it("choose build returns before QuickPick when CodeLens replacement is disabled", async (): Promise<void> => {
  const { chooseBuild } = await import("../../commands/build.ts");
  quickPickItems.length = 0;

  await chooseBuild(
    commandState(undefined, {
      flags: { codeLensReplace: false },
    }) as never,
    "left-pad\u001f0:30,0:43",
    {
      builds: ["1.0.0+build.1", "1.0.0+build.2"],
      currentVersion: "1.0.0+build.1",
      packageName: "left-pad",
    },
  );

  expect(quickPickItems).toHaveLength(0);
});

it("version lens toggles refresh registered code lenses", async (): Promise<void> => {
  const { registerCommands } = await import("../../commands/register.ts");
  clearRegisteredCommands();
  testState.codeLensRefreshCount = 0;
  testState.activeRefreshCount = 0;
  createdSessionConfigs.length = 0;
  let diagnosticsClearCount = 0;
  testState.activeTextEditor = {
    document: {
      uri: { scheme: "file", toString: (): string => "file:///package.json" },
    },
  };
  testState.analyzed = {
    activeProviderName: "npm",
    canSortDependencies: true,
    isSupportedManifest: true,
  };
  const state = commandState(
    { disposeSession: (): undefined => undefined },
    {
      context: {
        extensionPath: "/extension",
        secrets: { get: (): undefined => undefined },
      },
      ui: {
        codeLensRefresh: undefined,
        diagnostics: {
          clear(): void {
            diagnosticsClearCount += 1;
          },
        },
        outputChannel: undefined,
      },
    },
  );

  registerCommands(state as never);
  await registeredCommands["versionlens.editor.onHideVersionLenses"]?.();
  await registeredCommands["versionlens.editor.onShowVersionLenses"]?.();
  await registeredCommands["versionlens.editor.onShowPrereleaseVersions"]?.();

  expect(testState.codeLensRefreshCount).toBe(toggleRefreshCount);
  expect(diagnosticsClearCount).toBe(1);
  expect(testState.activeRefreshCount).toBe(1);
  expect(state.flags.showVersionLenses).toBe(true);
  expect(state.flags.showPrereleases).toBe(true);
  expect(createdSessionConfigs).toHaveLength(2);
});

it("refresh timer refreshes active diagnostics on schedule", async (): Promise<void> => {
  const { registerRefreshTimer } = await import(
    "../../lifecycle/refresh-timer.ts"
  );
  const originalSetInterval = globalThis.setInterval;
  const originalClearInterval = globalThis.clearInterval;
  let scheduled:
    | {
        callback: () => void;
        delay: number | undefined;
        timer: { unref: () => void };
      }
    | undefined;
  let cleared: unknown;
  testState.activeRefreshCount = 0;

  globalThis.setInterval = ((
    callback: () => void,
    delay?: number,
  ): TimerHandle => {
    using timer = {
      [Symbol.dispose]: (): undefined => undefined,
      [Symbol.toPrimitive]: (): number => 0,
      hasRef: (): boolean => false,
      ref: (): TimerHandle => timer,
      refresh: (): TimerHandle => timer,
      unref: (): TimerHandle => timer,
    };
    scheduled = { callback, delay, timer };
    return timer;
  }) as unknown as typeof setInterval;
  globalThis.clearInterval = ((timer: unknown): void => {
    cleared = timer;
  }) as typeof clearInterval;

  try {
    const disposable = registerRefreshTimer({} as never, refreshIntervalMs);
    expect(scheduled?.delay).toBe(refreshIntervalMs);

    scheduled?.callback();

    expect(testState.activeRefreshCount).toBe(1);
    disposable.dispose();
    expect(cleared).toBe(scheduled?.timer);
  } finally {
    globalThis.setInterval = originalSetInterval;
    globalThis.clearInterval = originalClearInterval;
  }
});

it("custom install command only runs for file-backed active editors", async (): Promise<void> => {
  const { registerCommands } = await import("../../commands/register.ts");
  executedTasks.length = 0;
  testState.taskCompletionMode = "auto";
  clearRegisteredCommands();
  testState.analyzed = {
    activeProviderName: "npm",
    canSortDependencies: true,
    installTaskConfigKey: "npm.onSaveChanges",
    isSupportedManifest: true,
  };

  testState.activeTextEditor = {
    document: {
      uri: {
        scheme: "versionlens",
        toString: (): string =>
          "versionlens:/versionlens.multi-registries.json",
      },
    },
  };
  registerCommands(commandState({}) as never);
  await registeredCommands["versionlens.editor.onCustomInstall"]?.();
  expect(executedTasks).toEqual([]);

  testState.activeTextEditor = {
    document: {
      uri: { scheme: "file", toString: (): string => "file:///package.json" },
    },
  };
  await registeredCommands["versionlens.editor.onCustomInstall"]?.();
  expect(executedTasks).toEqual([smokeTaskLabel]);
});

it("save task ignores unsupported documents without creating snapshots", async (): Promise<void> => {
  const { handleDidSaveTextDocument } = await import("../../tasks.ts");
  executedTasks.length = 0;
  testState.analyzed = {
    canSortDependencies: false,
    isSupportedManifest: false,
  };
  testState.dependencySnapshotValue = "";
  const state = {
    flags: { showOutdated: true },
    snapshots: {
      editedDependencies: new Map<string, string>(),
      savedDependencies: new Map<string, string>(),
    },
  };
  const document = {
    uri: { scheme: "file", toString: (): string => "file:///README.md" },
  };

  await handleDidSaveTextDocument(state as never, document as never);

  expect(executedTasks).toEqual([]);
  expect(state.snapshots.savedDependencies.has("file:///README.md")).toBe(
    false,
  );
  expect(state.snapshots.editedDependencies.has("file:///README.md")).toBe(
    false,
  );
  expect(state.flags.showOutdated).toBe(true);
});

it("save task runs only after dependency signature changes", async (): Promise<void> => {
  const { handleDidSaveTextDocument } = await import("../../tasks.ts");
  executedTasks.length = 0;
  const startingRefreshCount = testState.refreshCount;
  testState.taskCompletionMode = "auto";
  const state = {
    flags: { showOutdated: false },
    snapshots: {
      editedDependencies: new Map<string, string>(),
      savedDependencies: new Map<string, string>(),
    },
  };
  const document = {
    uri: { scheme: "file", toString: (): string => "file:///package.json" },
  };

  testState.analyzed = {
    activeProviderName: "npm",
    canSortDependencies: true,
    installTaskConfigKey: "npm.onSaveChanges",
    isSupportedManifest: true,
  };
  testState.dependencySnapshotValue = "left-pad@1";

  await handleDidSaveTextDocument(state as never, document as never);
  expect(executedTasks).toEqual([]);

  state.snapshots.editedDependencies.set("file:///package.json", "left-pad@2");
  await handleDidSaveTextDocument(state as never, document as never);
  expect(executedTasks).toEqual([smokeTaskLabel]);

  testState.dependencySnapshotValue = "left-pad@2";
  await handleDidSaveTextDocument(state as never, document as never);
  expect(executedTasks).toEqual([smokeTaskLabel]);
  expect(testState.refreshCount).toBe(startingRefreshCount + saveRefreshCount);
});
