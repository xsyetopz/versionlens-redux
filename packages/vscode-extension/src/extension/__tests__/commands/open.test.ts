import { expect, it, mockVscodeHost } from "../runtime.ts";

type MockModule = Record<string, unknown>;

let fileType = 1;
const openedExternal: unknown[] = [];
const openedTextDocuments: unknown[] = [];

mockVscodeHost(
  (): MockModule =>
    Object.fromEntries([
      [
        "FileType",
        Object.fromEntries([
          ["Directory", 2],
          ["File", 1],
        ]),
      ],
      [
        "Uri",
        {
          file: (path: string): { path: string; scheme: string } => ({
            path,
            scheme: "file",
          }),
        },
      ],
      [
        "env",
        {
          openExternal(uri: unknown): void {
            openedExternal.push(uri);
          },
        },
      ],
      [
        "window",
        {
          showTextDocument(uri: unknown): void {
            openedTextDocuments.push(uri);
          },
        },
      ],
      [
        "workspace",
        {
          fs: {
            stat: (): { type: number } => ({ type: fileType }),
          },
        },
      ],
    ]),
);

it("open dependency opens file paths in the editor", async (): Promise<void> => {
  const { openDependency } = await import("../../commands/open.ts");
  reset();
  fileType = 1;

  await openDependency("/repo/local/package.json");

  expect(openedTextDocuments).toEqual([
    { path: "/repo/local/package.json", scheme: "file" },
  ]);
  expect(openedExternal).toEqual([]);
});

it("open dependency opens directory paths externally", async (): Promise<void> => {
  const { openDependency } = await import("../../commands/open.ts");
  reset();
  fileType = 2;

  await openDependency("/repo/local");

  expect(openedExternal).toEqual([{ path: "/repo/local", scheme: "file" }]);
  expect(openedTextDocuments).toEqual([]);
});

function reset(): void {
  openedExternal.length = 0;
  openedTextDocuments.length = 0;
}
