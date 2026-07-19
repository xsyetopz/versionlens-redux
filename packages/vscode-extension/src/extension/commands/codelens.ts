import {
  CodeLens,
  type Disposable,
  EventEmitter,
  languages,
  type TextDocument,
} from "#vscode-host";
import { analyzeDocument } from "../diagnostics/analyze.ts";
import { resolveDocumentForDiagnostics } from "../diagnostics/resolve.ts";
import { toRange } from "../documents/range.ts";
import { documentSelectors } from "../documents/selectors.ts";
import type { NativeCodeLensPayload } from "../native/output.ts";
import type { ExtensionState } from "../state.ts";

const nativeArgumentsByCodeLens = new WeakMap<object, string[]>();

interface CodeLensResolutionContext {
  state: ExtensionState;
  owner: Disposable;
  refresh: EventEmitter<void>;
  resolutions: {
    pending: Set<string>;
    completed: Set<string>;
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
      if (output) {
        scheduleCodeLensResolution(
          { state, owner, refresh, resolutions },
          document,
          output.dependencySignature,
        );
      }
      state.flags.codeLensReplace = true;
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
      if (state.ui.codeLensProvider === owner) {
        state.ui.codeLensProvider = undefined;
        state.ui.codeLensRefresh = undefined;
        state.ui.resetCodeLensResolutions = undefined;
      }
    },
  };
  state.ui.codeLensProvider = owner;
  state.ui.resetCodeLensResolutions = (): void => resolutions.completed.clear();
  return owner;
}

function scheduleCodeLensResolution(
  context: CodeLensResolutionContext,
  document: TextDocument,
  dependencySignature: string,
): void {
  const { state, owner, refresh, resolutions } = context;
  const key = `${document.uri.toString()}\0${dependencySignature}`;
  if (
    dependencySignature === "" ||
    resolutions.pending.has(key) ||
    resolutions.completed.has(key)
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
        refresh.fire();
      })
      .catch((): undefined => undefined)
      .finally((): void => {
        resolutions.pending.delete(key);
      });
  }, 0);
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
