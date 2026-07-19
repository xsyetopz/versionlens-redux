import { type Disposable, type Uri, window, workspace } from "#vscode-host";
import { updateContexts } from "../commands/contexts.ts";
import { analyzeDocument } from "../diagnostics/analyze.ts";
import { refreshDiagnostics } from "../diagnostics/refresh.ts";
import { rememberDependencySnapshot } from "../diagnostics/snapshot.ts";
import { fileDocument } from "../documents/file.ts";
import type { ExtensionState } from "../state.ts";
import {
  isDefaultExcluded,
  packageFilePattern,
  packageFilePatterns,
} from "./package-patterns.ts";

async function initializePackageFileWatching(
  state: ExtensionState,
): Promise<void> {
  if ((workspace.workspaceFolders?.length ?? 0) > 0) {
    await scanWorkspacePackageFiles(state);
    watchActivePackageFileOutsideWorkspace(state);
    return;
  }

  scanActivePackageFile(state);
}

async function scanWorkspacePackageFiles(state: ExtensionState): Promise<void> {
  const seenUris = new Set<string>();
  const discoveredFiles = await Promise.all(
    packageFilePatterns().map(async ({ exclude, pattern }) =>
      workspace.findFiles(pattern, exclude),
    ),
  );
  const uniqueFiles = discoveredFiles.flat().filter((uri): boolean => {
    const key = uri.toString();
    if (seenUris.has(key)) {
      return false;
    }
    seenUris.add(key);
    return true;
  });
  await Promise.all(
    uniqueFiles.map((uri) => refreshWatchedPackageFile(state, uri, false)),
  );
}

function scanActivePackageFile(state: ExtensionState): void {
  const document = fileDocument(window.activeTextEditor?.document);
  if (!document) {
    return;
  }

  const output = analyzeDocument(state, document);
  if (!output?.isSupportedManifest) {
    return;
  }

  rememberDependencySnapshot(state, document, output.dependencySignature);
}

function watchActivePackageFileOutsideWorkspace(
  state: ExtensionState,
  document = window.activeTextEditor?.document,
): void {
  const file = fileDocument(document);
  if (!file || workspace.getWorkspaceFolder(file.uri)) {
    return;
  }

  const output = analyzeDocument(state, file);
  if (!output?.isSupportedManifest) {
    return;
  }

  rememberDependencySnapshot(state, file, output.dependencySignature);
  registerExternalPackageFileWatcher(state, file.uri);
}

function disposePackageFileWatchers(state: ExtensionState): void {
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

function registerPackageFileWatchers(
  state: ExtensionState,
  subscriptions?: Disposable[],
): Disposable[] {
  const lifecycle = ensureLifecycle(state);
  disposePackageFileWatchers(state);

  const watchers: Disposable[] = [];
  for (const { pattern } of packageFilePatterns()) {
    const watcher = workspace.createFileSystemWatcher(pattern);
    watchers.push(
      watcher.onDidCreate(
        (uri): Promise<void> => refreshWatchedPackageFile(state, uri, false),
      ),
      watcher.onDidDelete((uri): void => deleteWatchedPackageFile(state, uri)),
      watcher.onDidChange(
        (uri): Promise<void> => refreshWatchedPackageFile(state, uri, true),
      ),
      watcher,
    );
  }

  lifecycle.packageFileWatchers = watchers;
  subscriptions?.push(...watchers);
  return watchers;
}

interface PackageWatcherLifecycle {
  packageFileWatchers: Disposable[];
  externalPackageFileWatchers: Map<string, Disposable[]>;
  sessionGenerations: Map<string, number>;
}

function ensureLifecycle(state: ExtensionState): PackageWatcherLifecycle {
  state.lifecycle ??= {
    externalPackageFileWatchers: new Map(),
    packageFileWatchers: [],
    sessionGenerations: new Map(),
  };
  state.lifecycle.externalPackageFileWatchers ??= new Map();
  return state.lifecycle;
}

function registerExternalPackageFileWatcher(
  state: ExtensionState,
  uri: Uri,
): void {
  const lifecycle = ensureLifecycle(state);
  const key = uri.toString();
  if (lifecycle.externalPackageFileWatchers.has(key)) {
    return;
  }

  const watcher = workspace.createFileSystemWatcher(packageFilePattern(uri));
  const disposables = [
    watcher.onDidCreate(
      (createdUri): Promise<void> =>
        refreshWatchedPackageFile(state, createdUri, false),
    ),
    watcher.onDidDelete((deletedUri): void => {
      deleteWatchedPackageFile(state, deletedUri);
      for (const disposable of disposables) {
        disposable.dispose();
      }
      lifecycle.externalPackageFileWatchers.delete(key);
    }),
    watcher.onDidChange(
      (changedUri): Promise<void> =>
        refreshWatchedPackageFile(state, changedUri, true),
    ),
    watcher,
  ];
  lifecycle.externalPackageFileWatchers.set(key, disposables);
}

async function refreshWatchedPackageFile(
  state: ExtensionState,
  uri: Uri,
  notifyWhenChanged: boolean,
): Promise<void> {
  if (isDefaultExcluded(uri)) {
    return;
  }

  const key = uri.toString();
  const previousSnapshot = state.snapshots.savedDependencies.get(key) ?? "";
  const document = await workspace.openTextDocument(uri);
  const output = analyzeDocument(state, document);
  if (!output) {
    return;
  }

  rememberDependencySnapshot(state, document, output.dependencySignature);
  if (notifyWhenChanged && output.dependencySignature !== previousSnapshot) {
    await refreshActivePackageUi(state, key);
  }
}

function deleteWatchedPackageFile(state: ExtensionState, uri: Uri): void {
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
): Promise<void> {
  const activeDocument = window.activeTextEditor?.document;
  if (activeDocument?.uri.toString() !== changedKey) {
    return;
  }

  await refreshDiagnostics(state, activeDocument);
  await updateContexts(state);
  state.ui.codeLensRefresh?.fire();
}

export {
  disposePackageFileWatchers,
  initializePackageFileWatching,
  registerPackageFileWatchers,
  scanWorkspacePackageFiles,
  watchActivePackageFileOutsideWorkspace,
};
