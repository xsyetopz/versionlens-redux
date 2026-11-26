// Times one cold start of whichever VersionLens the window loaded: from opening the manifest to
// a resolved version lens on every dependency line. `just vscode-bench` runs it for the
// original 1.x extension and this one, in alternating fresh windows.
import { commands, Uri, window, workspace } from "vscode";

import { lenses, sleep } from "./probe.ts";

const LINES = [0, 1];
const TIMEOUT = 60_000;

/** Whether each dependency line has a lens whose title carries a version. */
const resolved = async (uri: Uri): Promise<boolean> => {
	const shown = await lenses(uri);
	return LINES.every((line) =>
		shown.some(
			(lens) =>
				lens.range.start.line === line && /\d/u.test(lens.command?.title ?? "")
		)
	);
};

/** Entry point VS Code calls for `--extensionTestsPath`; prints one `BENCH <ms>` line. */
export const run = async (): Promise<void> => {
	const root = workspace.workspaceFolders?.[0]?.uri;
	if (root === undefined) {
		throw new Error("open the bench workspace");
	}
	const manifest = Uri.joinPath(root, "requirements.txt");
	const titles = async (): Promise<string> => {
		const shown = await lenses(manifest);
		return JSON.stringify(shown.map((lens) => lens.command?.title));
	};
	const started = performance.now();
	// A 10 ms poll keeps the measurement finer than the shared 100 ms `eventually`.
	const wait = async (): Promise<void> => {
		if (await resolved(manifest)) {
			return;
		}
		if (performance.now() - started > TIMEOUT) {
			throw new Error(`no resolved lenses: ${await titles()}`);
		}
		await sleep(10);
		await wait();
	};
	await window.showTextDocument(await workspace.openTextDocument(manifest));
	await wait();
	const elapsed = Math.round(performance.now() - started);
	console.info(`BENCH ${elapsed} ${await titles()}`);
	await commands.executeCommand("workbench.action.closeAllEditors");
};
