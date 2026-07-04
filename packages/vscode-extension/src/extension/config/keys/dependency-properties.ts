import type { ConfigKeyPair } from "../key-pair.ts";

export const dependencyPropertyKeys: readonly ConfigKeyPair[] = [
	["cargo", "cargo.dependencyProperties"],
	["composer", "composer.dependencyProperties"],
	["deno", "deno.dependencyProperties"],
	["dotnet", "dotnet.dependencyProperties"],
	["dub", "dub.dependencyProperties"],
	["maven", "maven.dependencyProperties"],
	["npm", "npm.dependencyProperties"],
	["pnpm", "pnpm.dependencyProperties"],
	["pub", "pub.dependencyProperties"],
	["pypi", "pypi.dependencyProperties"],
] as const;
