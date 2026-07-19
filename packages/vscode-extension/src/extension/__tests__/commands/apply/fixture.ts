import { readFileSync } from "node:fs";
import process from "node:process";

function packageFileFixture(name: string): string {
  return readFileSync(
    `${process.cwd()}/tests/fixtures/vscode-extension/${name}`,
    "utf8",
  );
}

export { packageFileFixture };
