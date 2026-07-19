type FilePatternKey = readonly [
  ecosystem: string,
  key: string,
  languages: readonly string[],
  excludePatterns?: readonly string[],
];

export const filePatternKeys: readonly FilePatternKey[] = [
  ["cargo", "cargo.files", ["toml"]],
  ["composer", "composer.files", ["json", "jsonc"]],
  ["deno", "deno.files", ["json", "jsonc"]],
  ["docker", "docker.files", ["dockerfile", "dockercompose", "yaml"]],
  [
    "dotnet",
    "dotnet.files",
    ["xml", "json", "jsonc", "plaintext"],
    ["**/obj/**"],
  ],
  ["dub", "dub.files", ["json", "jsonc", "plaintext"]],
  ["golang", "golang.files", ["go.mod"]],
  [
    "maven",
    "maven.files",
    ["xml", "groovy", "kotlin", "toml", "scala", "clojure"],
  ],
  ["npm", "npm.files", ["json", "jsonc", "json5", "yaml"]],
  ["pnpm", "pnpm.files", ["yaml"]],
  ["pypi", "pypi.files", ["toml", "pip-requirements", "plaintext"]],
  ["pub", "pub.files", ["yaml"]],
  ["ruby", "ruby.files", ["ruby", "plaintext"]],
  ["hex", "hex.files", ["elixir", "erlang", "toml", "plaintext"]],
  ["opam", "opam.files", ["plaintext"]],
  ["hackage", "hackage.files", ["plaintext", "yaml"]],
  ["julia", "julia.files", ["toml"]],
  ["cran", "cran.files", ["plaintext", "json"]],
  ["conan", "conan.files", ["plaintext", "python"]],
  ["vcpkg", "vcpkg.files", ["json", "jsonc"]],
  ["swift", "swift.files", ["swift"]],
  ["zig", "zig.files", ["zig"]],
  ["nim", "nim.files", ["nim", "plaintext"]],
  ["luarocks", "luarocks.files", ["lua"]],
  ["cpan", "cpan.files", ["perl", "plaintext"]],
  ["haxelib", "haxelib.files", ["json", "jsonc"]],
  ["terraform", "terraform.files", ["terraform", "hcl", "plaintext"]],
  ["helm", "helm.files", ["yaml"]],
  ["ansible", "ansible.files", ["yaml"]],
  ["bazel", "bazel.files", ["starlark", "plaintext"]],
  ["nix", "nix.files", ["nix"]],
  ["kustomize", "kustomize.files", ["yaml"]],
  ["unity", "unity.files", ["json", "jsonc"]],
  ["cocoapods", "cocoapods.files", ["ruby", "plaintext"]],
  [
    "cpp",
    "cpp.files",
    ["cmake", "lua", "meson", "ini", "starlark", "plaintext"],
  ],
] as const;

export function enabledFilePatternKeys(
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
