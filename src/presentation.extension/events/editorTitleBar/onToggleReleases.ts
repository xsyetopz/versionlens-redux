import { ILogger } from '#domain/logging';
import { Disposable } from '#domain/utils';
import { VersionLensState } from '#extension';
import { SuggestionCodeLensProvider } from '#extension/suggestions';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class OnToggleReleases extends Disposable {

  constructor(
    readonly suggestionCodeLensProviders: SuggestionCodeLensProvider[],
    readonly state: VersionLensState,
    readonly logger: ILogger
  ) {
    super();
    throwUndefinedOrNull("suggestionCodeLensProviders", suggestionCodeLensProviders);
    throwUndefinedOrNull("state", state);
    throwUndefinedOrNull("logger", logger);
  }

  /**
   * Shows or hides version release info
   * @param toggle
   */
  async execute(toggle: boolean): Promise<void> {
    this.logger.debug("toggle version releases = %s", toggle);
    await this.state.show.change(toggle)

    // refresh the active code lenses
    const providerName = this.state.providerActive.value;
    const codelensProvider = this.suggestionCodeLensProviders.find(
      x => x.providerName === providerName
    );

    codelensProvider && codelensProvider.reloadCodeLenses();
  }

}