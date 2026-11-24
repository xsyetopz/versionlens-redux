// Drives VersionLens inside a real VS Code window through the public editor API, against the
// live registries and OSV. `just vscode-live` builds this file and launches VS Code with it.
import {
	type CodeAction,
	commands,
	ConfigurationTarget,
	extensions,
	type Hover,
	MarkdownString,
	Position,
	Range,
	type TextEditor,
	Uri,
	window,
	workspace
} from "vscode";

import { type Keys, menus } from "./menus.ts";
import {
	eventually,
	failures,
	hintCount,
	hints,
	label,
	lenses,
	ours,
	report,
	sleep
} from "./probe.ts";

const features = async (manifest: Uri, other: Uri): Promise<void> => {
	const started = Date.now();
	const resolved = await eventually(hintCount(manifest, 2));
	const shown = await hints(manifest);
	report(
		"inlay hints",
		resolved,
		`${shown.map((hint) => label(hint)).join(", ")} after ${Date.now() - started}ms`
	);
	const shownLenses = await lenses(manifest);
	report(
		"code lenses",
		shownLenses.length > 0 &&
			shownLenses.every((lens) => lens.command !== undefined),
		shownLenses.map((lens) => lens.command?.title ?? "").join(" | ")
	);
	const hxml = Uri.joinPath(manifest, "..", "libraries.hxml");
	await workspace.openTextDocument(hxml);
	report("libraries.hxml hints", await eventually(hintCount(hxml, 2)));
	// The original matched `dockerfile` and `*.dockerfile`; globs match regardless of case.
	const dockerfile = Uri.joinPath(manifest, "..", "dockerfile");
	await workspace.openTextDocument(dockerfile);
	report(
		"lowercase dockerfile hints",
		await eventually(hintCount(dockerfile, 1))
	);
	const otherHints = await hints(other);
	report(
		"other files get nothing",
		otherHints.length === 0 && ours(other).length === 0
	);
};

const assists = async (manifest: Uri): Promise<void> => {
	const before = Date.now();
	const actions = await commands.executeCommand<CodeAction[]>(
		"vscode.executeCodeActionProvider",
		manifest,
		new Range(0, 0, 0, 3)
	);
	report(
		"quick fixes on the package name",
		actions.some(({ title }) => title.startsWith("Update")),
		`${Date.now() - before}ms: ${actions.map(({ title }) => title).join(" | ")}`
	);
	const hovers = await commands.executeCommand<Hover[]>(
		"vscode.executeHoverProvider",
		manifest,
		new Position(0, 2)
	);
	const text = hovers
		.flatMap(({ contents }) =>
			contents.flatMap((part) =>
				part instanceof MarkdownString ? [part.value] : []
			)
		)
		.join(" ");
	report(
		"hover",
		text.includes("requests"),
		text.slice(0, 80).replaceAll("\n", " ")
	);
	const flagged = await eventually(() =>
		Promise.resolve(ours(manifest).length > 0)
	);
	report(
		"OSV vulnerability diagnostics",
		flagged,
		ours(manifest)
			.map(({ message }) => message.slice(0, 50))
			.join(" | ")
	);
};

const settings = async (manifest: Uri): Promise<void> => {
	const config = workspace.getConfiguration("versionlens");
	await config.update("inlayHints", false, ConfigurationTarget.Workspace);
	report(
		"inlayHints=false hides hints",
		await eventually(hintCount(manifest, 0), 5000)
	);
	await config.update("inlayHints", undefined, ConfigurationTarget.Workspace);
	report(
		"resetting it restores hints",
		await eventually(hintCount(manifest, 2), 5000)
	);
};

const palette = async (editor: TextEditor): Promise<void> => {
	const manifest = editor.document.uri;
	const hidden = async (): Promise<boolean> => {
		const shown = await lenses(manifest);
		return shown.length === 0;
	};
	await commands.executeCommand("versionlens.toggle");
	report("toggle hides lenses", await eventually(hidden, 5000));
	await commands.executeCommand("versionlens.toggle");
	report(
		"toggle again restores them",
		await eventually(async () => !(await hidden()), 5000)
	);
	await commands.executeCommand("versionlens.clearCache");
	await commands.executeCommand("versionlens.refresh");
	report(
		"clearCache and refresh resolve again",
		await eventually(hintCount(manifest, 2))
	);
	await commands.executeCommand("versionlens.updateAll");
	const updated = await eventually(
		() => Promise.resolve(!editor.document.getText().includes("==2.31.0")),
		5000
	);
	report(
		"updateAll edits the file",
		updated,
		editor.document.getText().replaceAll("\n", " | ")
	);
	await commands.executeCommand("workbench.action.files.revert");
	await editor.edit((edit) => {
		edit.insert(new Position(2, 0), "urllib3==1.26.5\n");
	});
	report(
		"a typed dependency gets a hint",
		await eventually(hintCount(manifest, 3))
	);
	await commands.executeCommand("workbench.action.files.revert");
	await commands.executeCommand("versionlens.showOutput");
	const restarted = Date.now();
	await commands.executeCommand("versionlens.restart");
	report(
		"restart brings the server back",
		await eventually(hintCount(manifest, 2)),
		`${Date.now() - restarted}ms`
	);
};

const output = async (logs: string): Promise<void> => {
	const directory = Uri.file(
		`${logs}/window1/exthost/xsyetopz.versionlens-redux`
	);
	const bytes = await workspace.fs.readFile(
		Uri.joinPath(directory, "VersionLens.log")
	);
	const log = new TextDecoder().decode(bytes);
	console.info(log);
	report(
		"output logs start, settings, and commands",
		["starting", "settings applied", "command versionlens.toggle"].every(
			(line) => log.includes(line)
		)
	);
	report("output has no errors", !log.includes("[error]"));
};

/** Entry point VS Code calls for `--extensionTestsPath`; a thrown error fails the run. */
export const run = async (): Promise<void> => {
	const root = workspace.workspaceFolders?.[0]?.uri;
	const logs = process.env["VERSIONLENS_LIVE_LOGS"];
	if (root === undefined || logs === undefined) {
		throw new Error("open the live workspace and set VERSIONLENS_LIVE_LOGS");
	}
	const api: { contextKeys: Keys } | undefined = await extensions
		.getExtension<{ contextKeys: Keys }>("xsyetopz.versionlens-redux")
		?.activate();
	if (api === undefined) {
		throw new Error("the extension did not activate");
	}
	const manifest = Uri.joinPath(root, "requirements.txt");
	const other = Uri.joinPath(root, "main.ts");
	const editor = await window.showTextDocument(
		await workspace.openTextDocument(manifest)
	);
	await workspace.openTextDocument(other);
	await features(manifest, other);
	await assists(manifest);
	await settings(manifest);
	await menus(editor, api.contextKeys);
	await palette(editor);
	await sleep(500);
	await output(logs);
	if (failures.length > 0) {
		throw new Error(`failed: ${failures.join(", ")}`);
	}
};
