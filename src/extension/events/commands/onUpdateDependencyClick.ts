import type { ILogger } from '#domain/logging';
import { SuggestionTypes, mapToSuggestionUpdate } from '#domain/packages';
import { Disposable } from '#domain/utils';
import type { VersionLensState } from '#extension/state';
import type { SuggestionCodeLens } from '#extension/suggestions';
import type { IVsCodeConstructFactory, IVsCodeWorkspace } from '#extension/vscode';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class OnUpdateDependencyClick extends Disposable {

  constructor(
    readonly construct: IVsCodeConstructFactory,
    readonly workspace: IVsCodeWorkspace,
    readonly state: VersionLensState,
    readonly logger: ILogger
  ) {
    super();
    throwUndefinedOrNull('construct', construct);
    throwUndefinedOrNull('workspace', workspace);
    throwUndefinedOrNull('state', state);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Executes when a codelens update suggestion is clicked
   * @param codeLens
   */
  async execute(codeLens: SuggestionCodeLens): Promise<void> {
    if (this.state.codeLensReplace.value === false) return;

    // disable codelens replace to prevent suggestion race condition
    await this.state.enableCodeLensReplace(false);

    const { version, type } = codeLens.packageResponse.suggestion!;
    const isTag = type & SuggestionTypes.tag;
    const suggestionUpdate = mapToSuggestionUpdate(codeLens.packageResponse);
    const replaceWithVersion: string = isTag
      ? version
      : codeLens.replaceVersionFn(suggestionUpdate, version);

    // create and apply the edit
    const edit = this.construct.createWorkspaceEdit();
    edit.replace(codeLens.documentUrl, codeLens.replaceRange, replaceWithVersion);
    await this.workspace.applyEdit(edit);
  }

}