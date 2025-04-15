import type { KeyDictionary } from '#domain/utils';
import type { IVsCodeWorkspace } from '#extension/vscode';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class EditorConfig {

  constructor(readonly workspace: IVsCodeWorkspace) {
    throwUndefinedOrNull('workspace', workspace);
  }

  get codeLens(): boolean {
    return this.workspace.getConfiguration().get('editor.codeLens') ?? true;
  }

  get excludeFiles(): KeyDictionary<boolean> {
    const value = this.workspace.getConfiguration()
      .get<KeyDictionary<boolean>>('files.exclude');

    return value || {};
  }

}