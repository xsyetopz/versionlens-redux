import { afterEach, beforeAll, describe, expect, it } from "bun:test";

import {
	extensionContext,
	stateFor,
} from "../src/extension/__tests__/auth-context.ts";
import type { ExtensionState } from "../src/extension/state.ts";
import type { Json, LspConnection } from "./support/lsp-client.ts";
import type { FakeRegistry } from "./support/session.ts";
import {
	buildServerBinary,
	diagnostics,
	hasPublishedDiagnostics,
	hasStatus,
	open,
	runCleanups,
	startRegistry,
	startSession,
} from "./support/session.ts";

// A cold debug build of the server takes minutes.
beforeAll(buildServerBinary, 600_000);

afterEach(runCleanups);

/** An extension state whose SecretStorage holds one credential. */
const authState = (
	registry: FakeRegistry,
	scheme: "Basic" | "Custom" | "NotSet",
	token?: string,
): ExtensionState => {
	const context = extensionContext({
		[registry.url]: {
			protocol: "http:",
			scheme,
			status: scheme === "NotSet" ? "User cancelled" : "NoStatus",
			url: registry.url,
		},
	});
	if (token !== undefined) {
		context.secrets.values.set(`/storage__${registry.url}`, token);
	}
	return stateFor(context);
};

const authorizationParams = (
	params: Json,
): { host: string; status: number } => {
	const { host, status } = params;
	if (typeof host !== "string" || typeof status !== "number") {
		throw new TypeError("Unexpected versionlens/authorization params");
	}
	return { host, status };
};

const authorizedSession = async (
	registry: FakeRegistry,
	state: ExtensionState,
): Promise<{ connection: LspConnection; asked: Json[] }> => {
	const { answerAuthorization } =
		await import("../src/extension/auth/authorization.ts");
	const asked: Json[] = [];
	const { connection } = await startSession({
		authorization: (params: Json): unknown => {
			asked.push(params);
			return answerAuthorization(state, authorizationParams(params));
		},
		registry,
	});
	return { asked, connection };
};

describe("versionlens/authorization answered by the client", (): void => {
	it("sends a custom value verbatim as the Authorization header", async (): Promise<void> => {
		const registry = startRegistry("Bearer s3cret");
		const { asked, connection } = await authorizedSession(
			registry,
			authState(registry, "Custom", "Bearer s3cret"),
		);
		open(connection);

		expect(await diagnostics(connection)).toHaveLength(1);
		expect(asked[0]).toEqual({ host: "127.0.0.1", status: 401 });
		expect(registry.authorization[0]).toBeNull();
		expect(registry.authorization).toContain("Bearer s3cret");
	});

	it("sends Basic credentials as Basic <base64>", async (): Promise<void> => {
		const encoded = Buffer.from("alice:pw").toString("base64");
		const registry = startRegistry(`Basic ${encoded}`);
		const { connection } = await authorizedSession(
			registry,
			authState(registry, "Basic", encoded),
		);
		open(connection);

		expect(await diagnostics(connection)).toHaveLength(1);
		expect(registry.authorization).toContain(`Basic ${encoded}`);
	});

	// The server publishes its diagnostics before it sends idle (checking.rs),
	// so an idle with nothing published means nothing will be.
	it("falls back to no credential when the user declined the host", async (): Promise<void> => {
		const registry = startRegistry("Bearer never");
		const { asked, connection } = await authorizedSession(
			registry,
			authState(registry, "NotSet"),
		);
		open(connection);

		await connection.waitFor("versionlens/status", (params): boolean =>
			hasStatus(params, "idle"),
		);
		expect(asked).toHaveLength(1);
		expect(registry.authorization.every((header) => header === null)).toBe(
			true,
		);
		expect(hasPublishedDiagnostics(connection)).toBe(false);
	});
});
