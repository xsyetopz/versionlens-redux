import { expect, test } from "bun:test";

function native() {
	return require("../../packages/vscode-extension/native/versionlens_napi.node");
}

function createSession(options = {}) {
	return native().createSession({
		http: { proxy: "", strictSsl: true, timeoutMs: 10_000 },
		showPrereleases: false,
		...options,
	});
}

test("applyCommand sorts requirements dependencies", () => {
	const session = createSession();

	const output = session.applyCommand({
		command: "sort",
		document: {
			languageId: "pip-requirements",
			text: "zeta==1\n# keep\nalpha==1\n",
			uri: "file:///requirements.txt",
		},
	});

	expect(output.edits).toHaveLength(2);
	expect(output.edits[0]?.newText).toBe("alpha==1");
	expect(output.edits[1]?.newText).toBe("zeta==1");
});

test("applyCommand updates project version", () => {
	const session = createSession();

	const output = session.applyCommand({
		command: "updateMajor",
		dependencyName: "1.2.3",
		document: {
			languageId: "json",
			text: '{"version":"1.2.3","dependencies":{"left-pad":"1.0.0"}}',
			uri: "file:///package.json",
		},
	});

	expect(output.edits).toHaveLength(1);
	expect(output.edits[0]?.newText).toBe("2.0.0");
});

test("resolveDocument is callable without registry work", async () => {
	const session = createSession();

	const output = await session.resolveDocument({
		languageId: "json",
		text: '{"dependencies":{"local":"workspace:*"}}',
		uri: "file:///package.json",
	});

	expect(output.edits).toHaveLength(0);
	expect(output.suggestions).toHaveLength(0);
});

test("clearCache and disposeSession are callable", () => {
	const session = createSession();

	session.clearCache();
	session.disposeSession();
});

test("disposeSession releases the native Rust session", async () => {
	const session = createSession();
	const input = {
		languageId: "json",
		text: '{"dependencies":{"left-pad":"1.0.0"}}',
		uri: "file:///package.json",
	};

	expect(session.analyzeDocument(input).isSupportedManifest).toBe(true);

	session.disposeSession();
	session.clearCache();

	const analyzed = session.analyzeDocument(input);
	expect(analyzed.isSupportedManifest).toBe(false);
	expect(analyzed.status.visible).toBe(false);
	expect((await session.resolveDocument(input)).edits).toHaveLength(0);
	expect(
		session.applyCommand({ command: "updateMajor", document: input }).edits,
	).toHaveLength(0);
});

test("analyzeDocument can disable vulnerability diagnostics", () => {
	const session = createSession({ showVulnerabilities: false });

	const output = session.analyzeDocument({
		languageId: "json",
		text: '{"dependencies":{"left-pad":"1.0.0"}}',
		uri: "file:///package.json",
	});

	expect(output.diagnostics).toHaveLength(0);
	expect(output.dependencies).toHaveLength(1);
	expect(output.dependencies[0]).toMatchObject({
		ecosystem: "npm",
		group: "dependencies",
		name: "left-pad",
		requirement: "1.0.0",
	});
	expect(output.status.vulnerabilityCount).toBe(0);
});

test("analyzeDocument omits native missing-suggestion code lens payloads", () => {
	const session = createSession({
		suggestionIndicators: { updateable: "U" },
	});

	const output = session.analyzeDocument({
		languageId: "json",
		text: '{"dependencies":{"left-pad":"1.0.0"}}',
		uri: "file:///package.json",
	});

	expect(output.codeLenses).toHaveLength(0);
});

test("analyzeDocument omits schema diagnostics across N-API", () => {
	const session = createSession();

	const output = session.analyzeDocument({
		languageId: "json",
		text: '{"npm":{"url":"not a url"}}',
		uri: "versionlens:/versionlens.multi-registries.json",
	});

	expect(output.isSupportedManifest).toBe(true);
	expect(output.diagnostics).toHaveLength(0);
});
