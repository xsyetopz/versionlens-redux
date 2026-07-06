import { expect, mock, test } from "bun:test";

let inputValues: (string | undefined)[] = [];
let quickPickResult: unknown;
const quickPickCalls: unknown[] = [];
const warningCalls: unknown[] = [];

type AuthCollection = { [url: string]: unknown };

mock.module("vscode", () => ({
	window: {
		activeTextEditor: {
			document: {
				uri: { path: "/workspace/package.json" },
			},
		},
		showInputBox() {
			return inputValues.shift();
		},
		showQuickPick(items: unknown[], options?: unknown) {
			quickPickCalls.push({ items, options });
			return quickPickResult ?? items[0];
		},
		showWarningMessage(...args: unknown[]) {
			warningCalls.push(args);
			return "Yes";
		},
	},
}));

test("stores basic auth metadata in workspace state and secret storage", async () => {
	const { addAuthHeader, authHeaders } = await import("../auth.ts");
	const context = extensionContext();
	inputValues = ["https://registry.example.com///", "alice", "s3cret"];
	quickPickResult = undefined;
	quickPickCalls.length = 0;

	const added = await addAuthHeader({ context } as never);

	expect(added).toBe(true);
	expect(context.workspaceState.value).toEqual({
		"https://registry.example.com": {
			label: "Basic Auth",
			protocol: "https:",
			scheme: "Basic",
			status: "NoStatus",
			url: "https://registry.example.com",
		},
	});
	expect(
		context.secrets.values.get("/storage__https://registry.example.com"),
	).toBe(Buffer.from("alice:s3cret").toString("base64"));
	expect(await authHeaders({ context } as never)).toEqual([
		{
			name: "Authorization",
			url: "https://registry.example.com",
			value: `Basic ${Buffer.from("alice:s3cret").toString("base64")}`,
		},
	]);
});

test("removes selected auth metadata and non-empty secrets", async () => {
	const { removeAuthHeader } = await import("../auth.ts");
	const context = extensionContext({
		"https://registry.example.com": {
			label: "Basic Auth",
			protocol: "https:",
			scheme: "Basic",
			status: "NoStatus",
			url: "https://registry.example.com",
		},
		"https://cancelled.example.com": {
			protocol: "https:",
			scheme: "NotSet",
			status: "User cancelled",
			url: "https://cancelled.example.com",
		},
	});
	context.secrets.values.set(
		"/storage__https://registry.example.com",
		"credential",
	);
	context.secrets.values.set(
		"/storage__https://cancelled.example.com",
		"should-remain",
	);
	quickPickResult = [
		{
			entry: context.workspaceState.value["https://registry.example.com"],
			label: "https://registry.example.com",
		},
		{
			entry: context.workspaceState.value["https://cancelled.example.com"],
			label: "https://cancelled.example.com",
		},
	];

	const removed = await removeAuthHeader({ context } as never);

	expect(removed).toBe(true);
	expect(context.workspaceState.value).toEqual({});
	expect(
		context.secrets.values.has("/storage__https://registry.example.com"),
	).toBe(false);
	expect(
		context.secrets.values.get("/storage__https://cancelled.example.com"),
	).toBe("should-remain");
});

function extensionContext(initialAuth: AuthCollection = {}) {
	const secrets = new Map<string, string>();
	const workspaceState = {
		value: initialAuth,
		get(_key: string, fallback: unknown) {
			return this.value ?? fallback;
		},
		update(_key: string, value: unknown) {
			this.value = value as AuthCollection;
		},
	};
	return {
		secrets: {
			values: secrets,
			delete(key: string) {
				secrets.delete(key);
			},
			get(key: string) {
				return secrets.get(key);
			},
			store(key: string, value: string) {
				secrets.set(key, value);
			},
		},
		storageUri: { path: "/storage" },
		workspaceState,
	};
}
