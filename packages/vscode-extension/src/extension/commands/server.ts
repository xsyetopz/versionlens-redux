import { ExecuteCommandRequest } from "vscode-languageclient/node";
import { window } from "#vscode-host";
import type { ExtensionState } from "../state.ts";

/** Command IDs the server lists in its `executeCommandProvider`. */
const SERVER_COMMANDS = {
  clearCache: "versionlens.server.clearCache",
  sort: "versionlens.server.sort",
  updateAll: "versionlens.server.updateAll",
} as const;

type UpdateLevel = "latest" | "major" | "minor" | "patch";

/** Runs a server command through `workspace/executeCommand`. */
async function runServerCommand(
  state: ExtensionState,
  command: string,
  argument?: object,
): Promise<unknown> {
  const { client } = state;
  if (!client?.isRunning()) {
    throw new Error("The VersionLens language server is not running.");
  }
  const args = argument ? [argument] : [];
  return await client.sendRequest(ExecuteCommandRequest.type, {
    arguments: args,
    command,
  });
}

/** Runs a server command on the document of the active editor. */
function runDocumentCommand(
  state: ExtensionState,
  command: string,
  extra: object = {},
): Promise<unknown> {
  const document = window.activeTextEditor?.document;
  if (!(document && state.client)) {
    throw new Error("Open a supported manifest file first.");
  }
  return runServerCommand(state, command, {
    ...extra,
    uri: state.client.code2ProtocolConverter.asUri(document.uri),
  });
}

export type { UpdateLevel };
export { runDocumentCommand, runServerCommand, SERVER_COMMANDS };
