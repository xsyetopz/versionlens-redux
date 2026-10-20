// Runs inside the VS Code extension host (see scripts/test/e2e.mjs). The
// fixture registry answers every package with versions 1.0.0, 1.1.0 and
// 2.0.0, so `alpha` at 1.0.0 is outdated.
import assert from "node:assert/strict";
import { join } from "node:path";
import * as vscode from "vscode";

const TIMEOUT_MS = 60_000;

async function waitFor<T>(
  description: string | (() => string),
  probe: () => Promise<T | undefined> | T | undefined,
): Promise<T> {
  const deadline = Date.now() + TIMEOUT_MS;
  while (Date.now() < deadline) {
    const value = await Promise.resolve()
      .then(probe)
      .catch((error: unknown) => {
        // VS Code cancels a provider request when another extension (here the
        // built-in JSON one) registers a provider mid-call; ask again.
        if (isCancellation(error)) {
          return undefined;
        }
        throw error;
      });
    if (value !== undefined) {
      return value;
    }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  throw new Error(
    `timed out waiting for ${typeof description === "string" ? description : description()}`,
  );
}

function isCancellation(error: unknown): boolean {
  return (
    error instanceof vscode.CancellationError ||
    (error instanceof Error && error.message === "Canceled")
  );
}

export async function run(): Promise<void> {
  const folder = vscode.workspace.workspaceFolders?.[0];
  assert.ok(folder, "the fixture workspace is open");
  const extension = vscode.extensions.getExtension(
    "xsyetopz.versionlens-redux",
  );
  assert.ok(extension, "the extension is installed");
  await extension.activate();

  const document = await vscode.workspace.openTextDocument(
    vscode.Uri.file(join(folder.uri.fsPath, "package.json")),
  );
  await vscode.window.showTextDocument(document);

  let seen = 0;
  const lenses = await waitFor(
    () => `code lenses (last answer had ${seen} lenses)`,
    async () => {
      const found = await vscode.commands.executeCommand<vscode.CodeLens[]>(
        "vscode.executeCodeLensProvider",
        document.uri,
      );
      seen = found.length;
      return found.some((lens) => lens.command) ? found : undefined;
    },
  );
  assert.ok(
    lenses.some((lens) => lens.command?.title.includes("2.0.0")),
    `a lens offers 2.0.0: ${JSON.stringify(lenses.map((l) => l.command?.title))}`,
  );

  const diagnostic = await waitFor("an update diagnostic", () =>
    vscode.languages
      .getDiagnostics(document.uri)
      .find((item) => item.code === "update-available"),
  );

  let titles: string[] = [];
  const fix = await waitFor(
    () => `a quick fix to 2.0.0 (last answer: ${JSON.stringify(titles)})`,
    async () => {
      const actions = await vscode.commands.executeCommand<
        (vscode.CodeAction | vscode.Command)[]
      >(
        "vscode.executeCodeActionProvider",
        document.uri,
        diagnostic.range,
        vscode.CodeActionKind.QuickFix.value,
      );
      titles = actions.map((action) => action.title);
      return actions.find(
        (
          action,
        ): action is vscode.CodeAction & { edit: vscode.WorkspaceEdit } =>
          "edit" in action &&
          Boolean(action.edit) &&
          action.title.includes("2.0.0"),
      );
    },
  );

  assert.ok(await vscode.workspace.applyEdit(fix.edit), "the edit applies");
  assert.match(document.getText(), /"alpha": "2\.0\.0"/u);
  console.log("e2e: lenses, update diagnostic and quick fix OK");
}
