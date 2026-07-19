type MockModule = Record<string, unknown>;

const vscodeRuntimeExports = [
  "CodeLens",
  "Diagnostic",
  "EventEmitter",
  "FileType",
  "Range",
  "RelativePattern",
  "TextDocumentChangeReason",
  "Uri",
  "WorkspaceEdit",
  "commands",
  "env",
  "extensions",
  "languages",
  "tasks",
  "window",
  "workspace",
] as const;

function completeVscodeHostMock(overrides: MockModule): MockModule {
  return Object.fromEntries([
    ...vscodeRuntimeExports.map((name): [string, undefined] => [
      name,
      undefined,
    ]),
    ...Object.entries(overrides),
  ]);
}

export { completeVscodeHostMock };
