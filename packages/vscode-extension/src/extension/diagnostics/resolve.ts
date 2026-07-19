import { type TextDocument, window } from "#vscode-host";
import { authorizationRequiredMessage } from "../auth/required.ts";
import { addAuthHeader, isAuthHeaderSuppressed } from "../auth/set.ts";
import { documentInput } from "../documents/input.ts";
import type { NativeSession } from "../native/module.ts";
import { recreateSessions, sessionForResource } from "../session/registry.ts";
import type { ExtensionState } from "../state.ts";
import { invalidateDocumentAnalysis } from "./analyze.ts";
import { logProviderError } from "./log.ts";
import {
  clearProviderError,
  decreaseProviderBusy,
  increaseProviderBusy,
  setProviderError,
} from "./provider.ts";

const addAuthenticationChoice = "Add Authentication";
const pendingAuthenticationDocuments = new Set<string>();

interface DocumentResolutionContext {
  state: ExtensionState;
  session: NativeSession;
  document: TextDocument;
  documentVersion: number;
  documentText: string;
}

type DocumentResolution = Promise<boolean>;

async function resolveDocumentForDiagnostics(
  state: ExtensionState,
  document: TextDocument,
  options?: { rejectOnError?: boolean },
): DocumentResolution {
  const session = sessionForResource(state, document.uri);
  if (!session) {
    return false;
  }
  const input = documentInput(document);
  const documentVersion = document.version;
  const resolution = {
    state,
    session,
    document,
    documentVersion,
    documentText: input.text,
  };

  clearProviderError(state);
  increaseProviderBusy(state);
  try {
    const output = await session.resolveDocument(input);
    if (!(output && ownsResolution(resolution))) {
      return false;
    }
    invalidateDocumentAnalysis(document);
    return await offerAuthenticationForDocument(
      resolution,
      output.authorizationRequiredCount,
      output.authorizationRequiredRequests[0],
    );
  } catch (error) {
    if (sessionForResource(state, document.uri) === session) {
      logProviderError(state, error);
      setProviderError(state);
    }
    if (options?.rejectOnError) {
      throw error;
    }
    return false;
  } finally {
    decreaseProviderBusy(state);
  }
}

async function offerAuthenticationForDocument(
  resolution: DocumentResolutionContext,
  count: number,
  authRequest: Parameters<typeof addAuthHeader>[1],
): Promise<boolean> {
  const { state, document } = resolution;
  if (count === 0 || isAuthHeaderSuppressed(state, authRequest)) {
    return ownsResolution(resolution);
  }
  const key = document.uri.toString();
  if (pendingAuthenticationDocuments.has(key)) {
    return ownsResolution(resolution);
  }
  pendingAuthenticationDocuments.add(key);
  try {
    const choice = await window.showWarningMessage(
      authorizationRequiredMessage(count),
      { modal: true },
      addAuthenticationChoice,
    );
    if (!ownsResolution(resolution)) {
      return false;
    }
    if (choice !== addAuthenticationChoice) {
      return true;
    }
    if (!(await addAuthHeader(state, authRequest))) {
      return ownsResolution(resolution);
    }
    if (!ownsResolution(resolution)) {
      return false;
    }
    if (!(await recreateSessions(state))) {
      return false;
    }
    const reloadedSession = sessionForResource(state, document.uri);
    if (!reloadedSession) {
      return false;
    }
    const reloadedInput = documentInput(document);
    const reloadedVersion = document.version;
    try {
      await reloadedSession.resolveDocument(reloadedInput);
    } catch (error) {
      if (sessionForResource(state, document.uri) === reloadedSession) {
        logProviderError(state, error);
        setProviderError(state);
      }
      return false;
    }
    if (
      !ownsResolution({
        state,
        session: reloadedSession,
        document,
        documentVersion: reloadedVersion,
        documentText: reloadedInput.text,
      })
    ) {
      return false;
    }
    invalidateDocumentAnalysis(document);
    state.ui.codeLensRefresh?.fire();
    return true;
  } finally {
    pendingAuthenticationDocuments.delete(key);
  }
}

function ownsResolution(resolution: DocumentResolutionContext): boolean {
  const { state, session, document, documentVersion, documentText } =
    resolution;
  return (
    sessionForResource(state, document.uri) === session &&
    document.version === documentVersion &&
    document.getText() === documentText
  );
}

export { resolveDocumentForDiagnostics };
