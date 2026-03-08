import { SuggestionStatusText } from '#domain/packages';
import { GetSuggestionProvider, GetSuggestions } from '#domain/useCases';
import { IVersionLensState, VersionLensExtension } from '#extension';
import { IVsCodeConstructFactory, IVsCodeWorkspace } from '#extension/vscode';
import { OnUpdateDependenciesClick } from './onUpdateDependenciesClick';

/**
 * Event handler for when the "Update dependencies to patch" toolbar icon is clicked.
 */
export class OnUpdateDependenciesPatchClick extends OnUpdateDependenciesClick {

  /**
   * Initializes a new instance of the OnUpdateDependenciesPatchClick class.
   * @param extension The extension instance.
   * @param construct Factory for VS Code constructs.
   * @param workspace VS Code workspace interface.
   * @param state Extension state.
   * @param getSuggestionProvider Use case for identifying suggestion providers.
   * @param getSuggestions Use case for retrieving version suggestions.
   */
  constructor(
    extension: VersionLensExtension,
    construct: IVsCodeConstructFactory,
    workspace: IVsCodeWorkspace,
    state: IVersionLensState,
    getSuggestionProvider: GetSuggestionProvider,
    getSuggestions: GetSuggestions
  ) {
    super(
      extension,
      construct,
      workspace,
      state,
      getSuggestionProvider,
      getSuggestions,
      SuggestionStatusText.UpdatePatch
    );
  }

}
