import type { ExtensionContext } from "#vscode-host";

import { createExtensionState, type ExtensionState } from "../state.ts";

type AuthCollection = Record<string, unknown>;

interface TestExtensionContext {
	secrets: {
		values: Map<string, string>;
		delete: (key: string) => void;
		get: (key: string) => string | undefined;
		store: (key: string, value: string) => void;
	};
	storageUri?: { path: string } | undefined;
	workspaceState: {
		value: AuthCollection;
		get: (_key: string, fallback: unknown) => AuthCollection;
		update: (_key: string, value: unknown) => void;
	};
}

const isCollection = (value: unknown): value is AuthCollection =>
	typeof value === "object" && value !== null;

const isExtensionContext = (value: object): value is ExtensionContext =>
	"secrets" in value && "workspaceState" in value;

const extensionContext = (
	initialAuth: AuthCollection = {},
): TestExtensionContext => {
	const secrets = new Map<string, string>();
	const workspaceState = {
		value: initialAuth,
		get(_key: string, _fallback: unknown): AuthCollection {
			return this.value;
		},
		update(_key: string, value: unknown): void {
			if (!isCollection(value)) {
				throw new TypeError("Auth metadata must be an object");
			}
			this.value = value;
		},
	};
	return {
		secrets: {
			values: secrets,
			delete(key: string): void {
				secrets.delete(key);
			},
			get(key: string): string | undefined {
				return secrets.get(key);
			},
			store(key: string, value: string): void {
				secrets.set(key, value);
			},
		},
		storageUri: { path: "/storage" },
		workspaceState,
	};
};

/** The extension state the auth commands read, around a fake context. */
const stateFor = (context: TestExtensionContext): ExtensionState => {
	if (!isExtensionContext(context)) {
		throw new TypeError("The fake context lacks secrets or workspace state");
	}
	return { ...createExtensionState(), context };
};

const rejectedAuthUrls = [
	{
		name: "suffix hosts",
		authUrl: "https://registry.example.com/packages",
		requestUrl: "https://registry.example.com.attacker.test/packages/pkg",
	},
	{
		name: "different ports",
		authUrl: "https://registry.example.com/packages",
		requestUrl: "https://registry.example.com:8443/packages/pkg",
	},
	{
		name: "different schemes",
		authUrl: "https://registry.example.com/packages",
		requestUrl: "http://registry.example.com/packages/pkg",
	},
	{
		name: "path prefixes without a segment boundary",
		authUrl: "https://registry.example.com/packages",
		requestUrl: "https://registry.example.com/packages-private/pkg",
	},
	{
		name: "path case differences",
		authUrl: "https://registry.example.com/packages",
		requestUrl: "https://registry.example.com/Packages/pkg",
	},
];

export type { AuthCollection, TestExtensionContext };
export { extensionContext, rejectedAuthUrls, stateFor };
