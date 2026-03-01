import { Uri, WorkspaceEdit } from 'vscode';
import type { IVsCodeConstructFactory } from './definitions';

/**
 * Constructs VS Code global concrete instances.
 * Prevents requiring the full VS Code environment to run unit tests.
 */
export class VsCodeConstructionFactory implements IVsCodeConstructFactory {

  /**
   * Creates a new WorkspaceEdit.
   * @returns A new instance of WorkspaceEdit.
   */
  createWorkspaceEdit(): WorkspaceEdit {
    return new WorkspaceEdit();
  }

  /**
   * Parses a string into a URI.
   * @param uri The URI string to parse.
   * @returns A URI instance.
   */
  createUri(uri: string): Uri {
    return Uri.parse(uri);
  }

  /**
   * Creates a file URI from a path.
   * @param path The file path.
   * @returns A file URI instance.
   */
  createFileUri(path: string): Uri {
    return Uri.file(path);
  }

}