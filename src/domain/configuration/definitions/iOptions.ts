import { IConfig } from '#domain/configuration';

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