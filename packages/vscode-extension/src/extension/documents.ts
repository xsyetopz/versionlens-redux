import {
	type DocumentFilter,
	type TextDocument,
	type Uri,
	workspace,
} from "#vscode-host";

import { enabledFilePatternKeys } from "./config/keys.ts";

const fileDocument = (
	document: TextDocument | undefined,
): TextDocument | undefined => {
	let file: TextDocument | undefined;
	if (document?.uri.scheme === "file") {
		file = document;
	}
	return file;
};

type FileDocumentSelectors = {
	language: string;
	pattern: string;
	scheme: string;
}[];

/** The documents the server is told about, from the enabled providers. */
const documentSelectors = (): FileDocumentSelectors => {
	const selectors = [
		undefined,
		...(workspace.workspaceFolders ?? []).map(({ uri }): Uri => uri),
	].flatMap((resource): FileDocumentSelectors =>
		selectorsForResource(resource),
	);
	return [
		...new Map(
			selectors.map((selector): [string, FileDocumentSelectors[number]] => [
				selectorKey(selector),
				selector,
			]),
		).values(),
	];
};

const selectorsForResource = (
	resource: Uri | undefined,
): FileDocumentSelectors => {
	const config = workspace.getConfiguration("versionlens", resource);
	return enabledFilePatternKeys(
		config.get<string[]>("enabledProviders"),
	).flatMap(([, key, languages]): FileDocumentSelectors => {
		const pattern = config.get<string>(key) ?? "**/*";
		return languages.map((language): FileDocumentSelectors[number] => ({
			language,
			pattern,
			scheme: "file",
		}));
	});
};

const selectorKey = ({ language, pattern, scheme }: DocumentFilter): string =>
	`${scheme}\0${language}\0${typeof pattern === "string" ? pattern : pattern?.pattern}`;

export { documentSelectors, fileDocument };
