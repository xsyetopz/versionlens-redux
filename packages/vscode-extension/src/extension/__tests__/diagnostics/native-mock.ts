import type { AnalyzeOutput, ResolveOutput, SessionStub } from "./state.ts";
import { diagnosticState, outputFor } from "./state.ts";

type MockModule = Record<string, unknown>;

function nativeSession(): Required<SessionStub> {
  return {
    analyzeDocument: (): AnalyzeOutput =>
      outputFor("file:///workspace/package.json"),
    applyCommand: (): undefined => undefined,
    clearCache: (): undefined => undefined,
    disposeSession: (): undefined => undefined,
    resolveDocument: (): ResolveOutput => {
      diagnosticState.diagnosticSession.reloadedResolveCount += 1;
      return {
        authorizationRequiredCount: 0,
        authorizationRequiredRequests: [],
        edits: [],
        suggestions: [],
        vulnerableUpdateCount: 0,
      };
    },
  };
}

function createNativeMock(): MockModule {
  return {
    loadNative: (): MockModule => ({
      createSession(config: unknown): Required<SessionStub> {
        diagnosticState.diagnosticSession.createdSessionConfigs.push(config);
        return nativeSession();
      },
    }),
  };
}

export { createNativeMock };
