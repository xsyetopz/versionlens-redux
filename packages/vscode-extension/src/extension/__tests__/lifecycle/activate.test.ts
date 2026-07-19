import nodeProcess from "node:process";
import { expect, it, mock, mockVscodeHost } from "../runtime.ts";

const outputLines: string[] = [];
let editorCodeLens = true;
let updateContextCount = 0;
let recreateSessionCount = 0;
let refreshDiagnosticsCount = 0;
let packageWatchingCount = 0;
let subscriptionCount = 0;
let uiInitCount = 0;
let recreateSessionError: Error | undefined;
let originalVersionLensInstalled = false;
let warningSelection: string | undefined;
const warningMessages: string[] = [];
const errorMessages: string[] = [];
const executedCommands: unknown[][] = [];

mockVscodeHost(() => ({
  commands: {
    executeCommand(...args: unknown[]): void {
      executedCommands.push(args);
    },
  },
  extensions: {
    getExtension(id: string): { id: string } | undefined {
      let extension: { id: string } | undefined;
      if (
        originalVersionLensInstalled &&
        id === "pflannery.vscode-versionlens"
      ) {
        extension = { id };
      }
      return extension;
    },
  },
  window: {
    showErrorMessage(message: string): void {
      errorMessages.push(message);
    },
    showWarningMessage(message: string): string | undefined {
      warningMessages.push(message);
      return warningSelection;
    },
  },
  workspace: {
    getConfiguration(): { get: (key: string, fallback?: unknown) => unknown } {
      return {
        get(key: string, fallback?: unknown): unknown {
          if (key === "editor.codeLens") {
            return editorCodeLens;
          }
          return fallback;
        },
      };
    },
  },
}));

mock.module("../../commands/contexts.ts", () => ({
  updateContexts: (): void => {
    updateContextCount += 1;
  },
}));

mock.module("../../diagnostics/refresh.ts", () => ({
  refreshActiveDiagnostics: (): void => {
    refreshDiagnosticsCount += 1;
  },
}));

mock.module("../../session/registry.ts", () => ({
  recreateSessions: (): void => {
    recreateSessionCount += 1;
    if (recreateSessionError) {
      throw recreateSessionError;
    }
  },
}));

mock.module("../../session/flags.ts", () => ({
  reloadConfigurationState: () => undefined,
}));

mock.module("../../lifecycle/package-watchers.ts", () => ({
  initializePackageFileWatching: (): void => {
    packageWatchingCount += 1;
  },
}));

mock.module("../../lifecycle/subscriptions.ts", () => ({
  registerExtensionSubscriptions: (): void => {
    subscriptionCount += 1;
  },
}));

mock.module("../../lifecycle/ui.ts", () => ({
  initializeUi: (state: {
    ui: { outputChannel?: { appendLine: (value: string) => void } };
  }): void => {
    uiInitCount += 1;
    state.ui.outputChannel = {
      appendLine(value: string): void {
        outputLines.push(value);
      },
    };
  },
}));

function reset(): void {
  outputLines.length = 0;
  editorCodeLens = true;
  updateContextCount = 0;
  recreateSessionCount = 0;
  refreshDiagnosticsCount = 0;
  packageWatchingCount = 0;
  subscriptionCount = 0;
  uiInitCount = 0;
  recreateSessionError = undefined;
  originalVersionLensInstalled = false;
  warningSelection = undefined;
  warningMessages.length = 0;
  errorMessages.length = 0;
  executedCommands.length = 0;
}

it("activation warns when VS Code editor code lenses are disabled", async (): Promise<void> => {
  reset();
  editorCodeLens = false;
  const { activateExtension } = await import("../../lifecycle/activate.ts");
  const state = {
    context: undefined,
    ui: {},
    flags: { showVersionLenses: true },
  };

  await activateExtension(state as never, { subscriptions: [] } as never);

  expect(outputLines).toContain(
    "Code lenses are disabled. This extension won't work unless you enable 'editor.codeLens' in your vscode settings",
  );
  expect(uiInitCount).toBe(1);
  expect(recreateSessionCount).toBe(1);
  expect(updateContextCount).toBe(1);
  expect(packageWatchingCount).toBe(1);
  expect(subscriptionCount).toBe(1);
  expect(refreshDiagnosticsCount).toBe(1);
});

it("activation registers commands before native session creation can fail", async (): Promise<void> => {
  reset();
  recreateSessionError = new Error("native session failed");
  const { activateExtension } = await import("../../lifecycle/activate.ts");
  const state = {
    context: undefined,
    ui: {},
    flags: { showVersionLenses: true },
  };

  await expect(
    activateExtension(state as never, { subscriptions: [] } as never),
  ).rejects.toThrow("native session failed");
  const runtime = [nodeProcess.platform, nodeProcess.arch].join("-");
  expect(errorMessages).toEqual([
    `VersionLens Redux could not load its native runtime for ${runtime}. Install the matching platform package.`,
  ]);
  expect(outputLines.at(-1)).toContain("native session failed");

  expect(uiInitCount).toBe(1);
  expect(subscriptionCount).toBe(1);
  expect(recreateSessionCount).toBe(1);
  expect(updateContextCount).toBe(0);
  expect(packageWatchingCount).toBe(0);
  expect(refreshDiagnosticsCount).toBe(0);
});

it("activation does not warn when VS Code editor code lenses are enabled", async (): Promise<void> => {
  reset();
  const { activateExtension } = await import("../../lifecycle/activate.ts");
  const state = {
    context: undefined,
    ui: {},
    flags: { showVersionLenses: true },
  };

  await activateExtension(state as never, { subscriptions: [] } as never);

  expect(outputLines).toEqual([]);
});

it("activation warns when original VersionLens is installed", async (): Promise<void> => {
  reset();
  originalVersionLensInstalled = true;
  const { activateExtension } = await import("../../lifecycle/activate.ts");
  const state = {
    context: undefined,
    ui: {},
    flags: { showVersionLenses: true },
  };

  await activateExtension(state as never, { subscriptions: [] } as never);

  const message =
    "VersionLens Redux conflicts with the original VersionLens extension. Disable the original extension before using VersionLens Redux in this workspace.";
  expect(outputLines).toContain(message);
  expect(warningMessages).toEqual([message]);
  expect(executedCommands).toEqual([]);
});

it("activation can disable original VersionLens from the conflict prompt", async (): Promise<void> => {
  reset();
  originalVersionLensInstalled = true;
  warningSelection = "Disable original VersionLens";
  const { activateExtension } = await import("../../lifecycle/activate.ts");
  const state = {
    context: undefined,
    ui: {},
    flags: { showVersionLenses: true },
  };

  await activateExtension(state as never, { subscriptions: [] } as never);

  expect(executedCommands).toEqual([
    [
      "workbench.extensions.action.disableExtension",
      "pflannery.vscode-versionlens",
    ],
  ]);
});
