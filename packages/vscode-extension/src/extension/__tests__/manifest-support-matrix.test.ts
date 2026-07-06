import { expect, test } from "bun:test";
import { resolve } from "node:path";

type ManifestSupportMatrix = {
	entries: MatrixEntry[];
};

type MatrixEntry = {
	dependencyProperties?: {
		default: string[];
		settingKey: string;
	};
	ecosystem: string;
	provider: string;
	registry: string;
	manifests: MatrixManifest[];
};

type MatrixManifest = {
	defaultGlob: string;
	defaultSettingKey: string;
	fileForms: string[];
	languages: string[];
	manifestKinds: string[];
	parsers: string[];
};

async function readMatrix() {
	return (await Bun.file(
		resolve("tests/fixtures/manifest-support-matrix.json"),
	).json()) as ManifestSupportMatrix;
}

const readmeLabelGroups = [
	{
		ansible: "Ansible",
		bazel: "Bazel",
		cargo: "Cargo",
		cocoapods: "CocoaPods",
		composer: "Composer",
		conan: "Conan",
		cpp: "C/C++ build files",
		cpan: "CPAN",
		cran: "CRAN",
		deno: "Deno",
		docker: "Docker",
		dotnet: ".NET",
		dub: "Dub",
		golang: "Go",
		hackage: "Haskell",
		haxelib: "Haxelib",
		helm: "Helm",
		hex: "Hex",
	},
	{
		julia: "Julia",
		kustomize: "Kustomize",
		luarocks: "LuaRocks",
		maven: "Maven",
		nim: "Nim",
		nix: "Nix",
		npm: "npm",
		opam: "opam",
		pnpm: "pnpm",
		pub: "Pub",
		pypi: "Python",
		ruby: "Ruby",
		swift: "Swift",
		terraform: "Terraform",
		unity: "Unity",
		vcpkg: "vcpkg",
		zig: "Zig",
	},
] satisfies Record<string, string>[];

const readmeLabels: Record<string, string> = Object.assign(
	{},
	...readmeLabelGroups,
);

async function readReadme() {
	return await Bun.file(resolve("packages/vscode-extension/README.md")).text();
}

async function readPackageJson() {
	return (await Bun.file(
		resolve("packages/vscode-extension/package.json"),
	).json()) as {
		contributes: {
			configuration: {
				properties: Record<string, { default?: unknown }>;
			};
		};
	};
}

test("readme documents every manifest support matrix provider", async () => {
	const matrix = await readMatrix();
	const packageJson = await readPackageJson();
	const properties = packageJson.contributes.configuration.properties;
	const readme = await readReadme();

	for (const entry of matrix.entries) {
		const label = readmeLabels[entry.provider];
		expect(label).toBeString();
		const providerLabel = label ?? "";
		expect(readme).toContain(providerLabel);
		const apiUrlSetting = `versionlens.${entry.provider}.apiUrl`;
		if (apiUrlSetting in properties) {
			expect(readme).toContain(apiUrlSetting);
		}
		for (const manifest of entry.manifests) {
			for (const fileForm of manifest.fileForms.filter(
				(fileForm) => !fileForm.includes("*"),
			)) {
				expect(readme).toContain(fileForm);
			}
		}
	}
});

test("manifest support matrix covers supported file defaults", async () => {
	const matrix = await readMatrix();
	const packageJson = await readPackageJson();
	const properties = packageJson.contributes.configuration.properties;
	const matrixBySetting = new Map(
		matrix.entries.flatMap((entry) =>
			entry.manifests.map((manifest) => [manifest.defaultSettingKey, manifest]),
		),
	);
	const fileSettingKeys = Object.keys(properties).filter((key) =>
		key.endsWith(".files"),
	);

	const dependencyPropertyKeys = Object.keys(properties).filter((key) =>
		key.endsWith(".dependencyProperties"),
	);
	const matrixDependencyProperties = new Map(
		matrix.entries.flatMap((entry) =>
			entry.dependencyProperties
				? [
						[
							entry.dependencyProperties.settingKey,
							entry.dependencyProperties.default,
						] as const,
					]
				: [],
		),
	);

	expect(matrix.entries.length).toBeGreaterThan(0);
	expect([...matrixBySetting.keys()].sort()).toEqual(fileSettingKeys.sort());
	expect([...matrixDependencyProperties.keys()].sort()).toEqual(
		dependencyPropertyKeys.sort(),
	);

	for (const [settingKey, defaults] of matrixDependencyProperties) {
		expect(properties[settingKey]?.default).toEqual(defaults);
		expect(defaults.length).toBeGreaterThan(0);
	}

	for (const [settingKey, manifest] of matrixBySetting) {
		expect(properties[settingKey]?.default).toBe(manifest.defaultGlob);
		expect(manifest.fileForms.length).toBeGreaterThan(0);
		expect(manifest.manifestKinds.length).toBeGreaterThan(0);
		expect(manifest.parsers.length).toBeGreaterThan(0);
		expect(manifest.languages.length).toBeGreaterThan(0);
	}
});

test("manifest support matrix documents pubspec override route", async () => {
	const matrix = await readMatrix();
	const pub = matrix.entries.find((entry) => entry.provider === "pub");
	const manifest = pub?.manifests.find(
		(manifest) => manifest.defaultSettingKey === "versionlens.pub.files",
	);

	expect(manifest?.fileForms).toContain("pubspec_overrides.yaml");
	expect(manifest?.manifestKinds).toContain("PubspecOverridesYaml");
	expect(manifest?.parsers).toContain(
		"parse_pubspec_overrides_yaml_with_paths",
	);
});

test("manifest support matrix documents jsr config route", async () => {
	const matrix = await readMatrix();
	const deno = matrix.entries.find((entry) => entry.provider === "deno");
	const manifest = deno?.manifests.find(
		(manifest) => manifest.defaultSettingKey === "versionlens.deno.files",
	);

	expect(manifest?.fileForms).toContain("jsr.json");
	expect(manifest?.fileForms).toContain("jsr.jsonc");
	expect(manifest?.manifestKinds).toContain("JsrJson");
	expect(manifest?.parsers).toContain("parse_jsr_json_with_paths");
});

test("manifest support matrix documents dotnet central package route", async () => {
	const matrix = await readMatrix();
	const dotnet = matrix.entries.find((entry) => entry.provider === "dotnet");
	const manifest = dotnet?.manifests.find(
		(manifest) => manifest.defaultSettingKey === "versionlens.dotnet.files",
	);

	expect(manifest?.fileForms).toContain("Directory.Packages.props");
	expect(manifest?.manifestKinds).toContain("DotnetXml");
	expect(manifest?.parsers).toContain("parse_dotnet_xml_with_paths");
	expect(dotnet?.dependencyProperties?.default).toContain(
		"Project.ItemGroup.PackageVersion",
	);
	expect(dotnet?.dependencyProperties?.default).toContain(
		"Project.ItemGroup.GlobalPackageReference",
	);
});
