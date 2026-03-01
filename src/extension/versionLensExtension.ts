import type { IFrozenOptions } from '#domain/configuration';
import { VersionLensState } from '#extension/state';
import { SuggestionsOptions } from '#extension/suggestions';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Represents the VersionLens extension and its core properties.
 */
export class VersionLensExtension {

  /** The display name of the extension. */
  static readonly extensionName: string = 'VersionLens';

  /**
   * Initializes a new instance of the VersionLensExtension class.
   * @param config The application configuration.
   * @param state The extension state.
   * @param suggestionOptions The suggestion options.
   * @param projectPath The root path of the project.
   */
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
   * Gets whether VS Code is currently in workspace mode (i.e., a folder is open).
   */
  get isWorkspaceMode() {
    return this.projectPath.length > 0;
  }

}