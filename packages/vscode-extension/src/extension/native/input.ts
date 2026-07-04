export type NativeDocumentInput = {
	languageId: string;
	text: string;
	uri: string;
	workspaceRoot?: string;
};

export type NativeApplyCommand =
	| "sort"
	| "update"
	| "updateMajor"
	| "updateMinor"
	| "updatePatch"
	| "updateRelease"
	| "updatePrerelease";

export type NativeApplyCommandInput = {
	command?: NativeApplyCommand;
	dependencyName?: string;
	document: NativeDocumentInput;
	selectedVersion?: string;
};
