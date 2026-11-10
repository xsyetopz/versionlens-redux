import { expect, it } from "bun:test";

import { field, items, manifestVersion, parseJson } from "../lib/json.mjs";
import { capture } from "../lib/spawn.mjs";

it("release check derives and validates every root Cargo workspace package", async () => {
	const metadata = await capture([
		"cargo",
		"metadata",
		"--no-deps",
		"--format-version",
		"1",
	]);
	expect(metadata.exitCode).toBe(0);
	const workspace = parseJson(metadata.stdout);
	const version = manifestVersion(
		parseJson(await Bun.file("package.json").text()),
	);
	const result = await capture([
		"bun",
		"scripts/release/version.mjs",
		version,
		"--check",
	]);
	expect(result.exitCode).toBe(0);
	const names = items(field(workspace, "packages")).map((entry) =>
		field(entry, "name"),
	);
	expect(names).toContain("versionlens-test-support");
});
