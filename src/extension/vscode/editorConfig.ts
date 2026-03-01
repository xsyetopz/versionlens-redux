import type { KeyDictionary } from '#domain/utils';
import type { IVsCodeWorkspace } from '#extension/vscode';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Provides access to standard VS Code editor and file configurations.
 */
export class EditorConfig {

  /**
   * Initializes a new instance of the EditorConfig class.
   * @param workspace The VS Code workspace adapter.
   */
  constructor(readonly workspace: IVsCodeWorkspace) {
    throwUndefinedOrNull('workspace', workspace);
  }

  /**
   * Gets whether code lenses are enabled in the editor settings.
   */
  get codeLens(): boolean {
    return this.workspace.getConfiguration().get('editor.codeLens') ?? true;
  }

  /**
   * Gets the list of excluded files from the VS Code settings.
   */
  get excludeFiles(): KeyDictionary<boolean> {
    const value = this.workspace.getConfiguration()
      .get<KeyDictionary<boolean>>('files.exclude');

    return value || {};
  }

}