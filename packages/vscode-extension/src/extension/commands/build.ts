import * as vscode from "vscode";
import type { ExtensionState } from "../state.ts";
import { applyRustEdits } from "./apply.ts";

export async function chooseBuild(
	state: ExtensionState,
	selector: string | undefined,
	packageName: string | undefined,
	currentVersion: string | undefined,
	builds: string[],
) {
	if (
		!selector ||
		builds.length === 0 ||
		state.flags.codeLensReplace === false
	) {
		return;
	}

	const selected = await vscode.window.showQuickPick(
		builds.map((build) => ({
			label: build,
			picked: build === currentVersion,
		})),
		{
			placeHolder: "Choose a build or press escape to cancel",
			title: `Choose a build for ${packageName ?? ""}`.trim(),
		},
	);
	if (!selected) {
		return;
	}

	await applyRustEdits(state, selector, undefined, selected.label);
}
