import type { ConfigKeyPair } from "../key-pair.ts";

export const providerStrictSslKeys: readonly ConfigKeyPair[] = [
	["cargo", "cargo.http.strictSSL"],
	["composer", "composer.http.strictSSL"],
	["deno", "deno.http.strictSSL"],
	["docker", "docker.http.strictSSL"],
	["dotnet", "dotnet.http.strictSSL"],
	["dub", "dub.http.strictSSL"],
	["golang", "golang.http.strictSSL"],
	["maven", "maven.http.strictSSL"],
	["pub", "pub.http.strictSSL"],
	["pypi", "pypi.http.strictSSL"],
	["ruby", "ruby.http.strictSSL"],
] as const;
