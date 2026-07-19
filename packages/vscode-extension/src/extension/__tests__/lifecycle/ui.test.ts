import { expect, it, mockVscodeHost } from "../runtime.ts";

type MockModule = Record<string, unknown>;

const diagnosticCollections: string[] = [];
let outputChannelCount = 0;
let statusBarItemCount = 0;

mockVscodeHost(
  (): MockModule =>
    Object.fromEntries([
      [
        "languages",
        {
          createDiagnosticCollection(name: string): {
            clear: () => undefined;
            dispose: () => undefined;
          } {
            diagnosticCollections.push(name);
            return {
              clear: (): undefined => undefined,
              dispose: (): undefined => undefined,
            };
          },
        },
      ],
      ["StatusBarAlignment", Object.fromEntries([["Right", 2]])],
      [
        "window",
        {
          createOutputChannel(): { dispose: () => undefined } {
            outputChannelCount += 1;
            return { dispose: (): undefined => undefined };
          },
          createStatusBarItem(): { dispose: () => undefined } {
            statusBarItemCount += 1;
            return { dispose: (): undefined => undefined };
          },
        },
      ],
    ]),
);

it("initializes upstream diagnostics and output UI without a status bar item", async (): Promise<void> => {
  const { initializeUi } = await import("../../lifecycle/ui.ts");
  const state = {
    ui: {
      codeLensRefresh: undefined,
      diagnostics: undefined,
      outputChannel: undefined,
    },
  };

  initializeUi(state as never);

  expect(diagnosticCollections).toEqual(["versionlens-vulnerabilities"]);
  expect(outputChannelCount).toBe(1);
  expect(statusBarItemCount).toBe(0);
});
