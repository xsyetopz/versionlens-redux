import type { IServiceCollection, IServiceProvider } from '#domain/di';
import {
  addCachingOptions,
  addHttpOptions,
  addPypiConfig,
  addPypiHttpClient,
  addPypiSuggestionResolver,
  addSuggestionProvider
} from '#domain/providers/pypi';

/**
 * Configures the PyPi service container by registering all necessary services.
 * @param serviceProvider The root service provider.
 * @param services The service collection to configure.
 * @returns A promise that resolves to the newly built child service provider.
 */
export async function configureContainer(
  serviceProvider: IServiceProvider,
  services: IServiceCollection
): Promise<IServiceProvider> {

  addCachingOptions(services);

  addHttpOptions(services);

  addPypiConfig(services);

  addPypiHttpClient(services);

  addPypiSuggestionResolver(services);

  addSuggestionProvider(services);

  return await services.buildChild("pypi", serviceProvider);
}