import { expect, it } from "../runtime.ts";

import {
  analysisOutput,
  codeLensProviders,
  codeLensState,
  testDocument,
} from "./support.ts";

function loadingLens(): object {
  return {
    arguments: [],
    command: "",
    range: {
      end: { character: 1, line: 0 },
      start: { character: 0, line: 0 },
    },
    title: "[V]",
  };
}

it("code lens provider renders a visible terminal error after resolve failure", async (): Promise<void> => {
  const { registerCodeLensProvider } = await import(
    "../../commands/codelens.ts"
  );
  codeLensProviders.length = 0;
  const failure = new Error("resolve failed");
  let resolveCount = 0;
  const loading = loadingLens();
  const document = testDocument(
    "file:///build.gradle.kts",
    "kotlin",
    'plugins { kotlin("jvm") version "2.0.0" }\n',
  );
  const state = codeLensState({
    analyzeDocument: (): ReturnType<typeof analysisOutput> =>
      analysisOutput([loading]),
    resolveDocument: (): never => {
      resolveCount += 1;
      throw failure;
    },
  });

  registerCodeLensProvider(state as never);
  let refreshCount = 0;
  let reevaluatedLenses: unknown[] = [];
  codeLensProviders[0]?.onDidChangeCodeLenses?.((): void => {
    refreshCount += 1;
    reevaluatedLenses = codeLensProviders[0]?.provideCodeLenses(document) ?? [];
  });

  expect(codeLensProviders[0]?.provideCodeLenses(document)).toHaveLength(1);
  await new Promise((resolve): NodeJS.Timeout => setTimeout(resolve, 0));
  expect(state.flags.providerError).toBe(true);
  expect(state.flags.providerBusy).toBe(0);
  expect(refreshCount).toBe(1);
  expect(reevaluatedLenses).toHaveLength(1);
  expect(
    (reevaluatedLenses[0] as { command: { title: string } }).command.title,
  ).toBe("[V] Unable to resolve dependencies");
  expect(resolveCount).toBe(1);

  document.version = 2;
  codeLensProviders[0]?.provideCodeLenses(document);
  await new Promise((resolve): NodeJS.Timeout => setTimeout(resolve, 0));
  expect(resolveCount).toBe(2);
});

it("stale resolutions refresh the current terminal state", async (): Promise<void> => {
  const { registerCodeLensProvider } = await import(
    "../../commands/codelens.ts"
  );
  codeLensProviders.length = 0;
  let finishResolution: ((value: unknown) => void) | undefined;
  const pendingResolution = new Promise((resolve): void => {
    finishResolution = resolve;
  });
  let refreshCount = 0;
  let resolveCount = 0;
  const document = {
    ...testDocument(
      "file:///build.gradle.kts",
      "kotlin",
      'plugins { kotlin("jvm") version "2.0.0" }\n',
    ),
    version: 1,
  };
  const loading = loadingLens();
  const terminalLens = { ...loading, title: "kotlin-jvm 2.1.0" };
  const state = codeLensState({
    analyzeDocument: (): ReturnType<typeof analysisOutput> =>
      analysisOutput(document.version === 1 ? [loading] : [terminalLens]),
    resolveDocument: (): Promise<unknown> => {
      resolveCount += 1;
      return pendingResolution;
    },
  });

  registerCodeLensProvider(state as never);
  codeLensProviders[0]?.onDidChangeCodeLenses?.((): void => {
    refreshCount += 1;
    codeLensProviders[0]?.provideCodeLenses(document);
  });
  const initialLenses = codeLensProviders[0]?.provideCodeLenses(document) ?? [];
  expect(initialLenses).toHaveLength(1);
  expect(
    (initialLenses[0] as { command: { title: string } }).command.title,
  ).toBe("[V]");
  await new Promise((resolve): NodeJS.Timeout => setTimeout(resolve, 0));

  document.version = 2;
  finishResolution?.({
    authorizationRequiredCount: 0,
    authorizationRequiredRequests: [],
    edits: [],
    suggestions: [],
  });
  await new Promise((resolve): NodeJS.Timeout => setTimeout(resolve, 0));
  await new Promise((resolve): NodeJS.Timeout => setTimeout(resolve, 0));

  expect(refreshCount).toBe(2);
  expect(resolveCount).toBe(2);
  const terminalLenses =
    codeLensProviders[0]?.provideCodeLenses(document) ?? [];
  expect(terminalLenses).toHaveLength(1);
  expect(
    (terminalLenses[0] as { command: { title: string } }).command.title,
  ).toBe("kotlin-jvm 2.1.0");
  expect(state.flags.providerBusy).toBe(0);
});

