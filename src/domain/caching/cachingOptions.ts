import { CachingFeatures } from '#domain/caching';
import { type IFrozenOptions, OptionsWithFallback } from '#domain/configuration';
import type { Nullable } from '#domain/utils';

/**
 * Options for caching, including cache duration.
 */
export class CachingOptions extends OptionsWithFallback {

  /**
   * Initializes a new instance of the CachingOptions class.
   * @param config The frozen options from the configuration.
   * @param section The configuration section name.
   * @param fallbackSection The fallback configuration section name.
   */
  constructor(
    config: IFrozenOptions,
    section: string,
    fallbackSection: Nullable<string> = null
  ) {
    super(config, section, fallbackSection);
  }

  /**
   * The cache duration in milliseconds.
   */
  get duration(): number {
    const durationMin = this.getOrDefault<number>(
      CachingFeatures.CacheDuration,
      0
    );
    // convert to milliseconds
    return durationMin * 60000;
  }

}