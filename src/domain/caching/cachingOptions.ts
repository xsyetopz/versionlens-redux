import { CachingFeatures } from '#domain/caching';
import { type IFrozenOptions, OptionsWithFallback } from '#domain/configuration';
import type { Nullable } from '#domain/utils';

export class CachingOptions extends OptionsWithFallback {

  constructor(
    config: IFrozenOptions,
    section: string,
    fallbackSection: Nullable<string> = null
  ) {
    super(config, section, fallbackSection);
  }

  get duration(): number {
    const durationMin = this.getOrDefault<number>(
      CachingFeatures.CacheDuration,
      0
    );
    // convert to milliseconds
    return durationMin * 60000;
  }

}