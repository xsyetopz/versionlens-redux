import type { IServiceCollection, IServiceProvider } from '#domain/di';
import {
  addCachingOptions,
  addComposerClient,
  addComposerConfig,
  addHttpOptions,
  addJsonClient,
  addPackagistClient,
  addSuggestionProvider
} from '#domain/providers/composer';

export async function configureContainer(
  serviceProvider: IServiceProvider,
  services: IServiceCollection
): Promise<IServiceProvider> {

  addCachingOptions(services);

  addHttpOptions(services);

  addComposerConfig(services);

  addJsonClient(services);

  addPackagistClient(services);

  addComposerClient(services);

  addSuggestionProvider(services);

  return await services.buildChild("composer", serviceProvider);
}