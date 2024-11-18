import type { CancellationToken, FileSystemWatcher, GlobPattern, Uri } from 'vscode';

export interface IVsCodeWorkspace {
  findFiles(
    include: GlobPattern,
    exclude?: GlobPattern | null,
    maxResults?: number,
    token?: CancellationToken
  ): Promise<Uri[]>;

  createFileSystemWatcher(
    globPattern: GlobPattern,
    ignoreCreateEvents?: boolean,
    ignoreChangeEvents?: boolean,
    ignoreDeleteEvents?: boolean
  ): FileSystemWatcher;
}