import {
  type Disposable,
  type TextDocument,
  commands as vscodeCommands,
  WorkspaceEdit,
  window,
  workspace,
} from "#vscode-host";
import { authorizationRequiredMessage } from "../auth/required.ts";
import { addAuthHeader, isAuthHeaderSuppressed } from "../auth/set.ts";
import { logProviderError } from "../diagnostics/log.ts";
import {
  clearProviderError,
  decreaseProviderBusy,
  increaseProviderBusy,
  setProviderError,
} from "../diagnostics/provider.ts";
import { refreshDiagnostics } from "../diagnostics/refresh.ts";
import { documentInput } from "../documents/input.ts";
import { toRange } from "../documents/range.ts";
import type {
  NativeApplyCommand,
  NativeApplyCommandInput,
} from "../native/input.ts";
import type {
  NativeTextEdit,
  ResolveDocumentOutput,
} from "../native/output.ts";
import { recreateSessions, sessionForResource } from "../session/registry.ts";
import type { ExtensionState } from "../state.ts";
import type {
  ApplyOptions,
  ApplySelection,
  AuthenticationResolution,
  CodeLensReplacementMode,
} from "./apply-types.ts";
import { updateContexts } from "./contexts.ts";

function registerApplyCommands(
  state: ExtensionState,
  commands: [string, NativeApplyCommand][],
): Disposable[] {
  return commands.map(
    ([commandId, command]): Disposable =>
      vscodeCommands.registerCommand(
        commandId,
        (): Promise<void> =>
          applyRustEdits(state, undefined, command, {
            ignoreCodeLensReplace: command === "sort",
          }),
      ),
  );
}

async function applyRustEdits(
  state: ExtensionState,
  dependencyName?: string,
  command?: NativeApplyCommand,
  options: ApplyOptions = {},
): Promise<void> {
  const editor = window.activeTextEditor;
  if (
    !(editor && sessionForResource(state, editor.document.uri)) ||
    (state.flags.codeLensReplace === false && !options.ignoreCodeLensReplace)
  ) {
    return;
  }

  let output = applyCommand(state, editor.document, {
    command,
    dependencyName,
    selectedVersion: options.selectedVersion,
  });
  if (!output) {
    return;
  }
  const selection = {
    command,
    dependencyName,
    selectedVersion: options.selectedVersion,
  };
  output = await resolveAuthenticationIfNeeded(
    state,
    editor.document,
    output,
    selection,
  );
  if (!output) {
    return;
  }
  if (output.edits.length === 0) {
    return;
  }
  if (output.vulnerableUpdateCount > 0) {
    const documentSnapshot = {
      text: editor.document.getText(),
      version: editor.document.version,
    };
    const choice = await window.showWarningMessage(
      vulnerableUpdateMessage(output),
      { modal: true },
      "Update Anyway",
    );
    if (choice !== "Update Anyway") {
      return;
    }
    if (
      editor.document.version !== documentSnapshot.version ||
      editor.document.getText() !== documentSnapshot.text ||
      !areEditRangesValid(documentSnapshot.text, output.edits)
    ) {
      return;
    }
  }

  await applyTextEdits(
    state,
    editor.document,
    output.edits,
    codeLensReplacementMode(selection),
  );
}

function codeLensReplacementMode(
  selection: ApplySelection,
): CodeLensReplacementMode {
  if (selection.command === "sort") {
    return "preserve";
  }
  if (selection.dependencyName || selection.selectedVersion) {
    return "disable";
  }
  return "disableThenEnable";
}

async function applyTextEdits(
  state: ExtensionState,
  document: TextDocument,
  edits: NativeTextEdit[],
  replacementMode: CodeLensReplacementMode,
): Promise<void> {
  const workspaceEdit = new WorkspaceEdit();
  for (const edit of edits) {
    workspaceEdit.replace(document.uri, toRange(edit.range), edit.newText);
  }
  if (replacementMode !== "preserve") {
    state.flags.codeLensReplace = false;
  }
  let applied = false;
  try {
    applied = await workspace.applyEdit(workspaceEdit);
    if (!applied) {
      throw new Error("VersionLens could not apply the requested edits.");
    }
    await refreshDiagnostics(state, document);
  } finally {
    if (replacementMode === "disableThenEnable" && applied) {
      state.flags.codeLensReplace = true;
    }
  }
}

