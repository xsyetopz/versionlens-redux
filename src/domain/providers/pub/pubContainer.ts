import { IServiceCollection, IServiceProvider } from '#domain/di';
import {
  addCachingOptions,
  addHttpOptions,
  addPubClient,
  addPubConfig,
  addPubJsonClient,
  addSuggestionProvider
} from '#domain/providers/pub';

export async function configureContainer(
  serviceProvider: IServiceProvider,
  services: IServiceCollection
): Promise<IServiceProvider> {

  addCachingOptions(services);

  addHttpOptions(services);

  addPubConfig(services);

  addPubJsonClient(services);

  addPubClient(services);

  addSuggestionProvider(services);

  return await services.buildChild("pub", serviceProvider);
}