import {
  type ExtensionContext,
  type TextDocumentChangeEvent,
  TextDocumentChangeReason,
  window,
  workspace,
} from "#vscode-host";
import { updateContexts } from "../commands/contexts.ts";
import { registerCommands } from "../commands/register.ts";
import { analyzeDocument } from "../diagnostics/analyze.ts";
import { refreshDiagnostics } from "../diagnostics/refresh.ts";
import { fileDocument } from "../documents/file.ts";
import type { ExtensionState } from "../state.ts";
import { handleDidSaveTextDocument } from "../tasks.ts";
import {
  registerPackageFileWatchers,
  watchActivePackageFileOutsideWorkspace,
} from "./package-watchers.ts";
import { registerRefreshTimer } from "./refresh-timer.ts";

function registerExtensionSubscriptions(
  state: ExtensionState,
  context: ExtensionContext,
): void {
  const { diagnostics, outputChannel } = state.ui;
  if (!(diagnostics && outputChannel)) {
    return;
  }

  const packageFileWatchers = registerPackageFileWatchers(state);

  context.subscriptions.push(
    diagnostics,
    outputChannel,
    ...registerCommands(state),
    ...packageFileWatchers,
    registerRefreshTimer(state),
    workspace.onDidChangeTextDocument(async (event): Promise<void> => {
      if (!isRelevantTextDocumentChange(event)) {
        return;
      }
      const output = analyzeDocument(state, event.document);
      if (!output?.isSupportedManifest) {
        return;
      }

      await refreshDiagnostics(state, event.document);
      if (event.document === window.activeTextEditor?.document) {
        await updateContexts(state);
      }
    }),
    workspace.onDidSaveTextDocument(
      (document): Promise<void> => handleDidSaveTextDocument(state, document),
    ),
    workspace.onDidCloseTextDocument((closedDocument): void => {
      const document = fileDocument(closedDocument);
      if (!document) {
        return;
      }

      const output = analyzeDocument(state, document);
      if (!output?.isSupportedManifest) {
        return;
      }

      state.snapshots.editedDependencies.delete(document.uri.toString());
    }),
    window.onDidChangeActiveTextEditor(async (editor): Promise<void> => {
      const providerActive = await updateContexts(state);
      const document = fileDocument(editor?.document);
      if (document && providerActive) {
        watchActivePackageFileOutsideWorkspace(state, document);
      }
    }),
  );
}

function isRelevantTextDocumentChange(event: TextDocumentChangeEvent): boolean {
  return (
    event.reason === TextDocumentChangeReason.Undo ||
    event.reason === TextDocumentChangeReason.Redo ||
    event.contentChanges.length > 0
  );
}

export { registerExtensionSubscriptions };
