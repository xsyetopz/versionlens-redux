/**
 * Interface for reading configuration values.
 */
export interface IConfig {

  /**
   * Gets a configuration value.
   * @template T The type of the value.
   * @param key The configuration key.
   * @returns The configuration value or undefined.
   */
  get<T>(key: string): T | undefined;
  /**
   * Gets a configuration value.
   * @template T The type of the value.
   * @param key The configuration key.
   * @param defaultValue The default value if the key is not found.
   * @returns The configuration value.
   */
  get<T>(key: string, defaultValue: T): T;
}

/**
 * Interface for options that can be cached and refreshed.
 */
export interface IFrozenOptions extends IConfig {

  /**
   * Clears the cached configuration so the next call to get(key)
   * will read from the raw configuration source.
   */
  defrost(): void;

}

/**
 * Interface for options with a default value fallback.
 */
export interface IOptions extends IFrozenOptions {

  /**
   * Gets a configuration value or returns a default value.
   * @template T The type of the value.
   * @param key The configuration key.
   * @param defaultValue The default value.
   * @returns The configuration value or the default value.
   */
  getOrDefault<T>(key: string, defaultValue: T): T;

}

/**
 * Type for a function that resolves a configuration section.
 * @param section The section name.
 * @returns An IConfig instance for the section.
 */
export type ConfigSectionResolver = (section: string) => IConfig