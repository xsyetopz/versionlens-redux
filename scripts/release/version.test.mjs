import { readFileSync } from "node:fs";

const { expect, it } = Bun.jest(import.meta.path);

const RUST_PACKAGES_PATTERN = /const rustPackages = \[(?<body>[\s\S]*?)\n\];/u;

it("release bump covers every root Cargo workspace package", () => {
  const metadata = Bun.spawnSync([
    "cargo",
    "metadata",
    "--no-deps",
    "--format-version",
    "1",
  ]);
  expect(metadata.exitCode).toBe(0);
  const workspace = JSON.parse(metadata.stdout.toString());
  const workspaceMembers = new Set(workspace.workspace_members);
  const expected = workspace.packages
    .filter(({ id }) => workspaceMembers.has(id))
    .map(({ name }) => name)
    .toSorted();

  const source = readFileSync("scripts/release/version.mjs", "utf8");
  const list = RUST_PACKAGES_PATTERN.exec(source)?.groups?.body;
  expect(list).toBeDefined();
  const configured = [...(list ?? "").matchAll(/"(?<name>[^"]+)"/gu)]
    .map((match) => match.groups?.name)
    .filter((name) => name !== undefined)
    .toSorted();

  expect(configured).toEqual(expected);
});
