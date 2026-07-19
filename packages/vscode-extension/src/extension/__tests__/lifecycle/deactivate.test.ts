import { expect, it, mockVscodeHost } from "../runtime.ts";

type MockModule = Record<string, unknown>;

interface Disposable {
  dispose: () => void;
}

interface DeactivationFixture {
  disposals: string[];
  state: {
    context: { subscriptions: Disposable[] };
    lifecycle: {
      externalPackageFileWatchers: Map<string, Disposable[]>;
      packageFileWatchers: Disposable[];
      sessionGenerations: Map<unknown, unknown>;
    };
    sessions: Map<
      string,
      { resource: undefined; session: { disposeSession: () => void } }
    >;
    snapshots: {
      editedDependencies: Map<string, string>;
      savedDependencies: Map<string, string>;
    };
    ui: {
      codeLensRefresh: undefined;
      diagnostics: undefined;
      outputChannel: undefined;
    };
  };
  subscriptions: Disposable[];
}

mockVscodeHost(
  (): MockModule => ({
    languages: {
      createDiagnosticCollection: (): {
        clear: () => undefined;
        dispose: () => undefined;
      } => ({
        clear: (): undefined => undefined,
        dispose: (): undefined => undefined,
      }),
    },
    window: {
      createOutputChannel: (): { dispose: () => undefined } => ({
        dispose: (): undefined => undefined,
      }),
    },
  }),
);

function deactivationFixture(): DeactivationFixture {
  const disposals: string[] = [];
  const subscriptions = [
    {
      dispose(): void {
        disposals.push("subscription");
      },
    },
  ];
  const state = {
    context: { subscriptions },
    lifecycle: {
      externalPackageFileWatchers: new Map([
        [
          "file:///outside/package.json",
          [
            {
              dispose(): void {
                disposals.push("external-watcher");
              },
            },
          ],
        ],
      ]),
      packageFileWatchers: [
        {
          dispose(): void {
            disposals.push("package-watcher");
          },
        },
      ],
      sessionGenerations: new Map(),
    },
    sessions: new Map(
      ["global", "workspace:file:///workspace"].map((key) => [
        key,
        {
          resource: undefined,
          session: {
            disposeSession(): void {
              disposals.push("session");
            },
          },
        },
      ]),
    ),
    snapshots: {
      editedDependencies: new Map([["file:///package.json", "edited"]]),
      savedDependencies: new Map([["file:///package.json", "saved"]]),
    },
    ui: {
      codeLensRefresh: undefined,
      diagnostics: undefined,
      outputChannel: undefined,
    },
  };
  return { disposals, state, subscriptions };
}

const expectedDeactivationDisposalCount = 5;

it("deactivation disposes package file watchers and clears lifecycle state", async (): Promise<void> => {
  const { deactivateExtension } = await import("../../lifecycle/deactivate.ts");
  const { disposals, state, subscriptions } = deactivationFixture();

  deactivateExtension(state as never);

  expect(disposals).toHaveLength(expectedDeactivationDisposalCount);
  expect(new Set(disposals)).toEqual(
    new Set(["package-watcher", "external-watcher", "subscription", "session"]),
  );
  expect(
    disposals.filter((label): boolean => label === "session"),
  ).toHaveLength(2);
  expect(state.lifecycle.packageFileWatchers).toEqual([]);
  expect(state.lifecycle.externalPackageFileWatchers.size).toBe(0);
  expect(subscriptions).toEqual([]);
  expect(state.sessions.size).toBe(0);
  expect(state.context).toBeUndefined();
  expect(state.snapshots.editedDependencies.size).toBe(0);
  expect(state.snapshots.savedDependencies.size).toBe(0);
});
