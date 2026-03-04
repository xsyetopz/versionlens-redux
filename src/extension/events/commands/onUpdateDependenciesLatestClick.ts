import {
  PackageResponse,
  SuggestionCategory,
  SuggestionStatusText,
  SuggestionTypes,
  defaultReplaceFn,
  mapToSuggestionUpdate
} from '#domain/packages';
import { GetSuggestionProvider, GetSuggestions } from '#domain/useCases';
import { Disposable } from '#domain/utils';
import { IVersionLensState, VersionLensExtension } from '#extension';
import { IVsCodeConstructFactory, IVsCodeWorkspace } from '#extension/vscode';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { dirname } from 'node:path';
import type { TextEditor, Uri } from 'vscode';

type SuggestionEdit = {
  uri: Uri,
  rangeStart: number,
  rangeEnd: number,
  replace: string
}

/**
 * Event handler for when the "Update dependencies to latest" toolbar icon is clicked.
 */
export class OnUpdateDependenciesLatestClick extends Disposable {

  /**
   * Initializes a new instance of the OnUpdateDependenciesLatestClick class.
   * @param extension The extension instance.
   * @param construct Factory for VS Code constructs.
   * @param workspace VS Code workspace interface.
   * @param state Extension state.
   * @param getSuggestionProvider Use case for identifying suggestion providers.
   * @param getSuggestions Use case for retrieving version suggestions.
   */
  constructor(
    readonly extension: VersionLensExtension,
    readonly construct: IVsCodeConstructFactory,
    readonly workspace: IVsCodeWorkspace,
    readonly state: IVersionLensState,
    readonly getSuggestionProvider: GetSuggestionProvider,
    readonly getSuggestions: GetSuggestions
  ) {
    super();
    throwUndefinedOrNull('extension', extension);
    throwUndefinedOrNull('construct', construct);
    throwUndefinedOrNull('workspace', workspace);
    throwUndefinedOrNull('state', state);
    throwUndefinedOrNull('getSuggestionProvider', getSuggestionProvider);
    throwUndefinedOrNull('getSuggestions', getSuggestions);
  }

  /**
   * Executes the update dependencies to latest command.
   * @param textEditor The active text editor.
   */
  async execute(textEditor?: TextEditor): Promise<void> {
    if (!textEditor) return;
    if (this.state.codeLensReplace.value === false) return;

    // get suggestion provider
    const document = textEditor.document;
    const provider = this.getSuggestionProvider.execute(document.fileName);
    if (!provider) return;

    // get the project path from workspace path otherwise the current file
    const packageFilePath = document.uri.fsPath;
    const packagePath = dirname(packageFilePath);
    const projectPath = this.extension.isWorkspaceMode && packagePath.startsWith(this.extension.projectPath)
      ? this.extension.projectPath
      : packagePath;

    // fetch the package suggestions
    let suggestions: Array<PackageResponse> = [];
    try {
      await this.state.increaseBusyState();
      suggestions = await this.getSuggestions.execute(
        provider,
        projectPath,
        packageFilePath,
        this.state.showPrereleases.value
      );
    } catch (error) {
      await this.state.setErrorState();
      return;
    } finally {
      await this.state.decreaseBusyState();
    }
    if (suggestions.length === 0) return;

    // filter the suggestions by latest
    let hasEdits = false;
    const rangesToEdit: Record<string, SuggestionEdit> = {};
    for (const packageResponse of suggestions) {
      if (!packageResponse || !packageResponse.suggestion) continue;

      // only update latest available releases
      const suggestion = packageResponse.suggestion;
      if (suggestion.type !== SuggestionTypes.release) continue;
      if (suggestion.category !== SuggestionCategory.Updateable) continue;
      if (suggestion.name !== SuggestionStatusText.Latest) continue;

      // map to suggestion update
      hasEdits = true;
      const suggestionUpdate = mapToSuggestionUpdate(packageResponse);
      const replaceWithVersion = provider.suggestionReplaceFn
        ? provider.suggestionReplaceFn(suggestionUpdate, suggestion.version)
        : defaultReplaceFn(suggestionUpdate, suggestion.version);
      const rangeStart = suggestionUpdate.parsedVersionRange.start;
      const rangeEnd = suggestionUpdate.parsedVersionRange.end;
      const rangeKey = `${rangeStart}-${rangeEnd}`;

      // map to SuggestionEdit
      rangesToEdit[rangeKey] = {
        uri: document.uri,
        rangeStart,
        rangeEnd,
        replace: replaceWithVersion
      };
    }

    // check if we have edits
    if (hasEdits === false) {
      await this.state.enableCodeLensReplace(true);
      return;
    }

    // disable codelens to prevent suggestion race condition
    await this.state.enableCodeLensReplace(false);

    // replace ranges
    const workspaceEdit = this.construct.createWorkspaceEdit();
    for (const rangeKey in rangesToEdit) {
      const rangeEdit = rangesToEdit[rangeKey]
      const range = this.construct.createRange(
        document.positionAt(rangeEdit.rangeStart),
        document.positionAt(rangeEdit.rangeEnd)
      );
      workspaceEdit.replace(rangeEdit.uri, range, rangeEdit.replace);
    }
    await this.workspace.applyEdit(workspaceEdit);

    // re-enable codelens
    await this.state.enableCodeLensReplace(true);
  }

}
