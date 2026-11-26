import { describe, expect, test } from "bun:test";

import { globRegex, matchesAny } from "./globs.ts";

describe("globRegex", () => {
	test("matches whole trailing segments regardless of case", () => {
		// Arrange
		const regexes = ["Dockerfile", "requirements/*.txt"].map((glob) =>
			globRegex(glob)
		);

		// Act
		const matched = [
			"/repo/dockerfile",
			String.raw`C:\repo\Requirements\dev.TXT`,
			"/repo/my-Dockerfile",
			"/repo/requirements/sub/dev.txt"
		].map((path) => matchesAny(regexes, path));

		// Assert
		expect(matched).toEqual([true, true, false, false]);
	});

	test("lets ** span zero or more directories", () => {
		// Arrange
		const regexes = [globRegex(".github/workflows/**/*.yml")];

		// Act
		const matched = [
			"/repo/.github/workflows/ci.yml",
			"/repo/.github/workflows/a/b/ci.yml",
			"/repo/workflows/ci.yml"
		].map((path) => matchesAny(regexes, path));

		// Assert
		expect(matched).toEqual([true, true, false]);
	});
});
