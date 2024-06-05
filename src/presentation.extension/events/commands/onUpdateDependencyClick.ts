import { throwUndefinedOrNull } from '@esm-test/guards';
import { ILogger } from 'domain/logging';
import { SuggestionTypes, mapToSuggestionUpdate } from 'domain/packages';
import { Disposable } from 'domain/utils';
import { SuggestionCodeLens, SuggestionCommandFeatures } from 'presentation.extension';
import { WorkspaceEdit, commands, workspace } from 'vscode';

export class OnUpdateDependencyClick extends Disposable {

  constructor(readonly logger: ILogger) {
    super();
    throwUndefinedOrNull("logger", logger);

    // register the vscode command
    this.disposable = commands.registerCommand(
      SuggestionCommandFeatures.OnUpdateDependencyClick,
      this.execute,
      this
    );
  }

  /**
   * Executes when a codelens update suggestion is clicked
   * @param codeLens
   */
  async execute(codeLens: SuggestionCodeLens): Promise<void> {
    if (codeLens.preventExtraClicks) return;
    codeLens.preventExtraClicks = true;

    const { version, type } = codeLens.packageResponse.suggestion;
    const isTag = type & SuggestionTypes.tag;
    const suggestionUpdate = mapToSuggestionUpdate(codeLens.packageResponse);
    const replaceWithVersion: string = isTag
      ? version
      : codeLens.replaceVersionFn(suggestionUpdate, version);

    // create and apply the edit
    const edit = new WorkspaceEdit();
    edit.replace(codeLens.documentUrl, codeLens.replaceRange, replaceWithVersion);
    await workspace.applyEdit(edit);
  }

}