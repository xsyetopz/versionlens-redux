import type { TextDocument } from "#vscode-host";
import { documentInput } from "../documents/input.ts";
import type { NativeSession } from "../native/module.ts";
import { sessionForResource } from "../session/registry.ts";
import type { ExtensionState } from "../state.ts";
import { logProviderError } from "./log.ts";
import {
  clearProviderError,
  decreaseProviderBusy,
  increaseProviderBusy,
  setProviderError,
} from "./provider.ts";

interface CachedAnalysis {
  session: NativeSession;
  version: number;
  output: ReturnType<NativeSession["analyzeDocument"]>;
}

const analysisCache = new Map<string, CachedAnalysis>();
const maximumCachedAnalyses = 128;

export function invalidateDocumentAnalysis(document: TextDocument): void {
  analysisCache.delete(document.uri.toString());
}

export function analyzeDocument(
  state: ExtensionState,
  document: TextDocument,
  options?: { rejectOnError?: boolean },
): CachedAnalysis["output"] | undefined {
  const session = sessionForResource(state, document.uri);
  if (!session) {
    return;
  }

  const cacheKey = document.uri.toString();
  const cached = analysisCache.get(cacheKey);
  if (cached?.session === session && cached.version === document.version) {
    return cached.output;
  }

  clearProviderError(state);
  increaseProviderBusy(state);
  let analysis: CachedAnalysis["output"] | undefined;
  try {
    analysis = session.analyzeDocument(documentInput(document));
    if (analysisCache.size >= maximumCachedAnalyses) {
      analysisCache.clear();
    }
    analysisCache.set(cacheKey, {
      session,
      version: document.version,
      output: analysis,
    });
  } catch (error) {
    logProviderError(state, error);
    setProviderError(state);
    if (options?.rejectOnError) {
      throw error;
    }
  } finally {
    decreaseProviderBusy(state);
  }
  return analysis;
}
