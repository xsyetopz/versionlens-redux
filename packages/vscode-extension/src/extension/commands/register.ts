import { commands, type Disposable } from "#vscode-host";
import { removeAuthHeader } from "../auth/remove.ts";
import { addAuthHeader } from "../auth/set.ts";
import { refreshActiveDiagnostics } from "../diagnostics/refresh.ts";
import type { NativeApplyCommand } from "../native/input.ts";
import { recreateSessions } from "../session/registry.ts";
import type { ExtensionState } from "../state.ts";
import { runCustomInstall } from "../tasks/custom-install.ts";
import { applyRustEdits, registerApplyCommands } from "./apply.ts";
import { chooseBuild } from "./build.ts";
import {
  nativeCodeLensArguments,
  refreshCodeLenses,
  registerCodeLensProvider,
} from "./codelens.ts";
import {
  registerConfigurationHandler,
  registerWorkspaceFolderHandler,
} from "./configuration.ts";
import { updateContexts } from "./contexts.ts";
import { showProviderError } from "./error.ts";
import { openDependency } from "./open.ts";
import { togglePrereleases, toggleVersionLenses } from "./toggles.ts";

const nativeBuildArgumentStart = 3;

interface UpdateDependencyRequest {
  codeLensOrName?: unknown;
  command?: NativeApplyCommand | undefined;
  selectedVersion?: string | undefined;
  selector?: string | undefined;
}

function registerCommands(state: ExtensionState): Disposable[] {
  return [
    ...registerSuggestionCommands(state),
    ...registerDisplayCommands(state),
    ...registerApplyCommands(state, [
      ["versionlens.editor.onSortDependencies", "sort"],
      ["versionlens.editor.onUpdateDependenciesLatest", "update"],
      ["versionlens.editor.onUpdateDependenciesMajor", "updateMajor"],
      ["versionlens.editor.onUpdateDependenciesMinor", "updateMinor"],
      ["versionlens.editor.onUpdateDependenciesPatch", "updatePatch"],
    ]),
    ...registerAuthenticationCommands(state),
    registerConfigurationHandler(state),
    registerWorkspaceFolderHandler(state),
    registerCodeLensProvider(state),
  ];
}

function registerSuggestionCommands(state: ExtensionState): Disposable[] {
  return [
    commands.registerCommand("versionlens.suggestion.onClearCache", () =>
      clearCache(state),
    ),
    commands.registerCommand(
      "versionlens.suggestion.onUpdateDependency",
      (
        codeLensOrName?: unknown,
        selector?: string,
        command?: NativeApplyCommand,
        selectedVersion?: string,
      ): Promise<void> =>
        updateDependency(state, {
          codeLensOrName,
          command,
          selectedVersion,
          selector,
        }),
    ),
    commands.registerCommand(
      "versionlens.suggestion.onFileLink",
      (codeLensOrPath?: unknown): Promise<void> => {
        const nativeArguments = nativeCodeLensArguments(codeLensOrPath);
        const path = nativeArguments?.[0] ?? codeLensOrPath;
        return openDependency(stringValue(path));
      },
    ),
    commands.registerCommand(
      "versionlens.suggestion.onChooseBuild",
      (codeLensOrSelector?: unknown, ...builds: string[]): Promise<void> =>
        chooseDependencyBuild(state, codeLensOrSelector, builds),
    ),
  ];
}

function registerDisplayCommands(state: ExtensionState): Disposable[] {
  return [
    commands.registerCommand("versionlens.editor.onShowVersionLenses", () =>
      toggleVersionLenses(state, true),
    ),
    commands.registerCommand("versionlens.editor.onHideVersionLenses", () =>
      toggleVersionLenses(state, false),
    ),
    commands.registerCommand("versionlens.editor.onShowError", () =>
      showProviderError(state),
    ),
    commands.registerCommand(
      "versionlens.icons.showingProgress",
      (): undefined => undefined,
    ),
    commands.registerCommand(
      "versionlens.editor.onShowPrereleaseVersions",
      () => togglePrereleases(state, true),
    ),
    commands.registerCommand(
      "versionlens.editor.onHidePrereleaseVersions",
      () => togglePrereleases(state, false),
    ),
  ];
}

function registerAuthenticationCommands(state: ExtensionState): Disposable[] {
  return [
    commands.registerCommand("versionlens.editor.onAddUrlAuthentication", () =>
      addAuthHeaderAndReload(state),
    ),
    commands.registerCommand(
      "versionlens.editor.onRemoveUrlAuthentication",
      () => removeAuthHeaderAndReload(state),
    ),
    commands.registerCommand("versionlens.editor.onCustomInstall", () =>
      runCustomInstall(state),
    ),
  ];
}

async function clearCache(state: ExtensionState): Promise<void> {
  for (const { session } of state.sessions.values()) {
    session.clearCache();
  }
  state.snapshots.savedDependencies.clear();
  state.snapshots.editedDependencies.clear();
  state.ui.diagnostics?.clear();
  refreshCodeLenses(state);
  await refreshActiveDiagnostics(state);
}

function updateDependency(
  state: ExtensionState,
  request: UpdateDependencyRequest,
): Promise<void> {
  const { codeLensOrName, command, selectedVersion, selector } = request;
  const nativeArguments = nativeCodeLensArguments(codeLensOrName);
  if (nativeArguments) {
    return applyRustEdits(
      state,
      nativeArguments[1] ?? nativeArguments[0],
      nativeArguments[2] as NativeApplyCommand | undefined,
      { selectedVersion: nativeArguments[3] },
    );
  }
  return applyRustEdits(
    state,
    selector ?? stringValue(codeLensOrName),
    command,
    { selectedVersion },
  );
}

function chooseDependencyBuild(
  state: ExtensionState,
  codeLensOrSelector: unknown,
  builds: string[],
): Promise<void> {
  const nativeArguments = nativeCodeLensArguments(codeLensOrSelector);
  if (nativeArguments) {
    return chooseBuild(state, nativeArguments[0], {
      builds: nativeArguments.slice(nativeBuildArgumentStart),
      currentVersion: nativeArguments[2],
      packageName: nativeArguments[1],
    });
  }
  return chooseBuild(state, stringValue(codeLensOrSelector), {
    builds: builds.slice(2),
    currentVersion: builds[1],
    packageName: builds[0],
  });
}

async function addAuthHeaderAndReload(state: ExtensionState): Promise<void> {
  if (await addAuthHeader(state)) {
    await reloadAuthBackedSession(state);
  }
}

async function removeAuthHeaderAndReload(state: ExtensionState): Promise<void> {
  if (await removeAuthHeader(state)) {
    await reloadAuthBackedSession(state);
  }
}

async function reloadAuthBackedSession(state: ExtensionState): Promise<void> {
  for (const { session } of state.sessions.values()) {
    session.clearCache();
  }
  if (state.context?.extensionPath && !(await recreateSessions(state))) {
    return;
  }
  await updateContexts(state);
  refreshCodeLenses(state);
  await refreshActiveDiagnostics(state);
}

function stringValue(value: unknown): string | undefined {
  let string: string | undefined;
  if (typeof value === "string") {
    string = value;
  }
  return string;
}

export { registerCommands };
