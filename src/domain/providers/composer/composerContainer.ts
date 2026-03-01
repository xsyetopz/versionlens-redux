import type { IServiceCollection, IServiceProvider } from '#domain/di';
import {
  addCachingOptions,
  addComposerConfig,
  addComposerSuggestionResolver,
  addHttpOptions,
  addPackagistClient,
  addSuggestionProvider
} from '#domain/providers/composer';

/**
 * Configures the Composer service container by registering all necessary services.
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

  addComposerConfig(services);

  addPackagistClient(services);

  addComposerSuggestionResolver(services);

  addSuggestionProvider(services);

  return await services.buildChild("composer", serviceProvider);
}