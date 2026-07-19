export const rustConfig = `
struct BuildRequest {
	pub package_name: String,
	pub package_version: String,
	pub registry_url: String,
	pub source_path: String,
	pub target_path: String,
	pub checksum: String,
	pub cache_key: String,
	pub retry_count: usize,
	pub timeout_ms: u64,
	pub user_agent: String,
	pub auth_token: Option<String>,
}

fn fetch_one(package_name: String, versions: Vec<ResolvedDependency>) -> Result<Vec<ResolvedDependency>, FetchError> {
	provider.fetch(package_name, versions)
}

fn fetch_two(package_name: String, versions: Vec<ResolvedDependency>) -> Result<Vec<ResolvedDependency>, FetchError> {
	provider.fetch(package_name, versions)
}

fn pass_through(package_name: String) -> Result<String, FetchError> {
	fetch_name(package_name)
}

fn suppressed(_package_name: String, version: String) -> String {
	version
}
`;

export const typeScriptConfig = `
interface ExtensionViewModel {
	name: string;
	version: string;
	ecosystem: string;
	current: string;
	latest: string;
	stable: string;
	wanted: string;
	range: string;
	registry: string;
	filePath: string;
	line: number;
}

function renderOne(packageName: string, versions: Array<ResolvedDependency>): Promise<Array<ResolvedDependency>> {
	return renderer.render(packageName, versions);
}

function renderTwo(packageName: string, versions: Array<ResolvedDependency>): Promise<Array<ResolvedDependency>> {
	return renderer.render(packageName, versions);
}

function adapter(packageName: string): string {
	return renderName(packageName);
}

function ignored(_packageName: string, version: string): string {
	return version;
}
`;

export const qualityFindings = [
  [
    "duplicateLogic",
    {
      firstPath: "crates/example/src/lib.rs",
      secondPath: "crates/example/src/lib.rs",
      firstName: "fetch_one",
      secondName: "fetch_two",
    },
  ],
  ["repeatedComplexTypes", { typeText: "Vec<ResolvedDependency>", count: 2 }],
  [
    "repeatedComplexTypes",
    { typeText: ["Array", "<ResolvedDependency>"].join(""), count: 2 },
  ],
  [
    "oversizedShapes",
    { path: "crates/example/src/lib.rs", name: "BuildRequest", fieldCount: 11 },
  ],
  [
    "oversizedShapes",
    {
      path: "packages/vscode-extension/src/example.ts",
      name: "ExtensionViewModel",
      fieldCount: 11,
    },
  ],
  [
    "suppressedParameters",
    {
      path: "crates/example/src/lib.rs",
      functionName: "suppressed",
      parameterName: "_package_name",
    },
  ],
  [
    "suppressedParameters",
    {
      path: "packages/vscode-extension/src/example.ts",
      functionName: "ignored",
      parameterName: "_packageName",
    },
  ],
  [
    "passThroughWrappers",
    {
      path: "crates/example/src/lib.rs",
      name: "pass_through",
      callee: "fetch_name",
    },
  ],
  [
    "passThroughWrappers",
    {
      path: "packages/vscode-extension/src/example.ts",
      name: "adapter",
      callee: "renderName",
    },
  ],
];
