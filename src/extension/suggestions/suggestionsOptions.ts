import type { IConfig } from '#domain/configuration';
import type { SuggestionCategory } from '#domain/packages';
import { SuggestionFeatures } from '#extension';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Accessor for suggestion-related configuration options.
 */
export class SuggestionsOptions {

  /**
   * Initializes a new instance of the SuggestionsOptions class.
   * @param config The underlying configuration source.
   */
  constructor(readonly config: IConfig) {
    throwUndefinedOrNull('config', config);
  }

  /** Gets whether to show version lenses on extension startup. */
  get showOnStartup(): boolean {
    return this.config.get(SuggestionFeatures.ShowOnStartup) ?? false;
  }

  /** Gets whether to show prerelease versions on extension startup. */
  get showPrereleasesOnStartup(): boolean {
    return this.config.get(SuggestionFeatures.ShowPrereleasesOnStartup) ?? false;
  }

  /** Gets whether to show suggestion statistics in the status bar. */
  get showSuggestionsStats(): boolean {
    return this.config.get(SuggestionFeatures.ShowSuggestionsStats) ?? true;
  }

  /** Gets the indicators used for different suggestion categories. */
  get indicators(): Record<SuggestionCategory, string> {
    return this.config.get(SuggestionFeatures.Indicators)!;
  }

}