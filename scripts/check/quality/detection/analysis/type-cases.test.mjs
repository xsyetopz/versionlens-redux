import { analyzeSources } from "../analysis.mjs";
import { qualityFindings, rustConfig, typeScriptConfig } from "./fixtures.mjs";

export function registerQualityShapeCase({ it, expect }) {
  it("reports requested repository quality shapes", () => {
    const result = analyzeSources([
      {
        path: "crates/example/src/lib.rs",
        language: "rust",
        source: rustConfig,
      },
      {
        path: "packages/vscode-extension/src/example.ts",
        language: "typescript",
        source: typeScriptConfig,
      },
    ]);

    for (const [key, expected] of qualityFindings) {
      expect(result[key]).toContainEqual(expect.objectContaining(expected));
    }
  });
}

export function registerMultilineParameterCase({ it, expect }) {
  it("parses multiline Rust parameters without body spillover", () => {
    const result = analyzeSources([
      {
        path: "crates/example/src/multiline.rs",
        language: "rust",
        source: `
fn collect_value(
    context: &JsonManifestContext<'_>,
    parents: &[&str],
    out: &mut Vec<Dependency>,
) {
    use_value(context, parents, out);
}

fn collect_other(
    context: &JsonManifestContext<'_>,
    parents: &[&str],
    out: &mut Vec<Dependency>,
) {
    use_value(context, parents, out);
}
`,
      },
    ]);

    expect(result.repeatedComplexTypes).toContainEqual(
      expect.objectContaining({
        typeText: "&mut Vec<Dependency>",
        count: 2,
      }),
    );
    expect(result.unusedParameters).toEqual([]);
    expect(result.suppressedParameters).toEqual([]);
  });
}

export function registerLocalAliasCase({ it, expect }) {
  it("does not report repeated aliased complex type usages", () => {
    const result = analyzeSources([
      {
        path: "crates/example/src/aliased.rs",
        language: "rust",
        source: `
type XmlEvent<'a> = BytesStart<'a>;

fn read_one(event: &XmlEvent<'_>) {
    read_event(event);
}

fn read_two(event: &XmlEvent<'_>) {
    read_event(event);
}
`,
      },
    ]);

    expect(result.repeatedComplexTypes).not.toContainEqual(
      expect.objectContaining({
        typeText: "&XmlEvent<lifetime>",
      }),
    );
  });
}

export function registerCrossFileAliasCase({ it, expect }) {
  it("does not report repeated aliased complex type usages declared in another file", () => {
    const result = analyzeSources([
      {
        path: "crates/example/src/request.rs",
        language: "rust",
        source: `
pub(super) type ResolutionRequest<'a> = Request<'a>;
`,
      },
      {
        path: "crates/example/src/parallel.rs",
        language: "rust",
        source: `
fn resolve_one(request: ResolutionRequest<'_>) {
    resolve(request);
}

fn resolve_two(request: ResolutionRequest<'_>) {
    resolve(request);
}
`,
      },
    ]);

    expect(result.repeatedComplexTypes).not.toContainEqual(
      expect.objectContaining({
        typeText: "ResolutionRequest<lifetime>",
      }),
    );
  });
}

export function registerLocalConcreteTypeCase({ it, expect }) {
  it("does not report repeated direct local concrete type references", () => {
    const result = analyzeSources([
      {
        path: "crates/example/src/context.rs",
        language: "rust",
        source: `
struct EventContext<'a> {
    value: &'a str,
}

fn read_one(context: &EventContext<'_>, values: Vec<EventContext<'_>>) {
    read_context(context, values);
}

fn read_two(context: &EventContext<'_>, values: Vec<EventContext<'_>>) {
    read_context(context, values);
}
`,
      },
    ]);

    expect(result.repeatedComplexTypes).not.toContainEqual(
      expect.objectContaining({
        typeText: "&EventContext<lifetime>",
      }),
    );
    expect(result.repeatedComplexTypes).toContainEqual(
      expect.objectContaining({
        typeText: ["Vec", "<EventContext<lifetime>>"].join(""),
        count: 2,
      }),
    );
    expect(result.repeatedComplexTypes).not.toContainEqual(
      expect.objectContaining({
        typeText: "EventContext<lifetime>",
      }),
    );
  });
}

export function registerCrossFileConcreteTypeCase({ it, expect }) {
  it("does not report direct concrete type references declared in another file", () => {
    const result = analyzeSources([
      {
        path: "crates/example/src/context.rs",
        language: "rust",
        source: `
pub(crate) struct SharedContext<'a> {
    value: &'a str,
}
`,
      },
      {
        path: "crates/example/src/state.rs",
        language: "rust",
        source: `
fn read_one(context: &SharedContext<'_>) {
    read_context(context);
}

fn read_two(context: &SharedContext<'_>) {
    read_context(context);
}
`,
      },
    ]);

    expect(result.repeatedComplexTypes).not.toContainEqual(
      expect.objectContaining({
        typeText: ["&SharedContext", "<lifetime>"].join(""),
      }),
    );
  });
}

export function registerCommonWrapperCase({ it, expect }) {
  it("ignores low-signal simple Option and Result wrappers in repository mode", () => {
    const result = analyzeSources(
      [
        {
          path: "crates/example/src/simple.rs",
          language: "rust",
          source: `
fn one(value: &str) -> Option<ManifestKind> {
    classify(value)
}

fn two(value: &str) -> Option<ManifestKind> {
    classify(value)
}

fn agent_one() -> Result<Agent, HttpError> {
    build_agent()
}

fn agent_two() -> Result<Agent, HttpError> {
    build_agent()
}

fn complex_one() -> Option<Vec<ManifestKind>> {
    classify_all()
}

fn complex_two() -> Option<Vec<ManifestKind>> {
    classify_all()
}
`,
        },
      ],
      { ignoreCommonComplexTypes: true },
    );

    expect(result.repeatedComplexTypes).not.toContainEqual(
      expect.objectContaining({
        typeText: ["Option", "<ManifestKind>"].join(""),
      }),
    );
    expect(result.repeatedComplexTypes).not.toContainEqual(
      expect.objectContaining({
        typeText: ["Result", "<Agent,HttpError>"].join(""),
      }),
    );
    expect(result.repeatedComplexTypes).toContainEqual(
      expect.objectContaining({
        typeText: ["Option", `<${["Vec", "<ManifestKind>"].join("")}>`].join(
          "",
        ),
        count: 2,
      }),
    );
  });
}

export function registerFormatCaptureCase({ it, expect }) {
  it("treats Rust format string captures as parameter usage", () => {
    const result = analyzeSources([
      {
        path: "crates/example/src/format.rs",
        language: "rust",
        source: `
fn cache_key(provider: &str, package: &str) -> String {
    format!("{provider}:{package}")
}
`,
      },
    ]);

    expect(result.unusedParameters).toEqual([]);
  });
}

export function registerStringCommentCase({ it, expect }) {
  it("preserves string contents while stripping comments for usage checks", () => {
    const result = analyzeSources([
      {
        path: "crates/example/src/string_comments.rs",
        language: "rust",
        source: `
fn match_registry(entry: &Entry, url: &str) -> Option<usize> {
    auth_registry_match_len(entry.registry.strip_prefix("//")?, url)
}
`,
      },
    ]);

    expect(result.unusedParameters).toEqual([]);
  });
}
