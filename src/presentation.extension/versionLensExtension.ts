import { throwUndefinedOrNull } from '@esm-test/guards';
import { IFrozenOptions } from '#domain/configuration';
import { VersionLensState } from "presentation.extension";
import { SuggestionsOptions } from "./suggestions/suggestionsOptions";

export class VersionLensExtension {

  static readonly extensionName: string = 'VersionLens';

  constructor(
    readonly config: IFrozenOptions,
    readonly state: VersionLensState,
    readonly suggestionOptions: SuggestionsOptions,
    readonly projectPath: string
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("state", state);
    throwUndefinedOrNull("suggestionOptions", suggestionOptions);
    throwUndefinedOrNull("projectPath", projectPath);
  }

  /**
   * Checks if vscode is in workspace mode
   */
  get isWorkspaceMode() {
    return this.projectPath.length > 0;
  }

}