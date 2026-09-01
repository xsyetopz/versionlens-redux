import {
  CodeLens,
  type Disposable,
  EventEmitter,
  languages,
  Range,
  type TextDocument,
} from "#vscode-host";
import { analyzeDocument } from "../diagnostics/analyze.ts";
import { resolveDocumentForDiagnostics } from "../diagnostics/resolve.ts";
import { documentSelectors, toRange } from "../documents.ts";
import type { NativeCodeLensPayload } from "../native.ts";
import type { ExtensionState } from "../state.ts";

const nativeArgumentsByCodeLens = new WeakMap<object, string[]>();

interface CodeLensResolutionContext {
  state: ExtensionState;
  owner: Disposable;
  refresh: EventEmitter<void>;
  resolutions: {
    pending: Set<string>;
    completed: Set<string>;
    failed: Set<string>;
  };
}

interface CodeLensProviderRegistration {
  dispose: () => void;
}

function registerCodeLensProvider(
  state: ExtensionState,
): CodeLensProviderRegistration {
  state.ui.codeLensProvider?.dispose();
  const resolutions = {
    pending: new Set<string>(),
    completed: new Set<string>(),
    failed: new Set<string>(),
  };
  const refresh = new EventEmitter<void>();
  state.ui.codeLensRefresh = refresh;
  const registration = languages.registerCodeLensProvider(documentSelectors(), {
    onDidChangeCodeLenses: refresh.event,
    provideCodeLenses(document: TextDocument): CodeLens[] {
      if (!state.flags.showVersionLenses) {
        return [];
      }

      const output = analyzeDocument(state, document, {
        rejectOnError: true,
      });
      const failed = output
        ? resolutions.failed.has(
            failedResolutionKey(document, output.dependencySignature),
          )
        : false;
      if (output && !failed) {
        scheduleCodeLensResolution(
          { state, owner, refresh, resolutions },
          document,
          output.dependencySignature,
        );
      }
      state.flags.codeLensReplace = true;
      if (failed) {
        return [new CodeLens(new Range(0, 0, 0, 0))].map((lens) => {
          lens.command = {
            command: "",
            title: "[V] Unable to resolve dependencies",
          };
          return lens;
        });
      }
      return (output?.codeLenses ?? []).map(toCodeLens);
    },
  });
  let disposed = false;
  const owner = {
    dispose(): void {
      if (disposed) {
        return;
      }
      disposed = true;
      registration.dispose();
      refresh.dispose();
      resolutions.pending.clear();
      resolutions.completed.clear();
      resolutions.failed.clear();
      if (state.ui.codeLensProvider === owner) {
        state.ui.codeLensProvider = undefined;
        state.ui.codeLensRefresh = undefined;
        state.ui.resetCodeLensResolutions = undefined;
      }
    },
  };
  state.ui.codeLensProvider = owner;
  state.ui.resetCodeLensResolutions = (): void => {
    resolutions.completed.clear();
    resolutions.failed.clear();
  };
  return owner;
}

function scheduleCodeLensResolution(
  context: CodeLensResolutionContext,
  document: TextDocument,
  dependencySignature: string,
): void {
  const { state, owner, refresh, resolutions } = context;
  const documentVersion = document.version;
  const key = resolutionKey(document, dependencySignature);
  const failedKey = failedResolutionKey(document, dependencySignature);
  if (
    dependencySignature === "" ||
    resolutions.pending.has(key) ||
    resolutions.completed.has(key) ||
    resolutions.failed.has(failedKey)
  ) {
    return;
  }

  resolutions.pending.add(key);
  setTimeout((): void => {
    if (state.ui.codeLensProvider !== owner || !state.flags.showVersionLenses) {
      resolutions.pending.delete(key);
      return;
    }
    resolveDocumentForDiagnostics(state, document, { rejectOnError: true })
      .then((completed): void => {
        if (
          !completed ||
          state.ui.codeLensProvider !== owner ||
          !state.flags.showVersionLenses
        ) {
          return;
        }
        resolutions.completed.add(key);
      })
      .catch((): void => {
        if (
          state.ui.codeLensProvider === owner &&
          state.flags.showVersionLenses &&
          document.version === documentVersion
        ) {
          resolutions.failed.add(failedKey);
        }
      })
      .finally((): void => {
        resolutions.pending.delete(key);
        if (
          state.ui.codeLensProvider === owner &&
          state.flags.showVersionLenses
        ) {
          refresh.fire();
        }
      });
  }, 0);
}

function resolutionKey(
  document: TextDocument,
  dependencySignature: string,
): string {
  return `${document.uri.toString()}\0${dependencySignature}`;
}

function failedResolutionKey(
  document: TextDocument,
  dependencySignature: string,
): string {
  return `${resolutionKey(document, dependencySignature)}\0${document.version}`;
}

function nativeCodeLensArguments(argument: unknown): string[] | undefined {
  if (typeof argument !== "object" || argument === null) {
    return;
  }

  return nativeArgumentsByCodeLens.get(argument);
}

function toCodeLens(lens: NativeCodeLensPayload): CodeLens {
  const rendered = new CodeLens(toRange(lens.range));
  nativeArgumentsByCodeLens.set(rendered, lens.arguments);
  rendered.command = {
    command: lens.command,
    title: lens.title,
  };
  if (lens.command) {
    rendered.command.arguments = [rendered];
  }
  return rendered;
}

function refreshCodeLenses(state: ExtensionState): void {
  state.ui.resetCodeLensResolutions?.();
  state.ui.codeLensRefresh?.fire();
}

export { nativeCodeLensArguments, refreshCodeLenses, registerCodeLensProvider };
