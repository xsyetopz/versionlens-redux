import { Undefinable } from '#domain/utils';

export interface IConfig {

  /**
   * @param key child key that exists in a configuration source
   * @returns T data retrieved from the specified key
   */
  get<T>(key: string): Undefinable<T>;

}
