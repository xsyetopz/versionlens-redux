import { ExecuteCommandRequest } from "vscode-languageclient/node";

import type { TextDocument } from "#vscode-host";

import type { ExtensionState } from "./state.ts";

/** Answer of the server's `versionlens.server.manifestInfo` command. */
interface ManifestInfo {
	canSortDependencies: boolean;
	dependencyCount: number;
	dependencySignature: string;
	installTaskConfigKey: string | null;
	provider: string | null;
	supported: boolean;
}

const MANIFEST_INFO_COMMAND = "versionlens.server.manifestInfo";

interface CachedInfo {
	info: Promise<ManifestInfo | undefined>;
	version: number;
}

const cache = new Map<string, CachedInfo>();

const clearManifestInfo = (): void => {
	cache.clear();
};

/**
 * What the server knows about an open document, or `undefined` when the
 * server is not running or has no such document. Cached per document
 * version.
 */
const manifestInfo = (
	state: ExtensionState,
	document: TextDocument,
): Promise<ManifestInfo | undefined> => {
	const key = document.uri.toString();
	const cached = cache.get(key);
	if (cached?.version === document.version) {
		return cached.info;
	}
	const info = requestManifestInfo(state, document);
	cache.set(key, { info, version: document.version });
	return info;
};

const requestManifestInfo = async (
	state: ExtensionState,
	document: TextDocument,
): Promise<ManifestInfo | undefined> => {
	const { client } = state;
	let info: ManifestInfo | undefined;
	if (client?.isRunning() === true) {
		try {
			const answer: unknown = await client.sendRequest(
				ExecuteCommandRequest.type,
				{
					arguments: [
						{ uri: client.code2ProtocolConverter.asUri(document.uri) },
					],
					command: MANIFEST_INFO_COMMAND,
				},
			);
			if (isManifestInfo(answer)) {
				info = answer;
			}
		} catch {
			// The server does not track documents outside the selector.
			cache.delete(document.uri.toString());
		}
	} else {
		cache.delete(document.uri.toString());
	}
	return info;
};

const isManifestInfo = (value: unknown): value is ManifestInfo =>
	typeof value === "object" &&
	value !== null &&
	"supported" in value &&
	"dependencySignature" in value;

export type { ManifestInfo };
export { clearManifestInfo, manifestInfo };
