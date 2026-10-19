import { existsSync } from "node:fs";
import type {
  Executable,
  LanguageClientOptions,
} from "vscode-languageclient/node";
import {
  ExecuteCommandRequest,
  LanguageClient,
  TransportKind,
} from "vscode-languageclient/node";
import { window } from "#vscode-host";
import {
  type AuthorizationParams,
  answerAuthorization,
} from "./auth/authorization.ts";
import { reportError } from "./commands/error.ts";
import { withToggles } from "./config/toggles.ts";
import { documentSelectors } from "./documents.ts";
import { clearManifestInfo } from "./manifest.ts";
import { resolveServerPath, SERVER_ENVIRONMENT_VARIABLE } from "./server.ts";
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
  const clientRef: { current: LanguageClient | undefined } = {
    current: undefined,
  };
  const clientOptions: LanguageClientOptions = {
    documentSelector: documentSelectors(),
    middleware: {
      // Choosing "hide" hides the diagnostics too, as in 0.4.4. The last
      // ones are kept so showing them again needs no new server work.
      handleDiagnostics: (uri, diagnostics, next) => {
        const key = uri.toString();
        if (diagnostics.length > 0) {
          state.diagnostics.set(key, { diagnostics, uri });
        } else {
          state.diagnostics.delete(key);
        }
        next(uri, state.flags.diagnosticsHidden ? [] : diagnostics);
      },
      provideCodeLenses: (document, token, next) => {
        if (!state.flags.showVersionLenses) {
          return [];
        }
        return next(document, token);
      },
      // The default handler turns a failure into a generic notification, so
      // send the request here and show the server's message once.
      executeCommand: async (command, args) => {
        try {
          return await clientRef.current?.sendRequest(
            ExecuteCommandRequest.type,
            { arguments: args, command },
          );
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
  clientRef.current = client;
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

/**
 * Start, stop and restart run one after another on one chain. A client that
 * is still starting cannot be shut down, so a stop waits for its start.
 */
function enqueue(
  state: ExtensionState,
  step: () => Promise<void>,
): Promise<void> {
  const run = state.lifecycle.then(step);
  state.lifecycle = run.catch((): void => undefined);
  return run;
}

function startClient(state: ExtensionState): Promise<void> {
  return enqueue(state, (): Promise<void> => start(state));
}

function stopClient(state: ExtensionState): Promise<void> {
  return enqueue(state, (): Promise<void> => stop(state));
}

function restartClient(state: ExtensionState): Promise<void> {
  return enqueue(state, async (): Promise<void> => {
    try {
      await stop(state);
    } finally {
      await start(state);
    }
  });
}

/**
 * Starts the server. A server that cannot start is reported, and the
 * commands that do not need it keep working.
 */
async function start(state: ExtensionState): Promise<void> {
  const extensionPath = state.context?.extensionPath;
  if (!extensionPath) {
    return;
  }
  const serverPath = resolveServerPath(extensionPath);
  if (!existsSync(serverPath)) {
    const message = `VersionLens Redux could not find its language server (${serverPath}). Install the extension for your platform, or set ${SERVER_ENVIRONMENT_VARIABLE} to a versionlens-lsp binary.`;
    state.ui.outputChannel?.error(message);
    // Not awaited: the promise settles only when the user closes the message.
    void window.showErrorMessage(message);
    return;
  }
  state.selectorKey = JSON.stringify(documentSelectors());
  const client = createLanguageClient(state, serverPath);
  state.client = client;
  try {
    await client.start();
    state.ui.outputChannel?.info(`language server started (${serverPath})`);
  } catch (error) {
    // The client already told the user that it could not connect.
    state.client = undefined;
    await disposeClient(state, client);
    state.ui.outputChannel?.error(
      `could not start the language server (${serverPath}): ${String(error)}`,
    );
  }
}

async function stop(state: ExtensionState): Promise<void> {
  const { client } = state;
  state.client = undefined;
  clearManifestInfo();
  state.diagnostics.clear();
  state.promptedHosts.clear();
  await resetProviderState(state);
  if (client) {
    await disposeClient(state, client);
  }
}

async function disposeClient(
  state: ExtensionState,
  client: LanguageClient,
): Promise<void> {
  try {
    await client.dispose();
  } catch (error) {
    state.ui.outputChannel?.error(
      `could not stop the server: ${String(error)}`,
    );
  }
}

/** Shows or hides the diagnostics the server published. */
function applyDiagnosticsVisibility(state: ExtensionState): void {
  const collection = state.client?.diagnostics;
  for (const { diagnostics, uri } of state.diagnostics.values()) {
    collection?.set(uri, state.flags.diagnosticsHidden ? [] : diagnostics);
  }
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
  applyDiagnosticsVisibility,
  createLanguageClient,
  refreshCodeLenses,
  restartClient,
  startClient,
  stopClient,
};
