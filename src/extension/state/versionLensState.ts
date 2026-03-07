import { IVersionLensState, StateFeatures } from '#extension';
import { ContextState } from '#extension/state';
import { SuggestionsOptions } from '#extension/suggestions';
import { throwUndefinedOrNull } from "@esm-test/guards";

/**
 * Manages the collective state of the VersionLens extension.
 */
export class VersionLensState implements IVersionLensState {

  /** Whether version lenses are currently shown. */
  show: ContextState<boolean>;

  /** Whether prerelease versions are currently shown. */
  showPrereleases: ContextState<boolean>;

  /** Whether suggestion statistics are currently shown. */
  showSuggestionsStats: ContextState<boolean>;

  /** Whether the active document has outdated dependencies. */
  showOutdated: ContextState<boolean>;

  /** The name of the active suggestion provider, or null if none. */
  providerActive: ContextState<string | null>;

  /** Counter for how many providers are currently busy fetching suggestions. */
  providerBusy: ContextState<number>;

  /** Whether an error occurred in the active provider. */
  providerError: ContextState<boolean>;

  /** Whether code lens replacement is currently enabled. */
  codeLensReplace: ContextState<boolean>;

  /** Whether to show the custom install icon. */
  showCustomInstall: ContextState<boolean>;

  /** Whether to show the alphabetical sort icon. */
  showSortAlphabetically: ContextState<boolean>;

  /**
   * Initializes a new instance of the VersionLensState class.
   * @param suggestionOptions The user-configured suggestion options.
   */
  constructor(readonly suggestionOptions: SuggestionsOptions) {
    throwUndefinedOrNull("suggestionOptions", this.suggestionOptions);

    this.show = new ContextState(StateFeatures.Show);
    this.showPrereleases = new ContextState(StateFeatures.ShowPrereleases);
    this.showSuggestionsStats = new ContextState(StateFeatures.ShowSuggestionsStats);
    this.showOutdated = new ContextState(StateFeatures.ShowOutdated);
    this.providerActive = new ContextState(StateFeatures.ProviderActive);
    this.providerBusy = new ContextState(StateFeatures.ProviderBusy);
    this.providerError = new ContextState(StateFeatures.ProviderError);
    this.codeLensReplace = new ContextState(StateFeatures.CodeLenReplace);
    this.showCustomInstall = new ContextState(StateFeatures.ShowCustomInstall);
    this.showSortAlphabetically = new ContextState(StateFeatures.ShowSortAlphabetically);
  }

  /**
   * Applies the default state values based on user configuration.
   */
  async applyDefaults(): Promise<void> {
    await this.show.change(this.suggestionOptions.showOnStartup);
    await this.showPrereleases.change(this.suggestionOptions.showPrereleasesOnStartup);
    await this.showSuggestionsStats.change(this.suggestionOptions.showSuggestionsStats);
    await this.showOutdated.change(false);
    await this.providerActive.change(null);
    await this.providerBusy.change(0);
    await this.providerError.change(false);
    await this.codeLensReplace.change(true);
    await this.showCustomInstall.change(false);
    await this.showSortAlphabetically.change(false);
  }

  /**
   * Increments the provider busy counter.
   */
  async increaseBusyState(): Promise<void> {
    await this.providerBusy.change(this.providerBusy.value + 1);
  }

  /**
   * Decrements the provider busy counter.
   */
  async decreaseBusyState(): Promise<void> {
    await this.providerBusy.change(Math.max(0, this.providerBusy.value - 1));
  }

  /**
   * Resets the provider busy counter to zero.
   */
  async clearBusyState(): Promise<void> {
    await this.providerBusy.change(0);
  }

  /**
   * Sets the provider error state to true.
   */
  async setErrorState(): Promise<void> {
    await this.providerError.change(true);
  }

  /**
   * Resets the provider error state to false.
   */
  async clearErrorState(): Promise<void> {
    await this.providerError.change(false);
  }

  /**
   * Enables or disables code lens replacement functionality.
   * @param state The new state to set.
   */
  async enableCodeLensReplace(state: boolean): Promise<void> {
    await this.codeLensReplace.change(state);
  }

}