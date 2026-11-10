import type { TextDocument } from "#vscode-host";

import { fileDocument } from "../documents.ts";
import { manifestInfo } from "../manifest.ts";
import type { ExtensionState } from "../state.ts";
import {
	dependencySnapshot,
	rememberDependencySnapshot,
} from "../tasks/snapshot.ts";

/** Remembers the dependencies of a manifest to tell later edits apart. */
const trackDocument = async (
	state: ExtensionState,
	textDocument: TextDocument,
): Promise<void> => {
	const document = fileDocument(textDocument);
	if (document === undefined) {
		return;
	}
	const info = await manifestInfo(state, document);
	if (info?.supported !== true) {
		return;
	}
	rememberDependencySnapshot(
		state,
		document,
		await dependencySnapshot(state, document),
	);
};

export { trackDocument };
