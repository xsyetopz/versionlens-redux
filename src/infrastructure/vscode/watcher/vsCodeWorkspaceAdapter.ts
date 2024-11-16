import type { FileSystemWatcher, Uri } from 'vscode';
import type { IWorkspaceAdapter } from './definitions';

export class VsCodeWorkspace implements IWorkspaceAdapter {

  constructor(private workspace: any) { }

  findFiles(include: string, exclude: string): Promise<Uri[]> {
    return this.workspace.findFiles(include, exclude);
  }

  createFileSystemWatcher(pattern: string): FileSystemWatcher {
    return this.workspace.createFileSystemWatcher(pattern);
  }

}