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
	["dotnet", "dotnet.files", ["xml", "json", "jsonc"], ["**/obj/**"]],
	["dub", "dub.files", ["json", "jsonc"]],
	["golang", "golang.files", ["go.mod"]],
	["maven", "maven.files", ["xml"]],
	["npm", "npm.files", ["json", "jsonc"]],
	["pnpm", "pnpm.files", ["yaml"]],
	["pypi", "pypi.files", ["toml", "pip-requirements", "plaintext"]],
	["pub", "pub.files", ["yaml"]],
	["ruby", "ruby.files", ["ruby", "plaintext"]],
] as const;

export function enabledFilePatternKeys(
	enabledProviders: readonly string[] | undefined,
) {
	if (!enabledProviders || enabledProviders.length === 0) {
		return filePatternKeys;
	}

	const enabled = new Set(enabledProviders);
	return filePatternKeys.filter(([ecosystem]) => enabled.has(ecosystem));
}
