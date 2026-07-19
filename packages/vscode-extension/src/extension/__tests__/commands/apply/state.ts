import { packageFileFixture } from "./fixture.ts";

interface CommandState {
  flags: {
    codeLensReplace: boolean;
    providerBusy: number;
    providerError: boolean;
    showPrereleases: boolean;
    showSuggestionStats: boolean;
    showVersionLenses: boolean;
  };
  lifecycle: object;
  sessions: Map<string, { resource: undefined; session: unknown }>;
  ui: object;
  [key: string]: unknown;
}
interface DocumentStub {
  getText: () => string;
  languageId: string;
  uri: { toString: () => string };
}

function commandState(
  session: unknown,
  extra: Record<string, unknown> = {},
): CommandState {
  return {
    lifecycle: {
      externalPackageFileWatchers: new Map(),
      packageFileWatchers: [],
      sessionGenerations: new Map(),
    },
    flags: {
      providerBusy: 0,
      providerError: false,
      codeLensReplace: true,
      showPrereleases: false,
      showSuggestionStats: false,
      showVersionLenses: true,
    },
    sessions: new Map([["global", { resource: undefined, session }]]),
    ui: {
      codeLensRefresh: undefined,
      diagnostics: undefined,
      outputChannel: undefined,
    },
    ...extra,
  };
}

function documentStub(packageName: string): DocumentStub {
  const dependencyName = packageName;
  return {
    getText: (): string =>
      packageFileFixture("package-left-pad-template.json").replace(
        "__PACKAGE__",
        dependencyName,
      ),
    languageId: "json",
    uri: { toString: (): string => "file:///package.json" },
  };
}

export { commandState, documentStub };
