import { basename, dirname } from "node:path";
import * as vscode from "vscode";
import { updateContexts } from "../commands.ts";
import { enabledFilePatternKeys } from "../config/keys/files.ts";
import { analyzeDocument } from "../diagnostics/analyze.ts";
import { rememberDependencySnapshot } from "../diagnostics/snapshot.ts";
import { refreshDiagnostics } from "../diagnostics.ts";
import { fileDocument } from "../documents.ts";
import type { ExtensionState } from "../state.ts";

const defaultExcludes = [
	"**/node_modules/**",
	"**/bower_components/**",
	"**/bin/**",
	"**/.git/**",
	"**/.vscode/**",
] as const;

export async function initializePackageFileWatching(state: ExtensionState) {
	if ((vscode.workspace.workspaceFolders?.length ?? 0) > 0) {
		await scanWorkspacePackageFiles(state);
		watchActivePackageFileOutsideWorkspace(state);
		return;
	}

	scanActivePackageFile(state);
}

export async function scanWorkspacePackageFiles(state: ExtensionState) {
	const seenUris = new Set<string>();
	await Promise.all(
		packageFilePatterns().map(async ({ exclude, pattern }) => {
			const files = await vscode.workspace.findFiles(pattern, exclude);
			for (const uri of files) {
				const key = uri.toString();
				if (seenUris.has(key)) {
					continue;
				}
				seenUris.add(key);
				await refreshWatchedPackageFile(state, uri, false);
			}
		}),
	);
}

function scanActivePackageFile(state: ExtensionState) {
	const document = fileDocument(vscode.window.activeTextEditor?.document);
	if (!document) {
		return;
	}

	const output = analyzeDocument(state, document);
	if (!output?.isSupportedManifest) {
		return;
	}

	rememberDependencySnapshot(state, document, output.dependencySignature);
}

export function watchActivePackageFileOutsideWorkspace(
	state: ExtensionState,
	document = vscode.window.activeTextEditor?.document,
) {
	const file = fileDocument(document);
	if (!file || vscode.workspace.getWorkspaceFolder(file.uri)) {
		return;
	}

	const output = analyzeDocument(state, file);
	if (!output?.isSupportedManifest) {
		return;
	}

	rememberDependencySnapshot(state, file, output.dependencySignature);
	registerExternalPackageFileWatcher(state, file.uri);
}

export function disposePackageFileWatchers(state: ExtensionState) {
	const lifecycle = ensureLifecycle(state);
	for (const disposable of lifecycle.packageFileWatchers) {
		disposable.dispose();
	}
	lifecycle.packageFileWatchers = [];

	for (const disposables of lifecycle.externalPackageFileWatchers.values()) {
		for (const disposable of disposables) {
			disposable.dispose();
		}
	}
	lifecycle.externalPackageFileWatchers.clear();
}

export function registerPackageFileWatchers(
	state: ExtensionState,
	subscriptions?: vscode.Disposable[],
) {
	const lifecycle = ensureLifecycle(state);
	disposePackageFileWatchers(state);

	const watchers: vscode.Disposable[] = [];
	for (const { pattern } of packageFilePatterns()) {
		const watcher = vscode.workspace.createFileSystemWatcher(pattern);
		watchers.push(
			watcher.onDidCreate((uri) =>
				refreshWatchedPackageFile(state, uri, false),
			),
			watcher.onDidDelete((uri) => deleteWatchedPackageFile(state, uri)),
			watcher.onDidChange((uri) => refreshWatchedPackageFile(state, uri, true)),
			watcher,
		);
	}

	lifecycle.packageFileWatchers = watchers;
	subscriptions?.push(...watchers);
	return watchers;
}

function ensureLifecycle(state: ExtensionState) {
	state.lifecycle ??= {
		externalPackageFileWatchers: new Map(),
		packageFileWatchers: [],
	};
	state.lifecycle.externalPackageFileWatchers ??= new Map();
	return state.lifecycle;
}

