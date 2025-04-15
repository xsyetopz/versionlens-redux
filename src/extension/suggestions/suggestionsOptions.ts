import type { IConfig } from '#domain/configuration';
import type { KeyDictionary } from '#domain/utils';
import { SuggestionFeatures } from '#extension';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class SuggestionsOptions {

  constructor(readonly config: IConfig) {
    throwUndefinedOrNull("config", config);
  }

  get showOnStartup(): boolean {
    return this.config.get(SuggestionFeatures.ShowOnStartup) ?? false;
  }

  get showPrereleasesOnStartup(): boolean {
    return this.config.get(SuggestionFeatures.ShowPrereleasesOnStartup) ?? false;
  }

  get indicators(): KeyDictionary<string> {
    return this.config.get(SuggestionFeatures.Indicators)!;
  }

}