it("disposed CodeLens generations cannot complete or refresh a replacement provider", async (): Promise<void> => {
  const { registerCodeLensProvider } = await import(
    "../../commands/codelens.ts"
  );
  codeLensProviders.length = 0;
  let finishOlder: ((value: unknown) => void) | undefined;
  const olderResolution = new Promise((resolve): void => {
    finishOlder = resolve;
  });
  let newerResolveCount = 0;
  const output = analysisOutput();
  const olderSession = {
    analyzeDocument: (): typeof output => output,
    resolveDocument: (): Promise<unknown> => olderResolution,
  };
  const newerSession = {
    analyzeDocument: (): typeof output => output,
    resolveDocument: (): {
      authorizationRequiredCount: number;
      authorizationRequiredRequests: never[];
      edits: never[];
      suggestions: never[];
    } => {
      newerResolveCount += 1;
      return {
        authorizationRequiredCount: 0,
        authorizationRequiredRequests: [],
        edits: [],
        suggestions: [],
      };
    },
  };
  const document = { ...testDocument("file:///race-package.json"), version: 1 };
  const state = codeLensState(olderSession);

  registerCodeLensProvider(state as never);
  codeLensProviders[0]?.provideCodeLenses(document);
  await new Promise((resolve): NodeJS.Timeout => setTimeout(resolve, 0));
  state.sessions.set("global", { resource: undefined, session: newerSession });
  registerCodeLensProvider(state as never);
  let replacementRefreshCount = 0;
  codeLensProviders[1]?.onDidChangeCodeLenses?.((): void => {
    replacementRefreshCount += 1;
  });
  finishOlder?.({
    authorizationRequiredCount: 0,
    authorizationRequiredRequests: [],
    edits: [],
    suggestions: [],
  });
  await new Promise((resolve): NodeJS.Timeout => setTimeout(resolve, 0));

  expect(replacementRefreshCount).toBe(0);
  codeLensProviders[1]?.provideCodeLenses(document);
  await new Promise((resolve): NodeJS.Timeout => setTimeout(resolve, 0));
  expect(newerResolveCount).toBe(1);
  expect(replacementRefreshCount).toBe(1);
});

it("code lens provider does not resolve or refresh after lenses are hidden", async (): Promise<void> => {
  const { registerCodeLensProvider } = await import(
    "../../commands/codelens.ts"
  );
  codeLensProviders.length = 0;
  let resolveDocumentCount = 0;
  let refreshCount = 0;
  const document = testDocument();
  const state = codeLensState({
    analyzeDocument: (): ReturnType<typeof analysisOutput> => analysisOutput(),
    resolveDocument: (): { edits: never[]; suggestions: never[] } => {
      resolveDocumentCount += 1;
      return { edits: [], suggestions: [] };
    },
  });

  registerCodeLensProvider(state as never);
  codeLensProviders[0]?.onDidChangeCodeLenses?.((): void => {
    refreshCount += 1;
  });
  codeLensProviders[0]?.provideCodeLenses(document);
  state.flags.showVersionLenses = false;

  await new Promise((resolve): NodeJS.Timeout => setTimeout(resolve, 0));

  expect(resolveDocumentCount).toBe(0);
  expect(refreshCount).toBe(0);
});

it("code lens provider rejects when native analyze fails", async (): Promise<void> => {
  const { registerCodeLensProvider } = await import(
    "../../commands/codelens.ts"
  );
  codeLensProviders.length = 0;
  const failure = new Error("analyze failed");
  const document = testDocument();
  const state = codeLensState({
    analyzeDocument: (): never => {
      throw failure;
    },
    resolveDocument: (): { edits: never[]; suggestions: never[] } => ({
      edits: [],
      suggestions: [],
    }),
  });

  registerCodeLensProvider(state as never);

  expect((): unknown[] | undefined =>
    codeLensProviders[0]?.provideCodeLenses(document),
  ).toThrow(failure);
  expect(state.flags.providerError).toBe(true);
});
