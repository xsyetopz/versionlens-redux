import { Undefinable } from '#domain/utils';

export interface IConfig {

  /**
   * @param key child key that exists in a configuration source
   * @returns T data retrieved from the specified key
   */
  get<T>(key: string): Undefinable<T>;

}

export interface IFrozenOptions extends IConfig {

  /**
   * Clears the cached configuration so the next call to get(key)
   * will read from the raw configuration source
   */
  defrost(): void;

}

export interface IOptions extends IFrozenOptions { }

export interface IOptionsWithDefaults extends IOptions {

  getOrDefault<T>(key: string, defaultValue: T): T;

}

export type TConfigSectionResolver =  (section: string) => IConfig