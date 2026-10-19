type FilePatternKey = readonly [
  ecosystem: string,
  key: string,
  languages: readonly string[],
  excludePatterns?: readonly string[],
];

const filePatternKeys: readonly FilePatternKey[] = [
  ["cargo", "cargo.files", ["plaintext", "toml"]],
  ["composer", "composer.files", ["json", "jsonc"]],
  ["deno", "deno.files", ["json", "jsonc"]],
  [
    "docker",
    "docker.files",
    ["dockercompose", "yaml", "dockerfile", "plaintext"],
  ],
  [
    "dotnet",
    "dotnet.files",
    ["xml", "plaintext", "json", "jsonc"],
    ["**/obj/**"],
  ],
  ["dub", "dub.files", ["json", "jsonc", "plaintext"]],
  ["golang", "golang.files", ["go.mod", "plaintext"]],
  [
    "maven",
    "maven.files",
    ["toml", "groovy", "kotlin", "plaintext", "scala", "clojure", "xml"],
  ],
  ["npm", "npm.files", ["plaintext", "json", "jsonc", "json5", "yaml"]],
  ["pnpm", "pnpm.files", ["yaml"]],
  ["pypi", "pypi.files", ["plaintext", "pip-requirements", "toml"]],
  ["pub", "pub.files", ["yaml"]],
  ["ruby", "ruby.files", ["ruby", "plaintext"]],
  ["hex", "hex.files", ["elixir", "plaintext", "erlang", "toml"]],
  ["opam", "opam.files", ["plaintext"]],
  ["hackage", "hackage.files", ["properties", "plaintext", "yaml"]],
  ["julia", "julia.files", ["toml"]],
  ["cran", "cran.files", ["plaintext", "json", "jsonc"]],
  ["conan", "conan.files", ["plaintext", "python"]],
  ["vcpkg", "vcpkg.files", ["json", "jsonc"]],
  ["swift", "swift.files", ["swift", "plaintext"]],
  ["zig", "zig.files", ["zig", "plaintext"]],
  ["nim", "nim.files", ["nim", "plaintext"]],
  ["luarocks", "luarocks.files", ["lua", "plaintext"]],
  ["cpan", "cpan.files", ["perl", "plaintext"]],
  ["haxelib", "haxelib.files", ["json", "jsonc"]],
  ["terraform", "terraform.files", ["terraform", "hcl", "plaintext"]],
  ["helm", "helm.files", ["yaml"]],
  ["ansible", "ansible.files", ["yaml"]],
  ["bazel", "bazel.files", ["starlark", "plaintext"]],
  ["nix", "nix.files", ["nix", "plaintext"]],
  ["kustomize", "kustomize.files", ["yaml"]],
  ["unity", "unity.files", ["json", "jsonc"]],
  ["cocoapods", "cocoapods.files", ["ruby", "plaintext"]],
  [
    "cpp",
    "cpp.files",
    ["cmake", "plaintext", "lua", "meson", "ini", "starlark"],
  ],
  ["github", "github.files", ["yaml"]],
] as const;

function enabledFilePatternKeys(
  enabledProviders: readonly string[] | undefined,
): readonly FilePatternKey[] {
  if (!enabledProviders || enabledProviders.length === 0) {
    return filePatternKeys;
  }

  const enabled = new Set(enabledProviders);
  return filePatternKeys.filter(([ecosystem]): boolean =>
    enabled.has(ecosystem),
  );
}

export { enabledFilePatternKeys, filePatternKeys };
