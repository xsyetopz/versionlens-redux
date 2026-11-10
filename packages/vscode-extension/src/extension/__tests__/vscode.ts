type MockModule = Record<string, unknown>;

const vscodeRuntimeExports = [
	"TextDocumentChangeReason",
	"commands",
	"extensions",
	"languages",
	"tasks",
	"window",
	"workspace",
] as const;

const completeVscodeHostMock = (overrides: MockModule): MockModule =>
	Object.fromEntries([
		...vscodeRuntimeExports.map((name): [string, undefined] => [
			name,
			undefined,
		]),
		...Object.entries(overrides),
	]);

export { completeVscodeHostMock };
