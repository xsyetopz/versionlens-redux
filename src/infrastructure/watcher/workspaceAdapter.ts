import { FileSystemWatcher, Uri } from 'vscode';
import { IWorkspaceAdapter } from './iWorkspaceAdapter';

export class WorkspaceAdapter implements IWorkspaceAdapter {

  constructor(private workspace: any) { }

  findFiles(include: string, exclude: string): Promise<Uri[]> {
    return this.workspace.findFiles(include, exclude);
  }

  createFileSystemWatcher(pattern: string): FileSystemWatcher {
    return this.workspace.createFileSystemWatcher(pattern);
  }

}