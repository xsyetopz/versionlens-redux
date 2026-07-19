import { resolve } from "node:path";
import { expect, it } from "../runtime.ts";

interface ManifestSupportMatrix {
  entries: MatrixEntry[];
}

interface MatrixEntry {
  dependencyProperties?: {
    default: string[];
    settingKey: string;
  };
  ecosystem: string;
  provider: string;
  registry: string;
  manifests: MatrixManifest[];
}

interface MatrixManifest {
  defaultGlob: string;
  defaultSettingKey: string;
  fileForms: string[];
  languages: string[];
  manifestKinds: string[];
  parsers: string[];
}

async function readMatrix(): Promise<ManifestSupportMatrix> {
  return (await Bun.file(
    resolve("tests/fixtures/manifest-support-matrix.json"),
  ).json()) as ManifestSupportMatrix;
}

const readmeLabelGroups: Record<string, string>[] = [
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

async function readReadme(): Promise<string> {
  return await Bun.file(resolve("packages/vscode-extension/README.md")).text();
}

async function readPackageJson(): Promise<{
  contributes: {
    configuration: { properties: Record<string, { default?: unknown }> };
  };
}> {
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

it("readme documents every manifest support matrix provider", async (): Promise<void> => {
  const matrix = await readMatrix();
  const packageJson = await readPackageJson();
  const {
    configuration: { properties },
  } = packageJson.contributes;
  const readme = await readReadme();

  for (const entry of matrix.entries) {
    const label = readmeLabels[entry.provider];
    expect(label).toBeString();
    if (!label) {
      throw new Error(`Missing README label for ${entry.provider}`);
    }
    expect(readme).toContain(label);
    const apiUrlSettings = Object.keys(properties).filter(
      (setting): boolean => setting === `versionlens.${entry.provider}.apiUrl`,
    );
    for (const apiUrlSetting of apiUrlSettings) {
      expect(readme).toContain(apiUrlSetting);
    }
    for (const manifest of entry.manifests) {
      for (const fileForm of manifest.fileForms.filter(
        (candidateFileForm): boolean => !candidateFileForm.includes("*"),
      )) {
        expect(readme).toContain(fileForm);
      }
    }
  }
});

it("manifest support matrix covers supported file defaults", async (): Promise<void> => {
  const matrix = await readMatrix();
  const packageJson = await readPackageJson();
  const {
    configuration: { properties },
  } = packageJson.contributes;
  const matrixBySetting = new Map(
    matrix.entries.flatMap((entry): [string, MatrixManifest][] =>
      entry.manifests.map((manifest): [string, MatrixManifest] => [
        manifest.defaultSettingKey,
        manifest,
      ]),
    ),
  );
  const fileSettingKeys = Object.keys(properties).filter((key): boolean =>
    key.endsWith(".files"),
  );

  const dependencyPropertyKeys = Object.keys(properties).filter(
    (key): boolean => key.endsWith(".dependencyProperties"),
  );
  const matrixDependencyProperties = new Map(
    matrix.entries.flatMap(
      ({ dependencyProperties }): [readonly [string, string[]]] | [] => {
        if (!dependencyProperties) {
          return [];
        }
        return [
          [
            dependencyProperties.settingKey,
            dependencyProperties.default,
          ] as const,
        ];
      },
    ),
  );

  expect(matrix.entries.length).toBeGreaterThan(0);
  expect([...matrixBySetting.keys()].sort()).toEqual(fileSettingKeys.sort());
  expect([...matrixDependencyProperties.keys()].sort()).toEqual(
    dependencyPropertyKeys.sort(),
  );

  for (const [dependencySettingKey, defaults] of matrixDependencyProperties) {
    expect(properties[dependencySettingKey]?.default).toEqual(defaults);
    expect(defaults.length).toBeGreaterThan(0);
  }

  for (const [manifestSettingKey, manifest] of matrixBySetting) {
    expect(properties[manifestSettingKey]?.default).toBe(manifest.defaultGlob);
    expect(manifest.fileForms.length).toBeGreaterThan(0);
    expect(manifest.manifestKinds.length).toBeGreaterThan(0);
    expect(manifest.parsers.length).toBeGreaterThan(0);
    expect(manifest.languages.length).toBeGreaterThan(0);
  }
});

it("manifest support matrix documents pubspec override route", async (): Promise<void> => {
  const matrix = await readMatrix();
  const pub = matrix.entries.find((entry): boolean => entry.provider === "pub");
  const manifest = pub?.manifests.find(
    (candidateManifest): boolean =>
      candidateManifest.defaultSettingKey === "versionlens.pub.files",
  );

  expect(manifest?.fileForms).toContain(
    ["pubspec", "_overrides.yaml"].join(""),
  );
  expect(manifest?.manifestKinds).toContain(
    ["Pubspec", "OverridesYaml"].join(""),
  );
  expect(manifest?.parsers).toContain(
    ["parse_pubspec", "_overrides_yaml_with_paths"].join(""),
  );
});

it("manifest support matrix documents jsr config route", async (): Promise<void> => {
  const matrix = await readMatrix();
  const deno = matrix.entries.find(
    (entry): boolean => entry.provider === "deno",
  );
  const manifest = deno?.manifests.find(
    (candidateManifest): boolean =>
      candidateManifest.defaultSettingKey === "versionlens.deno.files",
  );

  expect(manifest?.fileForms).toContain("jsr.json");
  expect(manifest?.fileForms).toContain("jsr.jsonc");
  expect(manifest?.manifestKinds).toContain("JsrJson");
  expect(manifest?.parsers).toContain("parse_jsr_json_with_paths");
});

it("manifest support matrix documents dotnet central package route", async (): Promise<void> => {
  const matrix = await readMatrix();
  const dotnet = matrix.entries.find(
    (entry): boolean => entry.provider === "dotnet",
  );
  const manifest = dotnet?.manifests.find(
    (candidateManifest): boolean =>
      candidateManifest.defaultSettingKey === "versionlens.dotnet.files",
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
