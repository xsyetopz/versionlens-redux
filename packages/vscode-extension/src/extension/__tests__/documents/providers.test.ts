import { expect, it } from "../runtime.ts";
import { defaultFilePatternEntries, defaultFilePatterns } from "./patterns.ts";
import { codeLensProviders, configured, testDocument } from "./support.ts";

const cachedLensOutput = {
  canSortDependencies: true,
  codeLenses: [
    {
      arguments: ["left-pad"],
      command: "versionlens.suggestion.onUpdateDependency",
      range: {
        end: { character: 20, line: 0 },
        start: { character: 5, line: 0 },
      },
      title: "left-pad 1.1.0 available",
    },
  ],
  dependencies: [],
  dependencySignature: "left-pad@1.0.0",
  diagnostics: [],
  isSupportedManifest: true,
  status: { text: "Version Lens", tooltip: "1 update", visible: true },
};

interface CachedLensSession {
  analyzeDocument: () => typeof cachedLensOutput;
  resolveDocument: () => {
    authorizationRequiredCount: number;
    authorizationRequiredRequests: never[];
    edits: never[];
    suggestions: never[];
  };
}

interface CachedLensState {
  flags: {
    codeLensReplace: boolean;
    providerBusy: number;
    providerError: boolean;
    showVersionLenses: boolean;
  };
  sessions: Map<string, { resource: undefined; session: object }>;
  ui: { codeLensRefresh: { dispose: () => undefined; fire: () => void } };
}
interface InvocationCounts {
  analyze: number;
  resolve: number;
}

function cachedLensSession(counts: InvocationCounts): CachedLensSession {
  const invocationCounts = counts;
  return {
    analyzeDocument: (): typeof cachedLensOutput => {
      invocationCounts.analyze += 1;
      return cachedLensOutput;
    },
    resolveDocument: (): {
      authorizationRequiredCount: number;
      authorizationRequiredRequests: never[];
      edits: never[];
      suggestions: never[];
    } => {
      invocationCounts.resolve += 1;
      return {
        authorizationRequiredCount: 0,
        authorizationRequiredRequests: [],
        edits: [],
        suggestions: [],
      };
    },
  };
}

function cachedLensState(
  counts: InvocationCounts,
  onRefresh: () => void,
): CachedLensState {
  const invocationCounts = counts;
  const refresh = onRefresh;
  return {
    flags: {
      codeLensReplace: false,
      providerBusy: 0,
      providerError: false,
      showVersionLenses: true,
    },
    sessions: new Map([
      [
        "global",
        { resource: undefined, session: cachedLensSession(invocationCounts) },
      ],
    ]),
    ui: {
      codeLensRefresh: {
        dispose: (): undefined => undefined,
        fire: refresh,
      },
    },
  };
}

it("documentSelectors stays file-backed like upstream CodeLens providers", async (): Promise<void> => {
  const { documentSelectors } = await import("../../documents/selectors.ts");

  expect(documentSelectors()).not.toContainEqual({ scheme: "versionlens" });
});

it("documentSelectors uses configured npm file patterns", async (): Promise<void> => {
  configured["npm.files"] = "**/{package.json,web-module.json}";
  const { documentSelectors } = await import("../../documents/selectors.ts");

  expect(documentSelectors()).toContainEqual({
    language: "json",
    pattern: "**/{package.json,web-module.json}",
    scheme: "file",
  });
  expect(documentSelectors()).toContainEqual({
    language: "jsonc",
    pattern: "**/{package.json,web-module.json}",
    scheme: "file",
  });
});

it("documentSelectors mirrors upstream provider language and file pattern filters", async (): Promise<void> => {
  const { documentSelectors } = await import("../../documents/selectors.ts");
  configured["npm.files"] = undefined;
  configured.enabledProviders = undefined;
  const selectors = documentSelectors();

  for (const [, pattern, languages] of defaultFilePatternEntries) {
    for (const language of languages) {
      expect(selectors).toContainEqual({
        language,
        pattern,
        scheme: "file",
      });
    }
  }
  expect(selectors).not.toContainEqual({ language: "json" });
  expect(selectors).not.toContainEqual({ pattern: defaultFilePatterns[0] });
});

it("documentSelectors filters file-backed providers using enabledProviders like upstream", async (): Promise<void> => {
  const { documentSelectors } = await import("../../documents/selectors.ts");
  configured.enabledProviders = ["npm"];
  configured["npm.files"] = undefined;
  const selectors = documentSelectors();

  expect(selectors).toContainEqual({
    language: "json",
    pattern: "**/{package.json,package.json5,package.yaml,package.yml}",
    scheme: "file",
  });
  expect(selectors).toContainEqual({
    language: "jsonc",
    pattern: "**/{package.json,package.json5,package.yaml,package.yml}",
    scheme: "file",
  });
  expect(selectors).toContainEqual({
    language: "yaml",
    pattern: "**/{package.json,package.json5,package.yaml,package.yml}",
    scheme: "file",
  });
  expect(selectors).not.toContainEqual({
    language: "toml",
    pattern: "**/Cargo.toml",
    scheme: "file",
  });

  configured.enabledProviders = undefined;
});

it("code lens provider renders cached Rust code lenses before background resolve", async (): Promise<void> => {
  const { registerCodeLensProvider } = await import(
    "../../commands/codelens.ts"
  );
  codeLensProviders.length = 0;
  const counts = { analyze: 0, resolve: 0 };
  let refreshCount = 0;
  const document = testDocument();
  const state = cachedLensState(counts, (): void => {
    refreshCount += 1;
  });

  registerCodeLensProvider(state as never);
  (
    state.ui.codeLensRefresh as unknown as {
      event: (listener: () => void) => { dispose: () => void };
    }
  ).event((): void => {
    refreshCount += 1;
  });
  const lenses = await Promise.resolve(
    codeLensProviders[0]?.provideCodeLenses(document),
  );

  expect(counts.resolve).toBe(0);
  expect(counts.analyze).toBe(1);
  expect(state.flags.codeLensReplace).toBe(true);
  expect(lenses).toHaveLength(1);
  expect(lenses?.[0]).toMatchObject({
    command: { command: "versionlens.suggestion.onUpdateDependency" },
  });
  const lens = lenses?.[0];
  if (!lens) {
    throw new Error("expected a CodeLens");
  }
  expect(
    (lens as { command?: { arguments?: unknown[] } }).command?.arguments,
  ).toEqual([lens]);

  await new Promise((resolve): NodeJS.Timeout => setTimeout(resolve, 0));

  expect(counts.resolve).toBe(1);
  expect(refreshCount).toBe(1);

  codeLensProviders[0]?.provideCodeLenses(document);

  expect(counts.analyze).toBe(2);
  expect(counts.resolve).toBe(1);
  expect(refreshCount).toBe(1);
});
