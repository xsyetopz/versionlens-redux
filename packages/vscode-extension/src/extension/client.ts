import type {
  Executable,
  LanguageClientOptions,
} from "vscode-languageclient/node";
import { LanguageClient, TransportKind } from "vscode-languageclient/node";

export function createLanguageClient(serverPath: string): LanguageClient {
  const serverOptions: Executable = {
    command: serverPath,
    transport: TransportKind.stdio,
  };
  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file" }, { scheme: "untitled" }],
  };

  return new LanguageClient(
    "versionlens",
    "VersionLens Redux",
    serverOptions,
    clientOptions,
  );
}
