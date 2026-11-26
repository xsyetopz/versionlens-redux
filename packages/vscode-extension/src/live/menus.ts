// The editor title icons and menus, driven the way VS Code's title bar drives them.
import { commands, type TextEditor, Uri } from "vscode";

import { eventually, hintCount, lenses, report } from "./probe.ts";

/** The extension API: context keys per file uri. */
export type Keys = (uri: string) => Record<string, boolean>;

/** The editor title icons: context keys, and the icon commands as the title bar calls them. */
const icons = async (manifest: Uri, keys: Keys): Promise<void> => {
	const key = (name: string): boolean | undefined =>
		keys(manifest.toString())[`versionlens.${name}`];
	report(
		"title icons: manifest is active, idle, shown",
		await eventually(() =>
			Promise.resolve(
				key("providerActive") === true &&
					key("providerBusy") === false &&
					key("show") === true
			)
		),
		JSON.stringify(keys(manifest.toString()))
	);
	// Editor title menus pass the file's Uri as the first argument.
	await commands.executeCommand("versionlens.hide", manifest);
	report(
		"hide icon hides lenses and flips the icon",
		await eventually(async () => {
			const shown = await lenses(manifest);
			return shown.length === 0 && key("show") === false;
		}, 5000)
	);
	await commands.executeCommand("versionlens.show", manifest);
	report(
		"show icon restores them",
		await eventually(() => Promise.resolve(key("show") === true), 5000)
	);
	await commands.executeCommand("versionlens.showPrereleases", manifest);
	report(
		"prerelease tag icon turns on",
		await eventually(
			() => Promise.resolve(key("showPrereleases") === true),
			5000
		)
	);
	await commands.executeCommand("versionlens.hidePrereleases", manifest);
	report(
		"prerelease tag icon turns off",
		await eventually(
			() => Promise.resolve(key("showPrereleases") === false),
			5000
		)
	);
};

/** The `...` menu entries, called with the file's Uri as the editor title menu passes it. */
const edits = async (editor: TextEditor): Promise<void> => {
	const manifest = editor.document.uri;
	await eventually(hintCount(manifest, 2));
	await commands.executeCommand("versionlens.updateAll.minor", manifest);
	const patched = await eventually(
		() => Promise.resolve(!editor.document.getText().includes("==2.31.0")),
		5000
	);
	report(
		"menu: update dependencies (minor-only) edits the file",
		patched,
		editor.document.getText().replaceAll("\n", " | ")
	);
	await commands.executeCommand("workbench.action.files.revert");
	await commands.executeCommand("versionlens.sort", manifest);
	report(
		"menu: sort dependencies alphabetically",
		await eventually(
			() => Promise.resolve(editor.document.getText().startsWith("flask")),
			5000
		),
		editor.document.getText().replaceAll("\n", " | ")
	);
	await commands.executeCommand("workbench.action.files.revert");
};

/** Checks the editor title icons and menus against the manifest in `editor`. */
export const menus = async (editor: TextEditor, keys: Keys): Promise<void> => {
	const manifest = editor.document.uri;
	report(
		"title icons: other files are inactive",
		keys(Uri.joinPath(manifest, "..", "main.ts").toString())[
			"versionlens.providerActive"
		] === false
	);
	await icons(manifest, keys);
	await edits(editor);
};
