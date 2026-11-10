// Runs inside the VS Code extension host (see scripts/test/e2e.mjs). The
// fixture registry answers every package with versions 1.0.0, 1.1.0 and
// 2.0.0, so `alpha` at 1.0.0 is outdated.
import assert from "node:assert/strict";
import { join } from "node:path";
import process from "node:process";

import * as vscode from "vscode";

const TIMEOUT_MS = 60_000;

const isCancellation = (error: unknown): boolean =>
	error instanceof vscode.CancellationError ||
	(error instanceof Error && error.message === "Canceled");

const attempt = async <T>(
	probe: () => Promise<T | undefined> | T | undefined,
): Promise<T | undefined> => {
	let value: T | undefined;
	try {
		value = await probe();
	} catch (error: unknown) {
		// VS Code cancels a provider request when another extension (here the
		// built-in JSON one) registers a provider mid-call; ask again.
		if (!isCancellation(error)) {
			throw error;
		}
	}
	return value;
};

const waitFor = async <T>(
	description: string | (() => string),
	probe: () => Promise<T | undefined> | T | undefined,
	deadline = Date.now() + TIMEOUT_MS,
): Promise<T> => {
	if (Date.now() >= deadline) {
		throw new Error(
			`timed out waiting for ${typeof description === "string" ? description : description()}`,
		);
	}
	const value = await attempt(probe);
	if (value !== undefined) {
		return value;
	}
	await new Promise<void>((resolve) => {
		setTimeout(resolve, 250);
	});
	return waitFor(description, probe, deadline);
};

const waitForLenses = (
	document: vscode.TextDocument,
): Promise<vscode.CodeLens[]> => {
	let seen = 0;
	return waitFor(
		() => `code lenses (last answer had ${seen} lenses)`,
		async () => {
			const found = await vscode.commands.executeCommand<vscode.CodeLens[]>(
				"vscode.executeCodeLensProvider",
				document.uri,
			);
			seen = found.length;
			return found.some((lens) => lens.command !== undefined)
				? found
				: undefined;
		},
	);
};

const waitForQuickFix = (
	document: vscode.TextDocument,
	diagnostic: vscode.Diagnostic,
): Promise<vscode.CodeAction & { edit: vscode.WorkspaceEdit }> => {
	let titles: string[] = [];
	return waitFor(
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
};

export const run = async (): Promise<void> => {
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

	const lenses = await waitForLenses(document);
	assert.ok(
		lenses.some((lens) => lens.command?.title.includes("2.0.0") === true),
		`a lens offers 2.0.0: ${JSON.stringify(lenses.map((l) => l.command?.title))}`,
	);

	const diagnostic = await waitFor("an update diagnostic", () =>
		vscode.languages
			.getDiagnostics(document.uri)
			.find((item) => item.code === "update-available"),
	);

	const fix = await waitForQuickFix(document, diagnostic);

	assert.ok(await vscode.workspace.applyEdit(fix.edit), "the edit applies");
	assert.match(document.getText(), /"alpha": "2\.0\.0"/u);
	process.stdout.write("e2e: lenses, update diagnostic and quick fix OK\n");
};
