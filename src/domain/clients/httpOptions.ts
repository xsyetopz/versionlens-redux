import { HttpFeatures } from '#domain/clients';
import { type IFrozenOptions, OptionsWithFallback } from '#domain/configuration';
import type { Nullable } from '#domain/utils';

/**
 * Options for HTTP clients, including SSL strictness.
 */
export class HttpOptions extends OptionsWithFallback {

  /**
   * Initializes a new instance of the HttpOptions class.
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
   * Whether to use strict SSL certificate validation.
   */
  get strictSSL(): boolean {
    return this.getOrDefault<boolean>(
      HttpFeatures.StrictSSL,
      true
    );
  }

}