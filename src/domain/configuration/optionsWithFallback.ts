import { type IFrozenOptions, Options } from '#domain/configuration';
import type { Nullable } from '#domain/utils';

/**
 * Configuration options that fall back to a different section if a key is not found.
 */
export class OptionsWithFallback extends Options {

  /** The fallback configuration section name. */
  protected fallbackSection: Nullable<string>;

  /**
   * Initializes a new instance of the OptionsWithFallback class.
   * @param config The underlying configuration source.
   * @param section The configuration section name.
   * @param fallbackSection The fallback configuration section name.
   */
  constructor(config: IFrozenOptions, section: string, fallbackSection: Nullable<string> = null) {
    super(config, section);
    this.fallbackSection = fallbackSection;
  }

  /**
   * Gets a configuration value, falling back to the fallback section if not found.
   * @template T The type of the value.
   * @param key The configuration key.
   * @returns The configuration value or undefined.
   */
  get<T>(key: string): T | undefined {
    // attempt to get the section value
    const sectionValue = this.config.get<T>(`${this.section}${key}`);

    // return section value
    if (sectionValue !== null && sectionValue !== undefined) return sectionValue;

    // attempt to get fallback section value
    let fallbackSectionValue: T | undefined;
    if (this.fallbackSection !== null) {
      fallbackSectionValue = this.config.get(`${this.fallbackSection}.${key}`);
    }

    // return fallback key value
    return fallbackSectionValue;
  }

}