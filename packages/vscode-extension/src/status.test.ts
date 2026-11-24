import { expect, test } from "bun:test";

import { parseStored, withAuthorization } from "./authorization.ts";
import { contextKeys, parseStatus } from "./status.ts";

test("a status payload sets the menu context keys", () => {
	// Arrange
	const status = parseStatus({
		busy: false,
		enabled: true,
		failed: 0,
		install: "bun install",
		prereleases: false,
		sortable: true,
		uri: "file:///p/package.json"
	});

	// Act
	const keys = contextKeys(status);

	// Assert
	expect(keys).toEqual({
		"versionlens.providerActive": true,
		"versionlens.providerBusy": false,
		"versionlens.providerError": false,
		"versionlens.show": true,
		"versionlens.showCustomInstall": true,
		"versionlens.showPrereleases": false,
		"versionlens.showSortAlphabetically": true
	});
});

test("no status turns every menu entry off, and a payload without uri is no status", () => {
	expect(parseStatus({ enabled: true })).toBeUndefined();
	expect(Object.values(contextKeys()).some(Boolean)).toBe(false);
});

test("failed lookups switch the title icon to the error icon", () => {
	const status = parseStatus({ failed: 2, uri: "file:///p/Cargo.toml" });
	expect(contextKeys(status)["versionlens.providerError"]).toBe(true);
});

test("stored headers join the configured ones and win for the same prefix", () => {
	// Arrange
	const section = {
		authorization: { "https/a/": "Bearer settings", "https/b/": "Bearer kept" },
		timeoutMs: 5000
	};

	// Act
	const merged = withAuthorization(section, { "https/a/": "Bearer secret" });

	// Assert
	expect(merged).toEqual({
		authorization: { "https/a/": "Bearer secret", "https/b/": "Bearer kept" },
		timeoutMs: 5000
	});
});

test("stored headers parse from secret storage text, and bad text is no headers", () => {
	expect(parseStored('{"https/a/":"Basic x","n":1}')).toEqual({
		"https/a/": "Basic x"
	});
	expect(parseStored("not json")).toEqual({});
	expect(parseStored()).toEqual({});
	expect(withAuthorization({ timeoutMs: 1 }, {})).toEqual({ timeoutMs: 1 });
});
