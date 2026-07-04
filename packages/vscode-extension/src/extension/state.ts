import type * as vscode from "vscode";
import type { NativeSession } from "./native/module.ts";

export interface ExtensionState {
	context: vscode.ExtensionContext | undefined;
	session: NativeSession | undefined;
	ui: {
		diagnostics: vscode.DiagnosticCollection | undefined;
		outputChannel: vscode.OutputChannel | undefined;
		codeLensRefresh: vscode.EventEmitter<void> | undefined;
	};
	flags: {
		showVersionLenses: boolean;
		showOutdated: boolean;
		showPrereleases: boolean;
		showSuggestionStats: boolean;
		providerBusy: number;
		providerError: boolean;
		codeLensReplace: boolean;
	};
	snapshots: {
		savedDependencies: Map<string, string>;
		editedDependencies: Map<string, string>;
	};
	lifecycle: {
		packageFileWatchers: vscode.Disposable[];
		externalPackageFileWatchers: Map<string, vscode.Disposable[]>;
	};
}

export function createExtensionState(): ExtensionState {
	return {
		context: undefined,
		session: undefined,
		ui: {
			diagnostics: undefined,
			outputChannel: undefined,
			codeLensRefresh: undefined,
		},
		flags: {
			showVersionLenses: true,
			showOutdated: false,
			showPrereleases: false,
			showSuggestionStats: false,
			providerBusy: 0,
			providerError: false,
			codeLensReplace: true,
		},
		snapshots: {
			savedDependencies: new Map(),
			editedDependencies: new Map(),
		},
		lifecycle: {
			packageFileWatchers: [],
			externalPackageFileWatchers: new Map(),
		},
	};
}
