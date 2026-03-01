import type { ILogger } from '#domain/logging';
import { Disposable } from '#domain/utils';
import { VersionLensState } from '#extension/state';
import { SuggestionCodeLensProvider } from '#extension/suggestions';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Event handler for toggling the display of version lenses (releases).
 */
export class OnToggleReleases extends Disposable {

  /**
   * Initializes a new instance of the OnToggleReleases class.
   * @param suggestionCodeLensProviders List of active code lens providers.
   * @param state Extension state.
   * @param logger Logger instance.
   */
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
   * Shows or hides version release info.
   * @param toggle Whether to show version lenses.
   */
  async execute(toggle: boolean): Promise<void> {
    this.logger.debug("toggle version releases = {toggle}", toggle);
    await this.state.show.change(toggle)

    // refresh the active code lenses
    const providerName = this.state.providerActive.value;
    const codelensProvider = this.suggestionCodeLensProviders.find(
      x => x.providerName === providerName
    );

    codelensProvider && codelensProvider.refreshCodeLenses();
  }

}