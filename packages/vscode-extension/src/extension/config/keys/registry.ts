import type { ConfigKeyPair } from "../key-pair.ts";

export const registryUrlKeys: readonly ConfigKeyPair[] = [
	["cargo", "cargo.apiUrl"],
	["composer", "composer.apiUrl"],
	["dub", "dub.apiUrl"],
	["golang", "golang.apiUrl"],
	["maven", "maven.apiUrl"],
	["pypi", "pypi.apiUrl"],
	["pub", "pub.apiUrl"],
	["ruby", "ruby.apiUrl"],
] as const;
