import { asValue, AwilixContainer, Resolver } from "awilix";
import { KeyDictionary } from '#domain/utils';

/**
 * Resolves and registers asynchronous services as values in the container.
 * @param container The Awilix container.
 * @param asyncSingletons A dictionary of async singleton factory functions.
 * @returns A promise resolving to a dictionary of Awilix resolvers.
 */
export async function registerAsyncSingletons(
  container: AwilixContainer<any>,
  asyncSingletons: KeyDictionary<any>
): Promise<KeyDictionary<Resolver<any>>> {

  const asyncKeys = Object.keys(asyncSingletons)
  const asyncResolvers: KeyDictionary<Resolver<any>> = {};

  for (const key of asyncKeys) {
    const awilixResolver = asValue(
      await asyncSingletons[key](container.cradle)
    );
    asyncResolvers[key] = awilixResolver;
  }

  return asyncResolvers;
}