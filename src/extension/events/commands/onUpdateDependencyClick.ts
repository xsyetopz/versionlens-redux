import type { ILogger } from '#domain/logging';
import { SuggestionTypes, mapToSuggestionUpdate } from '#domain/packages';
import { Disposable } from '#domain/utils';
import type { ISuggestionCodeLens, IVersionLensState } from '#extension';
import type { IVsCodeConstructFactory, IVsCodeWorkspace } from '#extension/vscode';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Event handler for when an update dependency suggestion is clicked.
 */
export class OnUpdateDependencyClick extends Disposable {

  /**
   * Initializes a new instance of the OnUpdateDependencyClick class.
   * @param construct Factory for VS Code constructs.
   * @param workspace VS Code workspace interface.
   * @param state Extension state.
   * @param logger Logger instance.
   */
  constructor(
    readonly construct: IVsCodeConstructFactory,
    readonly workspace: IVsCodeWorkspace,
    readonly state: IVersionLensState,
    readonly logger: ILogger
  ) {
    super();
    throwUndefinedOrNull('construct', construct);
    throwUndefinedOrNull('workspace', workspace);
    throwUndefinedOrNull('state', state);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Executes when a codelens update suggestion is clicked.
   * Applies the version update to the document using a WorkspaceEdit.
   * @param codeLens The clicked code lens.
   */
  async execute(codeLens: ISuggestionCodeLens): Promise<void> {
    if (this.state.codeLensReplace.value === false) return;

    // disable codelens replace to prevent suggestion race condition
    await this.state.enableCodeLensReplace(false);

    // get the replace version
    const { version, type } = codeLens.packageResponse.suggestion!;
    const isTag = type & SuggestionTypes.tag;
    const suggestionUpdate = mapToSuggestionUpdate(codeLens.packageResponse);
    const replaceWithVersion: string = isTag
      ? version
      : codeLens.replaceVersionFn(suggestionUpdate, version);

    // apply the edit
    const edit = this.construct.createWorkspaceEdit();
    edit.replace(codeLens.documentUrl, codeLens.replaceRange, replaceWithVersion);
    await this.workspace.applyEdit(edit);
  }

}