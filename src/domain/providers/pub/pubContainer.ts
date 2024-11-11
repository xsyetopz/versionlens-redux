import { IServiceCollection, IServiceProvider } from '#domain/di';
import {
  addCachingOptions,
  addHttpOptions,
  addJsonClient,
  addPubClient,
  addPubConfig,
  addSuggestionProvider
} from '#domain/providers/pub';

export async function configureContainer(
  serviceProvider: IServiceProvider,
  services: IServiceCollection
): Promise<IServiceProvider> {

  addCachingOptions(services);

  addHttpOptions(services);

  addPubConfig(services);

  addJsonClient(services);

  addPubClient(services);

  addSuggestionProvider(services);

  return await services.buildChild("pub", serviceProvider);
}