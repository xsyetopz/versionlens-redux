import * as vscode from "vscode";
import { enabledFilePatternKeys } from "../config/keys/files.ts";

export function documentSelectors(): vscode.DocumentSelector {
	const config = vscode.workspace.getConfiguration("versionlens");
	const selectors: vscode.DocumentFilter[] = enabledFilePatternKeys(
		config.get<string[]>("enabledProviders"),
	).flatMap(([, key, languages]) => {
		const pattern = config.get<string>(key) ?? "**/*";
		return languages.map((language) => ({
			language,
			pattern,
			scheme: "file",
		}));
	});
	return selectors;
}
