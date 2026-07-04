import type { ConfigKeyPair } from "../key-pair.ts";

export const providerCacheKeys: readonly ConfigKeyPair[] = [
	["cargo", "cargo.caching.duration"],
	["composer", "composer.caching.duration"],
	["deno", "deno.caching.duration"],
	["docker", "docker.caching.duration"],
	["dotnet", "dotnet.caching.duration"],
	["dub", "dub.caching.duration"],
	["golang", "golang.caching.duration"],
	["maven", "maven.caching.duration"],
	["npm", "npm.caching.duration"],
	["pub", "pub.caching.duration"],
	["pypi", "pypi.caching.duration"],
	["ruby", "ruby.caching.duration"],
] as const;
