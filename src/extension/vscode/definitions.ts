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

/**
 * Adapter interface for the VS Code 'workspace' namespace.
 * Provides a subset of the VS Code workspace API for better testability.
 */
export interface IVsCodeWorkspace {
  /**
   * Finds files in the workspace.
   * @param include A glob pattern that defines which files to search for.
   * @param exclude A glob pattern that defines which files to exclude from the search.
   * @param maxResults An upper-bound for the result set.
   * @param token A cancellation token.
   * @returns A promise resolving to an array of URIs.
   */
  findFiles(
    include: GlobPattern,
    exclude?: GlobPattern | null,
    maxResults?: number,
    token?: CancellationToken
  ): Thenable<Uri[]>;

  /**
   * Creates a file system watcher.
   * @param globPattern A glob pattern that defines which files to watch.
   * @param ignoreCreateEvents Ignore file creation events.
   * @param ignoreChangeEvents Ignore file change events.
   * @param ignoreDeleteEvents Ignore file deletion events.
   * @returns A file system watcher instance.
   */
  createFileSystemWatcher(
    globPattern: GlobPattern,
    ignoreCreateEvents?: boolean,
    ignoreChangeEvents?: boolean,
    ignoreDeleteEvents?: boolean
  ): FileSystemWatcher;

  /**
   * Applies a workspace edit.
   * @param edit The edit to apply.
   * @param metadata Metadata about the edit.
   * @returns A promise resolving to true if the edit was applied.
   */
  applyEdit(edit: WorkspaceEdit, metadata?: WorkspaceEditMetadata): Thenable<boolean>;

  /**
   * Opens a text document.
   * @param uri The URI of the document to open.
   * @returns A promise resolving to the text document.
   */
  openTextDocument(uri: Uri): Thenable<TextDocument>;

  /**
   * Gets a configuration object for a section.
   * @param section The configuration section name.
   * @param scope The configuration scope.
   * @returns The workspace configuration.
   */
  getConfiguration(
    section?: string,
    scope?: ConfigurationScope | null
  ): WorkspaceConfiguration
}

/**
 * Adapter interface for the VS Code 'window' namespace.
 */
export interface IVsCodeWindow {
  /** The currently active text editor. */
  activeTextEditor: TextEditor | undefined;

  /**
   * Shows an input box to the user.
   */
  showInputBox(
    options?: InputBoxOptions,
    token?: CancellationToken
  ): Thenable<string | undefined>

  /**
   * Shows a quick pick list to the user.
   */
  showQuickPick<T extends QuickPickItem>(
    items: readonly T[] | Thenable<readonly T[]>,
    options?: QuickPickOptions & { canPickMany?: false },
    token?: CancellationToken
  ): Thenable<T | undefined>;

  /**
   * Shows a quick pick list allowing multiple selections.
   */
  showQuickPick<T extends QuickPickItem>(
    items: readonly T[] | Thenable<readonly T[]>,
    options?: QuickPickOptions & { canPickMany: true },
    token?: CancellationToken
  ): Thenable<T[] | undefined>;

  /**
   * Shows an information message.
   */
  showInformationMessage<T extends string>(
    message: string,
    options: MessageOptions,
    ...items: T[]
  ): Thenable<T | undefined>;

  /**
   * Shows a warning message.
   */
  showWarningMessage<T extends string>(
    message: string,
    options: MessageOptions,
    ...items: T[]
  ): Thenable<T | undefined>;

  /**
   * Shows a text document in an editor.
   */
  showTextDocument(
    document: TextDocument,
    column?: ViewColumn,
    preserveFocus?: boolean
  ): Thenable<TextEditor>;

  /**
   * Shows a text document by URI.
   */
  showTextDocument(
    uri: Uri,
    options?: TextDocumentShowOptions
  ): Thenable<TextEditor>;

}

/**
 * Adapter interface for the VS Code 'env' namespace.
 */
export interface IVsCodeEnv {
  /**
   * Opens a URL in an external application.
   * @param target The URI to open.
   * @returns A promise resolving to true if successful.
   */
  openExternal(target: Uri): Thenable<boolean>;
}

/**
 * Adapter interface for the VS Code 'tasks' namespace.
 */
export interface IVsCodeTasks {
  /** Event that fires when a task process ends. */
  onDidEndTaskProcess: Event<TaskProcessEndEvent>;
  /**
   * Executes a task.
   * @param task The task to execute.
   * @returns A promise resolving to the task execution.
   */
  executeTask(task: Task): Thenable<TaskExecution>;
  /**
   * Fetches tasks from the environment.
   * @param filter Optional task filter.
   * @returns A promise resolving to an array of tasks.
   */
  fetchTasks(filter?: TaskFilter): Thenable<Task[]>;
}

/**
 * Concrete construction factory interface for VS Code classes.
 * Helps avoid direct dependencies on global 'vscode' classes.
 */
export interface IVsCodeConstructFactory {
  /** Creates a new WorkspaceEdit instance. */
  createWorkspaceEdit(): WorkspaceEdit;
  /** Creates a URI from a string. */
  createUri(uri: string): Uri;
  /** Creates a file URI from a path string. */
  createFileUri(path: string): Uri;
}

/**
 * Enum representing the reason for a text document change.
 */
export enum TextDocumentChangeReason {
  /** The text change is caused by an undo operation. */
  Undo = 1,

  /** The text change is caused by an redo operation. */
  Redo = 2,
}