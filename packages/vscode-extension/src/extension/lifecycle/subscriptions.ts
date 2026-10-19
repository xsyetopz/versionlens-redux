import {
  type ExtensionContext,
  type TextDocument,
  type TextDocumentChangeEvent,
  TextDocumentChangeReason,
  window,
  workspace,
} from "#vscode-host";
import {
  registerConfigurationHandler,
  registerWorkspaceFolderHandler,
} from "../commands/configuration.ts";
import { updateContexts } from "../commands/contexts.ts";
import { reportError } from "../commands/error.ts";
import { registerCommands } from "../commands/register.ts";
import { fileDocument } from "../documents.ts";
import { manifestInfo } from "../manifest.ts";
import type { ExtensionState } from "../state.ts";
import {
  dependencySnapshot,
  rememberDependencySnapshot,
} from "../tasks/snapshot.ts";
import { handleDidSaveTextDocument } from "../tasks.ts";

function registerExtensionSubscriptions(
  state: ExtensionState,
  context: ExtensionContext,
): void {
  const { outputChannel } = state.ui;
  if (!outputChannel) {
    return;
  }

  context.subscriptions.push(
    outputChannel,
    ...registerCommands(state),
    registerConfigurationHandler(state),
    registerWorkspaceFolderHandler(state),
    workspace.onDidOpenTextDocument(
      (document): Promise<void> =>
        guarded(state, "could not read the document", () =>
          trackDocument(state, document),
        ),
    ),
    workspace.onDidChangeTextDocument(
      (event): Promise<void> =>
        guarded(state, "could not read the document", async () => {
          if (!isRelevantTextDocumentChange(event)) {
            return;
          }
          await trackDocument(state, event.document);
          if (event.document === window.activeTextEditor?.document) {
            await updateContexts(state);
          }
        }),
    ),
    workspace.onDidSaveTextDocument(
      (document): Promise<void> =>
        guarded(state, "could not run the install task", async () => {
          await handleDidSaveTextDocument(state, document);
          await updateContexts(state);
        }),
    ),
    workspace.onDidCloseTextDocument((closedDocument): void => {
      const document = fileDocument(closedDocument);
      if (document) {
        state.snapshots.editedDependencies.delete(document.uri.toString());
      }
    }),
    window.onDidChangeActiveTextEditor(
      (): Promise<void> =>
        guarded(state, "could not update the editor state", async () => {
          await updateContexts(state);
        }),
    ),
  );
}

/** Remembers the dependencies of a manifest to tell later edits apart. */
async function trackDocument(
  state: ExtensionState,
  textDocument: TextDocument,
): Promise<void> {
  const document = fileDocument(textDocument);
  if (!(document && (await manifestInfo(state, document))?.supported)) {
    return;
  }
  rememberDependencySnapshot(
    state,
    document,
    await dependencySnapshot(state, document),
  );
}

async function guarded(
  state: ExtensionState,
  title: string,
  handler: () => Promise<void>,
): Promise<void> {
  try {
    await handler();
  } catch (error) {
    await reportError(state, title, error);
  }
}

function isRelevantTextDocumentChange(event: TextDocumentChangeEvent): boolean {
  return (
    event.reason === TextDocumentChangeReason.Undo ||
    event.reason === TextDocumentChangeReason.Redo ||
    event.contentChanges.length > 0
  );
}

export { registerExtensionSubscriptions, trackDocument };