function registerExternalPackageFileWatcher(
	state: ExtensionState,
	uri: vscode.Uri,
) {
	const lifecycle = ensureLifecycle(state);
	const key = uri.toString();
	if (lifecycle.externalPackageFileWatchers.has(key)) {
		return;
	}

	const watcher = vscode.workspace.createFileSystemWatcher(
		packageFilePattern(uri),
	);
	const disposables = [
		watcher.onDidCreate((createdUri) =>
			refreshWatchedPackageFile(state, createdUri, false),
		),
		watcher.onDidDelete((deletedUri) => {
			deleteWatchedPackageFile(state, deletedUri);
			for (const disposable of disposables) {
				disposable.dispose();
			}
			lifecycle.externalPackageFileWatchers.delete(key);
		}),
		watcher.onDidChange((changedUri) =>
			refreshWatchedPackageFile(state, changedUri, true),
		),
		watcher,
	];
	lifecycle.externalPackageFileWatchers.set(key, disposables);
}

async function refreshWatchedPackageFile(
	state: ExtensionState,
	uri: vscode.Uri,
	notifyWhenChanged: boolean,
) {
	if (isDefaultExcluded(uri)) {
		return;
	}

	const key = uri.toString();
	const previousSnapshot = state.snapshots.savedDependencies.get(key) ?? "";
	const document = await vscode.workspace.openTextDocument(uri);
	const output = analyzeDocument(state, document);
	if (!output) {
		return;
	}

	rememberDependencySnapshot(state, document, output.dependencySignature);
	if (notifyWhenChanged && output.dependencySignature !== previousSnapshot) {
		await refreshActivePackageUi(state, key);
	}
}

function deleteWatchedPackageFile(state: ExtensionState, uri: vscode.Uri) {
	if (isDefaultExcluded(uri)) {
		return;
	}

	const key = uri.toString();
	state.snapshots.savedDependencies.delete(key);
	state.snapshots.editedDependencies.delete(key);
}

async function refreshActivePackageUi(
	state: ExtensionState,
	changedKey: string,
) {
	const activeDocument = vscode.window.activeTextEditor?.document;
	if (activeDocument?.uri.toString() !== changedKey) {
		return;
	}

	await refreshDiagnostics(state, activeDocument);
	await updateContexts(state);
	state.ui.codeLensRefresh?.fire();
}

function packageFilePatterns() {
	if ((vscode.workspace.workspaceFolders?.length ?? 0) === 0) {
		const document = fileDocument(vscode.window.activeTextEditor?.document);
		return document
			? [
					{
						exclude: undefined,
						pattern: packageFilePattern(document.uri),
					},
				]
			: [];
	}

	const config = vscode.workspace.getConfiguration("versionlens");
	const fileExcludes = vscode.workspace
		.getConfiguration("files")
		.get<Record<string, boolean>>("exclude", {});
	const editorExcludes = Object.entries(fileExcludes ?? {})
		.filter(([, enabled]) => enabled)
		.map(([pattern]) => pattern);

	return enabledFilePatternKeys(config.get<string[]>("enabledProviders")).map(
		([, key, , excludePatterns]) => {
			const pattern = config.get<string>(key, "**/*");
			const excludes = [
				...defaultExcludes,
				...editorExcludes,
				...(excludePatterns ?? []),
			];
			return { exclude: mapToSinglePattern(excludes), pattern };
		},
	);
}

function packageFilePattern(uri: vscode.Uri) {
	return new vscode.RelativePattern(dirname(uri.fsPath), basename(uri.fsPath));
}

function mapToSinglePattern(patterns: readonly string[]) {
	return patterns.length === 1 ? patterns[0] : `{${patterns.join(",")}}`;
}

function isDefaultExcluded(uri: vscode.Uri) {
	const path = uri.fsPath.replaceAll("\\", "/");
	return (
		path.includes("/node_modules/") ||
		path.includes("/bower_components/") ||
		path.includes("/bin/") ||
		path.includes("/.git/") ||
		path.includes("/.vscode/")
	);
}
