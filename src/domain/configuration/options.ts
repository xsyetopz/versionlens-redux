import type { IFrozenOptions, IOptions } from '#domain/configuration';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Base class for configuration options tied to a specific section.
 */
export abstract class Options implements IOptions {

  /**
   * Initializes a new instance of the Options class.
   * @param config The underlying configuration source.
   * @param section The configuration section name.
   */
  constructor(
    readonly config: IFrozenOptions,
    protected section: string
  ) {
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('section', section);

    this.section = section.length > 0 ? section + '.' : '';
  }

  /**
   * Gets a configuration value from the current section.
   * @template T The type of the value.
   * @param key The configuration key relative to the section.
   * @returns The configuration value or undefined.
   */
  get<T>(key: string): T | undefined;
  /**
   * Gets a configuration value from the current section.
   * @template T The type of the value.
   * @param key The configuration key relative to the section.
   * @param defaultValue The default value if the key is not found.
   * @returns The configuration value.
   */
  get<T>(key: string, defaultValue: T): T;
  get<T>(key: string, defaultValue?: T): T | undefined {
    return this.config.get(`${this.section}${key}`, defaultValue);
  }

  /**
   * Gets a configuration value or returns a default value.
   * @template T The type of the value.
   * @param key The configuration key.
   * @param defaultValue The default value.
   * @returns The configuration value or the default value.
   */
  getOrDefault<T>(key: string, defaultValue: T): T {
    // attempt to get the section value
    const value = this.get<T>(key);

    // return key value
    if (value !== null && value !== undefined) return value;

    // return default value
    return defaultValue;
  }

  /**
   * Clears the cached configuration.
   */
  defrost(): void {
    this.config.defrost();
  }

}