type MockModule = Record<string, unknown>;

const vscodeRuntimeExports = [
  "TextDocumentChangeReason",
  "commands",
  "extensions",
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
