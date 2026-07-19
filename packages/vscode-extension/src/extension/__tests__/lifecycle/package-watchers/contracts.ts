interface Disposable {
  dispose: () => void;
}

interface DocumentUri {
  fsPath?: string;
}

interface TestUri {
  fsPath: string;
  scheme: string | undefined;
  toString: () => string;
}

interface TestDocument {
  getText: () => string;
  isDirty: boolean;
  languageId: string;
  uri: TestUri;
}

interface TestConfiguration {
  get: (key: string, fallback?: unknown) => unknown;
}

interface TestWorkspaceFolder {
  uri: { fsPath: string };
}

export type {
  Disposable,
  DocumentUri,
  TestConfiguration,
  TestDocument,
  TestUri,
  TestWorkspaceFolder,
};
