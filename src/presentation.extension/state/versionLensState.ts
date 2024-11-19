import { ContextState, StateFeatures } from '#extension';
import { SuggestionsOptions } from '#extension/suggestions';
import { throwUndefinedOrNull } from "@esm-test/guards";

export class VersionLensState {

  show: ContextState<boolean>;

  showPrereleases: ContextState<boolean>;

  showOutdated: ContextState<boolean>;

  providerActive: ContextState<string>;

  providerBusy: ContextState<number>;

  providerError: ContextState<boolean>;

  codeLensReplace: ContextState<boolean>;

  constructor(private readonly suggestionOptions: SuggestionsOptions) {
    throwUndefinedOrNull("suggestionOptions", this.suggestionOptions);

    this.show = new ContextState(StateFeatures.Show);
    this.showPrereleases = new ContextState(StateFeatures.ShowPrereleases);
    this.showOutdated = new ContextState(StateFeatures.ShowOutdated);
    this.providerActive = new ContextState(StateFeatures.ProviderActive);
    this.providerBusy = new ContextState(StateFeatures.ProviderBusy);
    this.providerError = new ContextState(StateFeatures.ProviderError);
    this.codeLensReplace = new ContextState(StateFeatures.CodeLenReplace);
  }

  async applyDefaults(): Promise<void> {
    await this.show.change(this.suggestionOptions.showOnStartup);
    await this.showPrereleases.change(this.suggestionOptions.showPrereleasesOnStartup);
    await this.showOutdated.change(false);
    await this.providerActive.change(null);
    await this.providerBusy.change(0);
    await this.providerError.change(false);
    await this.codeLensReplace.change(true);
  }

  async increaseBusyState(): Promise<void> {
    await this.providerBusy.change(this.providerBusy.value + 1);
  }

  async decreaseBusyState(): Promise<void> {
    await this.providerBusy.change(this.providerBusy.value - 1);
  }

  async clearBusyState(): Promise<void> {
    await this.providerBusy.change(0);
  }

  async setErrorState(): Promise<void> {
    await this.providerError.change(true);
  }

  async clearErrorState(): Promise<void> {
    await this.providerError.change(false);
  }

  async enableCodeLensReplace(state: boolean): Promise<void> {
    await this.codeLensReplace.change(state);
  }

}