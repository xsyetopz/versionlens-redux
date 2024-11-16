import type { FileSystemWatcher, Uri } from 'vscode';

export interface IWorkspaceAdapter {
  findFiles: (include: string, exclude: string) => Promise<Uri[]>;
  createFileSystemWatcher: (pattern: string) => FileSystemWatcher;
}