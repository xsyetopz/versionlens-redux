import { formatFindings } from "../../report.mjs";
import { analyzeSources } from "../analysis.mjs";
import { rustConfig } from "./fixtures.mjs";

export function registerQualityShapeCase0({ it, expect }) {
  it("does not report enum and type constructors as pass-through function wrappers", () => {
    const result = analyzeSources([
      {
        path: "crates/example/src/constructors.rs",
        language: "rust",
        source: `
fn read_value(response: Response) -> Result<String, Error> {
    Ok(response.into_string()?)
}
`,
      },
    ]);

    expect(result.passThroughWrappers).toEqual([]);
  });
}

export function registerQualityShapeCase1({ it, expect }) {
  it("does not report chained expressions as pass-through wrappers", () => {
    const result = analyzeSources([
      {
        path: "crates/example/src/chained.rs",
        language: "rust",
        source: `
fn package_stanzas(text: &str) -> Vec<Stanza> {
    top_level_stanzas(text)
        .into_iter()
        .filter(|stanza| stanza.open > 0)
        .collect()
}
`,
      },
    ]);

    expect(result.passThroughWrappers).toEqual([]);
  });
}

export function registerQualityShapeCase2({ it, expect }) {
  it("parses TypeScript default parameters and ignores non-arrow const expressions", () => {
    const result = analyzeSources([
      {
        path: "packages/vscode-extension/src/example.ts",
        language: "typescript",
        source: `
function label(state: ExtensionState, document = vscode.window.activeTextEditor?.document) {
	return document ? render(state, document) : undefined;
}

async function runTask(label: string) {
	const task = (await vscode.tasks.fetchTasks()).find((item) => item.name === label);
	return task;
}
`,
      },
    ]);

    expect(result.unusedParameters).toEqual([]);
  });
}

export function registerQualityShapeCase3({ it, expect }) {
  it("parses Rust function bodies containing char literals", () => {
    const result = analyzeSources([
      {
        path: "crates/example/src/chars.rs",
        language: "rust",
        source: `
fn parse_source(text: &str, source: &str, source_offset: usize) -> Range {
    let tuple_end = source.find('}').map(|index| source_offset + index).unwrap_or(source.len());
    offset_range(text, source_offset, tuple_end)
}
`,
      },
    ]);

    expect(result.unusedParameters).toEqual([]);
  });
}

export function registerQualityShapeCase4({ it, expect }) {
  it("can ignore cfg-test-only Rust pass-through helpers", () => {
    const result = analyzeSources(
      [
        {
          path: "crates/example/src/lib.rs",
          language: "rust",
          source: `
#[cfg(test)]
fn helper(value: &Config) -> bool {
    cache_key(value)
}

fn adapter(value: &Config) -> bool {
    cache_key(value)
}
`,
        },
      ],
      { ignoreTestFilesForPassThrough: true },
    );

    expect(result.passThroughWrappers).toContainEqual(
      expect.objectContaining({ name: "adapter" }),
    );
    expect(result.passThroughWrappers).not.toContainEqual(
      expect.objectContaining({ name: "helper" }),
    );
  });
}

export function registerQualityShapeCase5({ it, expect }) {
  it("can ignore public API type spellings and public adapter wrappers", () => {
    const result = analyzeSources(
      [
        {
          path: "crates/example/src/api.rs",
          language: "rust",
          source: `
pub fn parse_one(text: &str) -> Vec<Dependency> {
    parse_with_paths(text, &[])
}

pub(crate) fn parse_two(text: &str) -> Vec<Dependency> {
    parse_with_paths(text, &[])
}

fn private_one(text: &str) -> Vec<Dependency> {
    parse_with_paths(text, &[])
}

fn private_two(text: &str) -> Vec<Dependency> {
    parse_with_paths(text, &[])
}
`,
        },
      ],
      {
        ignorePublicApiTypes: true,
        ignorePublicPassThroughWrappers: true,
      },
    );

    expect(result.repeatedComplexTypes).toContainEqual(
      expect.objectContaining({
        typeText: "Vec<Dependency>",
        count: 2,
      }),
    );
    expect(result.passThroughWrappers).not.toContainEqual(
      expect.objectContaining({ name: "parse_one" }),
    );
    expect(result.passThroughWrappers).not.toContainEqual(
      expect.objectContaining({ name: "parse_two" }),
    );
    expect(result.passThroughWrappers).toContainEqual(
      expect.objectContaining({ name: "private_one" }),
    );
  });
}

export function registerQualityShapeCase6({ it, expect }) {
  it("formats paths, ranges, counts, and diffs", () => {
    const result = analyzeSources([
      {
        path: "crates/example/src/lib.rs",
        language: "rust",
        source: rustConfig,
      },
    ]);

    const output = formatFindings(result, {
      diffCommand: "diff",
      color: false,
    });

    expect(output).toContain("duplicate logic");
    expect(output).toContain("crates/example/src/lib.rs:");
    expect(output).toContain("fetch_one");
    expect(output).toContain("fetch_two");
    expect(output).toContain("@@");
    expect(output).toContain("repeated complex types");
    expect(output).toContain("Vec<ResolvedDependency> count=2");
    expect(output).toContain("oversized shapes");
    expect(output).toContain("BuildRequest fields=11");
  });
}

export function registerQualityShapeCase7({ it, expect }) {
  it("reports overqualified Rust crate and std paths", () => {
    const result = analyzeSources([
      {
        path: "crates/example/src/support.rs",
        language: "rust",
        source: `
use crate::MemoryCache;
use std::fs;

fn create() -> crate::MemoryCache<String> {
    crate::memory::memory_cache(std::time::Duration::from_secs(1))
}
`,
      },
    ]);

    expect(result.overqualifiedPaths).toContainEqual(
      expect.objectContaining({
        path: "crates/example/src/support.rs",
        line: 5,
        kind: "crate-type",
        qualified: "crate::MemoryCache",
        suggested: "MemoryCache",
      }),
    );
    expect(result.overqualifiedPaths).toContainEqual(
      expect.objectContaining({
        path: "crates/example/src/support.rs",
        line: 6,
        kind: "crate-module-call",
        qualified: "crate::memory::memory_cache",
        suggested: "memory::memory_cache()",
      }),
    );
    expect(result.overqualifiedPaths).toContainEqual(
      expect.objectContaining({
        path: "crates/example/src/support.rs",
        line: 6,
        kind: "std-module-call",
        qualified: "std::time::Duration::from_secs",
        suggested: "time::Duration::from_secs()",
      }),
    );
  });
}

export function registerQualityShapeCase8({ it, expect }) {
  it("does not report overqualified paths in Rust use declarations", () => {
    const result = analyzeSources([
      {
        path: "crates/example/src/lib.rs",
        language: "rust",
        source: `
use crate::MemoryCache;
pub use crate::ProviderSettings;
pub(crate) use crate::support::default;
`,
      },
    ]);

    expect(result.overqualifiedPaths).toEqual([]);
  });
}
