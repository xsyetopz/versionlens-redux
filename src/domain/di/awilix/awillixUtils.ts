import { asValue, AwilixContainer, Resolver } from "awilix";
import { KeyDictionary } from '#domain/utils';

/**
 * Resolves async services using proxy injection mode
 * 
 * @param container 
 * @param asyncSingletons 
 * @returns {Promise<KeyDictionary<Resolver<any>>>}
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