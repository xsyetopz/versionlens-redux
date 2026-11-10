import { afterEach, beforeAll, describe, expect, it } from "bun:test";

import {
	buildServerBinary,
	diagnostics,
	hasStatus,
	MANIFEST,
	objectContaining,
	open,
	runCleanups,
	settled,
	startSession,
	stringContaining,
	stringMatching,
	URI,
} from "./support/session.ts";

// A cold debug build of the server takes minutes.
beforeAll(buildServerBinary, 600_000);

afterEach(runCleanups);

describe("versionlens-lsp as the VS Code client drives it", (): void => {
	it("reports busy then idle and answers code lenses from the registry", async (): Promise<void> => {
		const { connection } = await startSession();
		open(connection);

		await connection.waitFor("versionlens/status", (params): boolean =>
			hasStatus(params, "busy"),
		);
		await settled(connection);
		const lenses = await connection.request("textDocument/codeLens", {
			textDocument: { uri: URI },
		});

		expect(lenses).toContainEqual(
			objectContaining({
				command: objectContaining({
					title: stringContaining("7.3.5"),
				}),
			}),
		);
		expect(lenses).toContainEqual(
			objectContaining({
				command: objectContaining({
					arguments: [
						{
							dependency: stringMatching(/^astro/u),
							level: "latest",
							uri: URI,
							version: "7.3.5",
						},
					],
					command: "versionlens.server.update",
				}),
			}),
		);
	});

	it("publishes an update-available diagnostic on the requirement", async (): Promise<void> => {
		const { connection } = await startSession();
		open(connection);

		expect(await diagnostics(connection)).toEqual([
			{
				code: "update-available",
				message: "^7.2.2 → ^7.3.5",
				range: {
					end: { character: 32, line: 0 },
					start: { character: 26, line: 0 },
				},
				severity: 3,
				source: "VersionLens",
			},
		]);
	});

	it("offers a preferred quick fix whose documentChanges edit applies", async (): Promise<void> => {
		const { connection, registry } = await startSession();
		open(connection);
		const published = await diagnostics(connection);
		await settled(connection);
		const requestsBefore = registry.authorization.length;

		const cursor = { character: 28, line: 0 };
		const actions = await connection.request("textDocument/codeAction", {
			context: { diagnostics: published },
			range: { end: cursor, start: cursor },
			textDocument: { uri: URI },
		});

		// Applied to the manifest, this edit gives {"dependencies":{"astro":"^7.3.5"}}.
		expect(MANIFEST.slice(26, 32)).toBe("^7.2.2");
		expect(actions).toContainEqual(
			objectContaining({
				edit: {
					documentChanges: [
						{
							edits: [
								{
									newText: "^7.3.5",
									range: {
										end: { character: 32, line: 0 },
										start: { character: 26, line: 0 },
									},
								},
							],
							textDocument: { uri: URI, version: 1 },
						},
					],
				},
				isPreferred: true,
				kind: "quickfix",
				title: "Bump to ^7.3.5",
			}),
		);
		// Code actions read the last plan and never reach the registry.
		expect(registry.authorization).toHaveLength(requestsBefore);
	});

	it("applies the editor toggles over the settings it hands the server", async (): Promise<void> => {
		const { configurations, connection } = await startSession({
			flags: { showPrereleases: true },
		});
		open(connection);
		await settled(connection);

		expect(JSON.stringify(connection.requests)).toContain(
			"workspace/didChangeWatchedFiles",
		);
		expect(configurations).not.toHaveLength(0);
		expect(configurations[0]).toMatchObject({
			suggestions: {
				showPrereleasesOnStartup: true,
				showVulnerabilities: false,
			},
		});
	});
});
