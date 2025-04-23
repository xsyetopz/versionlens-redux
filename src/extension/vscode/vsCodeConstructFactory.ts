import { Uri, WorkspaceEdit } from 'vscode';
import type { IVsCodeConstructFactory } from './definitions';

/**
 * Constructs vscode global concrete
 * Prevents requiring the editor to run unit tests.
 */
export class VsCodeConstructionFactory implements IVsCodeConstructFactory {

  createWorkspaceEdit(): WorkspaceEdit {
    return new WorkspaceEdit();
  }

  createUri(uri: string): Uri {
    return Uri.parse(uri);
  }

  createFileUri(uri: string): Uri {
    return Uri.file(uri);
  }

}