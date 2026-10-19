import { commands, type Disposable } from "#vscode-host";
import { removeAuthHeader } from "../auth/remove.ts";
import { addAuthHeader } from "../auth/set.ts";
import { restartClient } from "../client.ts";
import type { ExtensionState } from "../state.ts";
import { runCustomInstall } from "../tasks/custom-install.ts";
import { updateContexts } from "./contexts.ts";
import { reportError, showProviderError } from "./error.ts";
import {
  runDocumentCommand,
  runServerCommand,
  SERVER_COMMANDS,
  type UpdateLevel,
} from "./server.ts";
import { togglePrereleases, toggleVersionLenses } from "./toggles.ts";

type Handler = () => Promise<void> | undefined;

/**
 * Registers a command that shows its failures to the user. Commands the
 * server implements are forwarded with `workspace/executeCommand`. The
 * others (toggles, authentication, tasks) run in the extension.
 */
function registerCommand(
  state: ExtensionState,
  id: string,
  handler: Handler,
): Disposable {
  return commands.registerCommand(id, async (): Promise<void> => {
    try {
      await handler();
    } catch (error) {
      await reportError(state, `${id} failed`, error);
    }
  });
}

function registerCommands(state: ExtensionState): Disposable[] {
  const register = (id: string, handler: Handler): Disposable =>
    registerCommand(state, id, handler);
  const updateAll = (level: UpdateLevel) => (): Promise<unknown> =>
    runDocumentCommand(state, SERVER_COMMANDS.updateAll, { level });

  return [
    register("versionlens.editor.onShowVersionLenses", () =>
      toggleVersionLenses(state, true),
    ),
    register("versionlens.editor.onHideVersionLenses", () =>
      toggleVersionLenses(state, false),
    ),
    register("versionlens.editor.onShowError", () => showProviderError(state)),
    register("versionlens.icons.showingProgress", () => undefined),
    register("versionlens.editor.onShowPrereleaseVersions", () =>
      togglePrereleases(state, true),
    ),
    register("versionlens.editor.onHidePrereleaseVersions", () =>
      togglePrereleases(state, false),
    ),
    register("versionlens.suggestion.onClearCache", async () => {
      state.snapshots.savedDependencies.clear();
      state.snapshots.editedDependencies.clear();
      await runServerCommand(state, SERVER_COMMANDS.clearCache);
    }),
    register("versionlens.editor.onSortDependencies", async () => {
      await runDocumentCommand(state, SERVER_COMMANDS.sort);
    }),
    register("versionlens.editor.onUpdateDependenciesLatest", async () => {
      await updateAll("latest")();
    }),
    register("versionlens.editor.onUpdateDependenciesMajor", async () => {
      await updateAll("major")();
    }),
    register("versionlens.editor.onUpdateDependenciesMinor", async () => {
      await updateAll("minor")();
    }),
    register("versionlens.editor.onUpdateDependenciesPatch", async () => {
      await updateAll("patch")();
    }),
    register("versionlens.editor.onAddUrlAuthentication", async () => {
      if (await addAuthHeader(state)) {
        await reloadCredentials(state);
      }
    }),
    register("versionlens.editor.onRemoveUrlAuthentication", async () => {
      if (await removeAuthHeader(state)) {
        await reloadCredentials(state);
      }
    }),
    register("versionlens.editor.onCustomInstall", () =>
      runCustomInstall(state),
    ),
  ];
}

/**
 * The server asks for a credential once per host and session, so a new
 * session is how it learns about a changed one.
 */
async function reloadCredentials(state: ExtensionState): Promise<void> {
  await restartClient(state);
  await updateContexts(state);
}

export { registerCommands };
