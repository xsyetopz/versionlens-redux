import type {
  Executable,
  LanguageClientOptions,
} from "vscode-languageclient/node";
import { LanguageClient, TransportKind } from "vscode-languageclient/node";
import { window } from "#vscode-host";
import {
  type AuthorizationParams,
  answerAuthorization,
} from "./auth/authorization.ts";
import { reportError } from "./commands/error.ts";
import { documentSelectors } from "./documents.ts";
import { clearManifestInfo } from "./manifest.ts";
import { resolveServerPath } from "./server.ts";
import {
  applyStatus,
  resetProviderState,
  type StatusParams,
} from "./session/status.ts";
import type { ExtensionState } from "./state.ts";

const STATUS_METHOD = "versionlens/status";
const AUTHORIZATION_METHOD = "versionlens/authorization";

/**
 * Client capabilities come from `vscode-languageclient`: `applyEdit`,
 * `workspaceEdit.documentChanges`, `codeLens.refreshSupport` and
 * `workspace.configuration` are all advertised by default.
 */
function createLanguageClient(
  state: ExtensionState,
  serverPath: string,
): LanguageClient {
  const serverOptions: Executable = {
    command: serverPath,
    transport: TransportKind.stdio,
  };
  const clientOptions: LanguageClientOptions = {
    documentSelector: documentSelectors(),
    middleware: {
      provideCodeLenses: (document, token, next) => {
        if (!state.flags.showVersionLenses) {
          return [];
        }
        return next(document, token);
      },
      // The command fails with a message from the server, so show it.
      executeCommand: async (command, args, next) => {
        try {
          return await next(command, args);
        } catch (error) {
          await reportError(state, `${command} failed`, error);
          return;
        }
      },
      workspace: {
        configuration: async (params, token, next) => {
          const settings = await next(params, token);
          if (!Array.isArray(settings)) {
            return settings;
          }
          return settings.map((value, index): unknown => {
            if (params.items[index]?.section === "versionlens") {
              return withToggles(state, value);
            }
            return value;
          });
        },
      },
    },
  };
  if (state.ui.outputChannel) {
    clientOptions.outputChannel = state.ui.outputChannel;
  }

  const client = new LanguageClient(
    "versionlens",
    "VersionLens Redux",
    serverOptions,
    clientOptions,
  );
  client.onNotification(STATUS_METHOD, (params: StatusParams): void => {
    applyStatus(state, params).catch((error: unknown): void => {
      state.ui.outputChannel?.error(String(error));
    });
  });
  client.onRequest(AUTHORIZATION_METHOD, (params: AuthorizationParams) =>
    answerAuthorization(state, params),
  );
  return client;
}

/** The toggles of the editor win over the settings file. */
function withToggles(state: ExtensionState, value: unknown): unknown {
  const settings = JSON.parse(JSON.stringify(value ?? {})) as {
    suggestions?: Record<string, unknown>;
  };
  settings.suggestions = {
    ...settings.suggestions,
    showPrereleasesOnStartup: state.flags.showPrereleases,
  };
  return settings;
}

async function startClient(state: ExtensionState): Promise<void> {
  const extensionPath = state.context?.extensionPath;
  if (!extensionPath) {
    throw new Error("The extension is not activated.");
  }
  const serverPath = resolveServerPath(extensionPath);
  state.selectorKey = JSON.stringify(documentSelectors());
  const client = createLanguageClient(state, serverPath);
  state.client = client;
  try {
    await client.start();
  } catch (error) {
    state.client = undefined;
    const message = `VersionLens Redux could not start its language server (${serverPath}).`;
    state.ui.outputChannel?.error(`${message} ${String(error)}`);
    await window.showErrorMessage(message);
    throw error;
  }
}

async function stopClient(state: ExtensionState): Promise<void> {
  const { client } = state;
  state.client = undefined;
  clearManifestInfo();
  state.promptedHosts.clear();
  await resetProviderState(state);
  if (client) {
    await client.dispose();
  }
}

async function restartClient(state: ExtensionState): Promise<void> {
  await stopClient(state);
  await startClient(state);
}

/** Asks the editor to ask the server for lenses again. */
function refreshCodeLenses(state: ExtensionState): void {
  const feature = state.client?.getFeature("textDocument/codeLens");
  for (const editor of window.visibleTextEditors) {
    const provider = feature?.getProvider(editor.document);
    if (provider) {
      provider.onDidChangeCodeLensEmitter.fire();
      return;
    }
  }
}

export {
  createLanguageClient,
  refreshCodeLenses,
  restartClient,
  startClient,
  stopClient,
};
