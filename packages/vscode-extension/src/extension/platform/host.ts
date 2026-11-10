import type * as vscode from "vscode";

export type {
	CodeLens,
	ConfigurationChangeEvent,
	Diagnostic,
	DiagnosticCollection,
	DiagnosticSeverity,
	Disposable,
	DocumentFilter,
	DocumentSelector,
	EventEmitter,
	ExtensionContext,
	LogOutputChannel,
	OutputChannel,
	QuickPickItem,
	Range,
	RelativePattern,
	Task,
	TaskExecution,
	TextDocument,
	TextDocumentChangeEvent,
	Uri,
	WorkspaceConfiguration,
	WorkspaceEdit,
	WorkspaceFolder,
} from "vscode";

export declare const TextDocumentChangeReason: typeof vscode.TextDocumentChangeReason;
export declare const commands: typeof vscode.commands;
export declare const extensions: typeof vscode.extensions;
export declare const languages: typeof vscode.languages;
export declare const tasks: typeof vscode.tasks;
export declare const window: typeof vscode.window;
export declare const workspace: typeof vscode.workspace;
