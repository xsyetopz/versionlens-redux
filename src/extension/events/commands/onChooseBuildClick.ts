import type { ILogger } from '#domain/logging';
import { mapToSuggestionUpdate } from '#domain/packages';
import { Disposable } from '#domain/utils';
import type { ISuggestionCodeLens, IVersionLensState } from '#extension';
import type { SuggestionInteractions } from '#extension/suggestions';
import type { IVsCodeConstructFactory, IVsCodeWorkspace } from '#extension/vscode';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class OnChooseBuildClick extends Disposable {

  constructor(
    readonly interactions: SuggestionInteractions,
    readonly construct: IVsCodeConstructFactory,
    readonly workspace: IVsCodeWorkspace,
    readonly state: IVersionLensState,
    readonly logger: ILogger
  ) {
    super();
    throwUndefinedOrNull('interactions', interactions);
    throwUndefinedOrNull('construct', construct);
    throwUndefinedOrNull('workspace', workspace);
    throwUndefinedOrNull('state', state);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Executes when clicking a build suggestion link
   * @param codeLens
   */
  async execute(codeLens: ISuggestionCodeLens): Promise<void> {
    if (this.state.codeLensReplace.value === false) return;

    const { packageResponse } = codeLens;
    const { package: pkg } = packageResponse.parsedDependency;
    const buildVersions = packageResponse.suggestion.version.split(',');

    // show interactive choices
    const selectedBuild = await this.interactions.chooseBuild(
      buildVersions,
      pkg.name,
      pkg.version
    );
    if (!selectedBuild) return;

    // disable codelens replace to prevent suggestion race condition
    await this.state.enableCodeLensReplace(false);

    // get the replace version
    const suggestionUpdate = mapToSuggestionUpdate(packageResponse);
    const replaceWithVersion = codeLens.replaceVersionFn(suggestionUpdate, selectedBuild);

    // apply the edit
    const edit = this.construct.createWorkspaceEdit();
    edit.replace(codeLens.documentUrl, codeLens.replaceRange, replaceWithVersion);
    await this.workspace.applyEdit(edit);
  }

}