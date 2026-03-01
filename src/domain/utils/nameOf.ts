import { PropertyNameDictionary } from '#domain/utils';

/**
 * Provides a way to get the name of a property in a type-safe manner.
 * @template T The type to extract property names from.
 * @returns A proxy that returns the property name string when accessed.
 * @example nameOf<MyType>().myProperty // returns "myProperty"
 */
export function nameOf<T>() {
  return new Proxy({}, {
    get: (_, prop) => prop,
    set: () => {
      throw Error('Set not supported');
    },
  }) as PropertyNameDictionary<T>;
}