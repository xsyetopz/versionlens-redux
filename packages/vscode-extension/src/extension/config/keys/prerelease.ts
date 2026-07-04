import type { ConfigKeyPair } from "../key-pair.ts";

export const prereleaseTagKeys: readonly ConfigKeyPair[] = [
	["cargo", "cargo.prereleaseTagFilter"],
	["composer", "composer.prereleaseTagFilter"],
	["deno", "deno.prereleaseTagFilter"],
	["dotnet", "dotnet.prereleaseTagFilter"],
	["dub", "dub.prereleaseTagFilter"],
	["golang", "golang.prereleaseTagFilter"],
	["maven", "maven.prereleaseTagFilter"],
	["npm", "npm.prereleaseTagFilter"],
	["pypi", "pypi.prereleaseTagFilter"],
	["pub", "pub.prereleaseTagFilter"],
	["ruby", "ruby.prereleaseTagFilter"],
] as const;