function areEditRangesValid(text: string, edits: NativeTextEdit[]): boolean {
  const lines = text.split("\n").map((line): string => normalizedLine(line));
  return edits.every(({ range }): boolean => {
    const { start, end } = range;
    return (
      isValidPosition(lines, start.line, start.character) &&
      isValidPosition(lines, end.line, end.character) &&
      (start.line < end.line ||
        (start.line === end.line && start.character <= end.character))
    );
  });
}

function normalizedLine(line: string): string {
  if (line.endsWith("\r")) {
    return line.slice(0, -1);
  }
  return line;
}

function isValidPosition(
  lines: string[],
  line: number,
  character: number,
): boolean {
  return (
    Number.isInteger(line) &&
    Number.isInteger(character) &&
    line >= 0 &&
    line < lines.length &&
    character >= 0 &&
    character <= (lines[line]?.length ?? -1)
  );
}

async function resolveAuthenticationIfNeeded(
  state: ExtensionState,
  document: TextDocument,
  output: ResolveDocumentOutput,
  selection: ApplySelection,
): AuthenticationResolution {
  if (!(output.authorizationRequiredCount > 0)) {
    return output;
  }
  const [authRequest] = output.authorizationRequiredRequests;
  let resolution: ResolveDocumentOutput | undefined;
  if (!isAuthHeaderSuppressed(state, authRequest)) {
    const choice = await window.showWarningMessage(
      authorizationRequiredMessage(output.authorizationRequiredCount),
      { modal: true },
      "Add Authentication",
    );
    if (choice === "Add Authentication") {
      const retried = await retryAfterAddingAuthentication(
        state,
        document,
        selection,
        authRequest,
      );
      if (retried && !(retried.authorizationRequiredCount > 0)) {
        resolution = retried;
      }
    }
  }
  return resolution;
}

async function retryAfterAddingAuthentication(
  state: ExtensionState,
  document: TextDocument,
  selection: ApplySelection,
  authRequest: Parameters<typeof addAuthHeader>[1],
): Promise<ResolveDocumentOutput | undefined> {
  if (!(await addAuthHeader(state, authRequest))) {
    return;
  }
  if (!(await reloadAuthBackedSession(state, document))) {
    return;
  }
  return applyCommand(state, document, selection);
}

async function reloadAuthBackedSession(
  state: ExtensionState,
  document: TextDocument,
): Promise<boolean> {
  if (state.context?.extensionPath && !(await recreateSessions(state))) {
    return false;
  }
  await updateContexts(state);
  await refreshDiagnostics(state, document);
  return true;
}

function applyCommand(
  state: ExtensionState,
  document: TextDocument,
  selection: ApplySelection,
): ResolveDocumentOutput | undefined {
  clearProviderError(state);
  increaseProviderBusy(state);
  const { command, dependencyName, selectedVersion } = selection;
  let output: ResolveDocumentOutput | undefined;
  try {
    const input: NativeApplyCommandInput = {
      document: documentInput(document),
    };
    if (command) {
      input.command = command;
    }
    if (dependencyName) {
      input.dependencyName = dependencyName;
    }
    if (selectedVersion) {
      input.selectedVersion = selectedVersion;
    }
    output = sessionForResource(state, document.uri)?.applyCommand(input);
  } catch (error) {
    logProviderError(state, error);
    setProviderError(state);
  } finally {
    decreaseProviderBusy(state);
  }
  return output;
}

function vulnerableUpdateMessage(output: ResolveDocumentOutput): string {
  if (output.vulnerableUpdatePackage && output.vulnerableUpdateVersion) {
    return `Vulnerabilities found in ${output.vulnerableUpdatePackage}@${output.vulnerableUpdateVersion}. Do you want to continue?`;
  }
  if (output.vulnerableUpdateCount === 1) {
    return "Version Lens found a vulnerability for this dependency. Update anyway?";
  }
  return `Version Lens found vulnerabilities for ${output.vulnerableUpdateCount} dependencies. Update anyway?`;
}

export { applyRustEdits, registerApplyCommands };
