import { join } from "node:path";
import process from "node:process";

/** Where the packaged extension keeps the server, relative to its folder. */
const BUNDLED_SERVER_DIRECTORY = "server";

/** Environment variable that points at a server binary to use instead. */
const SERVER_ENVIRONMENT_VARIABLE = "VERSIONLENS_LSP";

function bundledServerPath(
  extensionPath: string,
  platform: NodeJS.Platform = process.platform,
): string {
  let name = "versionlens-lsp";
  if (platform === "win32") {
    name = "versionlens-lsp.exe";
  }
  return join(extensionPath, BUNDLED_SERVER_DIRECTORY, name);
}

function resolveServerPath(
  extensionPath: string,
  environment: NodeJS.ProcessEnv = process.env,
): string {
  const override = environment[SERVER_ENVIRONMENT_VARIABLE]?.trim();
  if (override) {
    return override;
  }
  return bundledServerPath(extensionPath);
}

export { bundledServerPath, resolveServerPath, SERVER_ENVIRONMENT_VARIABLE };
