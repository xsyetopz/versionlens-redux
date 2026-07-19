import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";

const { afterEach, describe, expect, it } = Bun.jest(import.meta.path);

const entrypoint = resolve(import.meta.dir, "../quality.mjs");
const temporaryDirectories = [];

function fixture(files) {
  const directory = mkdtempSync(join(tmpdir(), "versionlens-quality-test-"));
  temporaryDirectories.push(directory);
  for (const [filePath, source] of Object.entries(files)) {
    const absolutePath = join(directory, filePath);
    mkdirSync(dirname(absolutePath), { recursive: true });
    writeFileSync(absolutePath, source);
  }
  return directory;
}

function run(directory) {
  return Bun.spawnSync(["bun", entrypoint], {
    cwd: directory,
    stderr: "pipe",
    stdout: "pipe",
  });
}

const cargoBuildAdapter = `
use napi_build::setup;
fn main() {
    setup();
}
`;

afterEach(() => {
  for (const directory of temporaryDirectories.splice(0)) {
    rmSync(directory, { force: true, recursive: true });
  }
});

describe("quality command", () => {
  it("exits zero and reports an active checked exception", () => {
    const directory = fixture({
      "crates/example/src/lib.rs": "fn parse(value: &str) -> &str { value }",
      "crates/versionlens-napi/build.rs": cargoBuildAdapter,
    });
    const result = run(directory);

    expect(result.exitCode).toBe(0);
    expect(result.stderr.toString()).toContain(
      "checked quality exceptions (1)",
    );
  });

  it("exits nonzero for an unexcepted production finding", () => {
    const duplicateBody = `
    let one = value.trim();
    let two = one.to_lowercase();
    let three = two.replace('-', "_");
    let four = three.replace('.', "_");
    let five = four.split('_').collect::<Vec<_>>();
    let six = five.join("-");
    let seven = six.trim_matches('-');
    let eight = seven.to_owned();
    let nine = eight.to_lowercase();
    let ten = nine.replace("--", "-");
    ten
`;
    const directory = fixture({
      "crates/example/src/lib.rs": `
fn normalize_one(value: &str) -> String {${duplicateBody}}
fn normalize_two(value: &str) -> String {${duplicateBody}}
`,
      "crates/versionlens-napi/build.rs": cargoBuildAdapter,
    });
    const result = run(directory);

    expect(result.exitCode).toBe(1);
    expect(result.stderr.toString()).toContain("duplicate logic");
  });

  it("fails when a checked exception no longer matches", () => {
    const directory = fixture({
      "crates/example/src/lib.rs": "fn parse(value: &str) -> &str { value }",
    });
    const result = run(directory);

    expect(result.exitCode).toBe(1);
    expect(result.stderr.toString()).toContain("stale quality exception");
  });
});
