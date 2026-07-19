import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import process from "node:process";
import { expect, it } from "./runtime.ts";

interface NativeOutput {
  [key: string]: unknown;
  codeLenses: unknown[];
  dependencies: unknown[];
  diagnostics: unknown[];
  edits: Array<{ newText?: string }>;
  isSupportedManifest: boolean;
  status: { visible: boolean; vulnerabilityCount: number };
  suggestions: unknown[];
}

interface NativeSession {
  analyzeDocument: (input: object) => NativeOutput;
  applyCommand: (input: object) => NativeOutput;
  clearCache: () => void;
  disposeSession: () => void;
  resolveDocument: (input: object) => Promise<NativeOutput>;
}

interface NativeModule {
  createSession: (options: object) => NativeSession;
}
function native(): NativeModule {
  const loaded: { exports: Partial<NativeModule> } = { exports: {} };
  process.dlopen(
    loaded,
    resolve("packages/vscode-extension/native/versionlens_napi.node"),
  );
  return loaded.exports as NativeModule;
}

function createSession(options: object = {}): NativeSession {
  return native().createSession({
    http: { proxy: "", strictSsl: true, timeoutMs: 10_000 },
    showPrereleases: false,
    ...options,
  });
}

it("applyCommand sorts requirements dependencies", (): void => {
  const session = createSession();

  const output = session.applyCommand({
    command: "sort",
    document: {
      languageId: "pip-requirements",
      text: packageFileFixture("requirements-unsorted.txt"),
      uri: "file:///requirements.txt",
    },
  });

  expect(output.edits).toHaveLength(2);
  expect(output.edits[0]?.newText).toBe("alpha==1");
  expect(output.edits[1]?.newText).toBe("zeta==1");
});

it("applyCommand updates project version", (): void => {
  const session = createSession();

  const output = session.applyCommand({
    command: "updateMajor",
    dependencyName: "1.2.3",
    document: {
      languageId: "json",
      text: packageFileFixture("package-project-version.json"),
      uri: "file:///package.json",
    },
  });

  expect(output.edits).toHaveLength(1);
  expect(output.edits[0]?.newText).toBe("2.0.0");
});

it("resolveDocument is callable without registry work", async (): Promise<void> => {
  const session = createSession();

  const output = await session.resolveDocument({
    languageId: "json",
    text: packageFileFixture("package-workspace-local.json"),
    uri: "file:///package.json",
  });

  expect(output.edits).toHaveLength(0);
  expect(output.suggestions).toHaveLength(0);
});

it("clearCache and disposeSession are callable", (): void => {
  const session = createSession();

  expect(() => session.clearCache()).not.toThrow();
  expect(() => session.disposeSession()).not.toThrow();
});

it("disposeSession releases the native Rust session", async (): Promise<void> => {
  const session = createSession();
  const input = {
    languageId: "json",
    text: packageFileFixture("package-left-pad.json"),
    uri: "file:///package.json",
  };

  expect(session.analyzeDocument(input).isSupportedManifest).toBe(true);

  session.disposeSession();
  session.clearCache();

  const analyzed = session.analyzeDocument(input);
  expect(analyzed.isSupportedManifest).toBe(false);
  expect(analyzed.status.visible).toBe(false);
  expect((await session.resolveDocument(input)).edits).toHaveLength(0);
  expect(
    session.applyCommand({ command: "updateMajor", document: input }).edits,
  ).toHaveLength(0);
});

it("analyzeDocument can disable vulnerability diagnostics", (): void => {
  const session = createSession({ showVulnerabilities: false });

  const output = session.analyzeDocument({
    languageId: "json",
    text: packageFileFixture("package-left-pad.json"),
    uri: "file:///package.json",
  });

  expect(output.diagnostics).toHaveLength(0);
  expect(output.dependencies).toHaveLength(1);
  expect(output.dependencies[0]).toMatchObject({
    ecosystem: "npm",
    group: "dependencies",
    name: "left-pad",
    requirement: "1.0.0",
  });
  expect(output.status.vulnerabilityCount).toBe(0);
});

it("analyzeDocument omits native missing-suggestion code lens payloads", (): void => {
  const session = createSession({
    suggestionIndicators: { updateable: "U" },
  });

  const output = session.analyzeDocument({
    languageId: "json",
    text: packageFileFixture("package-left-pad.json"),
    uri: "file:///package.json",
  });

  expect(output.codeLenses).toHaveLength(0);
});

it("analyzeDocument omits schema diagnostics across N-API", (): void => {
  const session = createSession();

  const output = session.analyzeDocument({
    languageId: "json",
    text: packageFileFixture("versionlens-schema.json"),
    uri: "versionlens:/versionlens.multi-registries.json",
  });

  expect(output.isSupportedManifest).toBe(true);
  expect(output.diagnostics).toHaveLength(0);
});

function packageFileFixture(name: string): string {
  return readFileSync(`${process.cwd()}/tests/fixtures/napi/${name}`, "utf8");
}
