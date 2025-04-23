/**
 * Interfaces for vscode namespaces
 */
import type {
  CancellationToken,
  ConfigurationScope,
  Event,
  FileSystemWatcher,
  GlobPattern,
  InputBoxOptions,
  MessageOptions,
  QuickPickItem,
  QuickPickOptions,
  Task,
  TaskExecution,
  TaskFilter,
  TaskProcessEndEvent,
  TextDocument,
  TextDocumentShowOptions,
  TextEditor,
  Uri,
  ViewColumn,
  WorkspaceConfiguration,
  WorkspaceEdit,
  WorkspaceEditMetadata
} from 'vscode';

/***
 * Adapter interface for the 'workspace' namespace
 */
export interface IVsCodeWorkspace {
  findFiles(
    include: GlobPattern,
    exclude?: GlobPattern | null,
    maxResults?: number,
    token?: CancellationToken
  ): Thenable<Uri[]>;

  createFileSystemWatcher(
    globPattern: GlobPattern,
    ignoreCreateEvents?: boolean,
    ignoreChangeEvents?: boolean,
    ignoreDeleteEvents?: boolean
  ): FileSystemWatcher;

  applyEdit(edit: WorkspaceEdit, metadata?: WorkspaceEditMetadata): Thenable<boolean>;

  openTextDocument(uri: Uri): Thenable<TextDocument>;

  getConfiguration(
    section?: string,
    scope?: ConfigurationScope | null
  ): WorkspaceConfiguration
}

/***
 * Adapter interface for the 'window' namespace
 */
export interface IVsCodeWindow {
  activeTextEditor: TextEditor | undefined;

  showInputBox(
    options?: InputBoxOptions,
    token?: CancellationToken
  ): Thenable<string | undefined>

  showQuickPick<T extends QuickPickItem>(
    items: readonly T[] | Thenable<readonly T[]>,
    options?: QuickPickOptions & { canPickMany?: false },
    token?: CancellationToken
  ): Thenable<T | undefined>;

  showQuickPick<T extends QuickPickItem>(
    items: readonly T[] | Thenable<readonly T[]>,
    options?: QuickPickOptions & { canPickMany: true },
    token?: CancellationToken
  ): Thenable<T[] | undefined>;

  showInformationMessage<T extends string>(
    message: string,
    options: MessageOptions,
    ...items: T[]
  ): Thenable<T | undefined>;

  showWarningMessage<T extends string>(
    message: string,
    options: MessageOptions,
    ...items: T[]
  ): Thenable<T | undefined>;

  showTextDocument(
    document: TextDocument,
    column?: ViewColumn,
    preserveFocus?: boolean
  ): Thenable<TextEditor>;

  showTextDocument(
    uri: Uri,
    coluoptionsmn?: TextDocumentShowOptions
  ): Thenable<TextEditor>;

}

/***
 * Adapter interface for the 'env' namespace
 */
export interface IVsCodeEnv {
  openExternal(target: Uri): Thenable<boolean>;
}

/***
 * Adapter interface for the 'tasks' namespace
 */
export interface IVsCodeTasks {
  onDidEndTaskProcess: Event<TaskProcessEndEvent>;
  executeTask(task: Task): Thenable<TaskExecution>;
  fetchTasks(filter?: TaskFilter): Thenable<Task[]>;
}

/***
 * Concrete construction factory interface for vscode classes
 */
export interface IVsCodeConstructFactory {
  createWorkspaceEdit(): WorkspaceEdit;
  createUri(uri: string): Uri;
  createFileUri(path: string): Uri;
}

/***
 * Concrete enums
 */
export enum TextDocumentChangeReason {
  /** The text change is caused by an undo operation. */
  Undo = 1,

  /** The text change is caused by an redo operation. */
  Redo = 2,
}